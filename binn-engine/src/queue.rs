//! Hierarchical timing-wheel event queue (U04).
//!
//! O(1) amortized `insert` (append into the mapped slot) and monotone
//! `pop_earliest` for delivery times `at >= now`. Equal-`Tick` events pop in
//! insertion order (deterministic FIFO tie-break).
//!
//! Bucket slots are **not** kept sorted: the hierarchical mapping already places
//! each absolute tick into a slot, and level-0 buckets are homogeneous for the
//! current window. Multi-tick collisions that share a coarser slot are handled
//! by cascade + “front must match `now`” pop — not by re-sorting the bucket.

use std::collections::VecDeque;

use binn_core::time::Tick;

/// Bits per wheel level (256 slots).
const SLOT_BITS: u32 = 8;
/// Slots per level.
const SLOTS: usize = 1 << SLOT_BITS;
/// Levels cover a full `u64` tick space (`8 * 8 = 64`).
const LEVELS: usize = 8;
const SLOT_MASK: u64 = (SLOTS as u64) - 1;

/// Engine-local queued event payload.
///
/// Kept minimal for U04; the LIF engine (U05) schedules richer payloads
/// through this queue without changing the wheel itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Event {
    /// Opaque identity (cell/synapse routing lands in U05).
    pub id: u64,
    /// Optional `f32` payload stored as bits so events retain total equality.
    amount_bits: u32,
}

impl Event {
    /// Create an event with the given opaque id.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self { id, amount_bits: 0 }
    }

    /// Create an event carrying a finite numeric payload.
    #[inline]
    pub fn with_amount(id: u64, amount: f32) -> Self {
        assert!(amount.is_finite(), "event amount must be finite");
        Self {
            id,
            amount_bits: amount.to_bits(),
        }
    }

    /// Decode the numeric payload (`0.0` for [`Event::new`]).
    #[inline]
    pub fn amount(self) -> f32 {
        f32::from_bits(self.amount_bits)
    }
}

#[derive(Clone, Debug)]
struct Entry {
    at: Tick,
    event: Event,
}

/// Hierarchical timing wheel with O(1) append insert.
///
/// # Monotone time
///
/// `insert(at, _)` requires `at >= now()`, where `now` is the tick of the last
/// popped event (or `0` when the queue is empty). This matches the event-driven
/// engine, which only schedules at or after the current simulation time.
pub struct TimingWheel {
    levels: Vec<Vec<VecDeque<Entry>>>,
    /// Cursor: no remaining event has `at < now`.
    now: Tick,
    len: usize,
    /// Cached minimum scheduled tick (append-only buckets are not sorted).
    earliest: Option<Tick>,
    /// Occupancy bitmask: bit `s` of `occupied[level][s / 64]` is set iff
    /// `levels[level][s]` is non-empty.
    ///
    /// # Why this exists
    ///
    /// [`Self::scan_earliest`] used to walk all `LEVELS * SLOTS = 2048`
    /// `VecDeque` headers on every call. A `VecDeque` header is 32 bytes, so
    /// that was ~64 KB of pointer-chased traffic *per tick*, whether the queue
    /// held one event or a million — which contradicts the engine's premise
    /// that work scales with events rather than with the idle population.
    ///
    /// With the mask, `scan_earliest` visits only non-empty buckets, found by
    /// `trailing_zeros` over 32 `u64` words. It computes the same minimum over
    /// the same set of entries, so results are bit-identical.
    ///
    /// # Invariant
    ///
    /// `bit(level, slot) == !levels[level][slot].is_empty()`, maintained at the
    /// three — and only three — sites that mutate `levels` after construction:
    /// `schedule` (push_back), `pop_earliest` (pop_front), and `cascade`
    /// (`mem::take`). [`Self::debug_assert_occupancy`] checks it; the proptests
    /// call it after every operation.
    occupied: [[u64; SLOTS / 64]; LEVELS],
}

impl Default for TimingWheel {
    fn default() -> Self {
        Self::new()
    }
}

impl TimingWheel {
    /// Empty wheel at tick 0.
    pub fn new() -> Self {
        let levels = (0..LEVELS)
            .map(|_| (0..SLOTS).map(|_| VecDeque::new()).collect())
            .collect();
        Self {
            levels,
            now: 0,
            len: 0,
            earliest: None,
            occupied: [[0; SLOTS / 64]; LEVELS],
        }
    }

    /// Set the occupancy bit for `(level, slot)`.
    #[inline]
    fn mark_occupied(&mut self, level: usize, slot: usize) {
        self.occupied[level][slot >> 6] |= 1u64 << (slot & 63);
    }

    /// Clear the occupancy bit for `(level, slot)`.
    #[inline]
    fn mark_empty(&mut self, level: usize, slot: usize) {
        self.occupied[level][slot >> 6] &= !(1u64 << (slot & 63));
    }

    /// Whether `levels[level][slot]` is recorded as non-empty.
    #[inline]
    fn slot_occupied(&self, level: usize, slot: usize) -> bool {
        self.occupied[level][slot >> 6] & (1u64 << (slot & 63)) != 0
    }

    /// Assert the occupancy mask agrees with the actual buckets.
    ///
    /// Test-only and deliberately O(LEVELS * SLOTS): it is the check the mask
    /// exists to avoid, so it must not run on the hot path. Mask drift would
    /// make `scan_earliest` silently miss events — a wrong simulation rather
    /// than a crash — so the proptests call this after every single operation.
    #[cfg(test)]
    fn assert_occupancy_invariant(&self) {
        for level in 0..LEVELS {
            for slot in 0..SLOTS {
                assert_eq!(
                    self.slot_occupied(level, slot),
                    !self.levels[level][slot].is_empty(),
                    "occupancy mask drifted at level {level} slot {slot}"
                );
            }
        }
    }

    /// Current time cursor (last popped tick, or `0` if empty).
    #[inline]
    pub fn now(&self) -> Tick {
        self.now
    }

    /// Number of queued events.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// True when no events are queued.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Earliest queued tick without advancing the cursor.
    ///
    /// O(1) via a cache maintained on insert / pop (buckets are append-only).
    pub fn peek_earliest_tick(&self) -> Option<Tick> {
        self.earliest
    }

    /// Schedule `ev` for delivery at tick `at`.
    ///
    /// # Panics
    ///
    /// Panics if `at < self.now()` (non-monotone insert).
    pub fn insert(&mut self, at: Tick, ev: Event) {
        assert!(
            at >= self.now,
            "TimingWheel::insert requires at >= now (at={at}, now={})",
            self.now
        );
        self.schedule(Entry { at, event: ev });
        self.len += 1;
        self.earliest = Some(match self.earliest {
            Some(e) => e.min(at),
            None => at,
        });
    }

    /// Remove and return the earliest event, or `None` if empty.
    ///
    /// Equal-`Tick` events are returned in insertion order.
    pub fn pop_earliest(&mut self) -> Option<(Tick, Event)> {
        if self.len == 0 {
            return None;
        }

        loop {
            let slot = (self.now & SLOT_MASK) as usize;
            let bucket = &mut self.levels[0][slot];

            if let Some(front) = bucket.front() {
                if front.at == self.now {
                    let entry = bucket.pop_front().expect("front existed");
                    // Mutation site 3 of 3. Captured while `bucket` is still
                    // borrowed; applied below once the borrow has ended.
                    let bucket_now_empty = bucket.is_empty();
                    if bucket_now_empty {
                        self.mark_empty(0, slot);
                    }
                    self.len -= 1;
                    if self.len == 0 {
                        self.now = 0;
                        self.earliest = None;
                    } else {
                        // Same-tick FIFO: remaining events at `now` stay in this
                        // level-0 bucket. Otherwise refresh the earliest cache.
                        let still_same_tick = self.levels[0][slot]
                            .front()
                            .is_some_and(|e| e.at == entry.at);
                        if !still_same_tick {
                            self.earliest = self.scan_earliest();
                        }
                    }
                    return Some((entry.at, entry.event));
                } else if front.at < self.now {
                    unreachable!("missed event at {} while now={}", front.at, self.now);
                }
                // Same low bits, later revolution — leave for later.
            }

            self.advance_cursor();
        }
    }

    /// Drain every event at the current earliest tick in deterministic FIFO
    /// order. This is the delta bucket consumed by U18 parallel stepping.
    pub fn pop_earliest_batch(&mut self) -> Option<(Tick, Vec<Event>)> {
        let (tick, first) = self.pop_earliest()?;
        let mut events = vec![first];
        while self.peek_earliest_tick() == Some(tick) {
            let (same_tick, event) = self.pop_earliest().expect("earliest tick existed");
            debug_assert_eq!(same_tick, tick);
            events.push(event);
        }
        Some((tick, events))
    }

    /// Append into the hierarchical slot for `entry.at` (O(1)).
    ///
    /// Same-tick order is FIFO via `push_back`. Absolute tick order across a
    /// coarser multi-tick slot is recovered on cascade into level-0, where each
    /// current-window slot is a single absolute tick.
    fn schedule(&mut self, entry: Entry) {
        // Fail closed and loud. `Tick` is unsigned, so a cursor that has
        // overtaken this entry underflows here. In debug that panics; in
        // release `overflow-checks` is off, so it would silently wrap to a
        // huge delta, file the event at the wrong level, and reorder delivery.
        // An explicit check makes that corruption impossible to pass silently
        // regardless of profile.
        let delta = entry.at.checked_sub(self.now).unwrap_or_else(|| {
            panic!(
                "TimingWheel cursor overtook a scheduled event: entry.at={} < now={}",
                entry.at, self.now
            )
        });
        let level = level_for(delta);
        let shift = (level as u32) * SLOT_BITS;
        let slot = ((entry.at >> shift) as usize) & (SLOTS - 1);
        self.levels[level][slot].push_back(entry);
        // Mutation site 1 of 3 for the occupancy invariant.
        self.mark_occupied(level, slot);
    }

    /// Minimum scheduled tick over all remaining entries (used when the cached
    /// min is drained).
    ///
    /// Visits only non-empty buckets, located via the occupancy mask. The set
    /// of entries examined — and therefore the returned minimum — is identical
    /// to the previous implementation, which walked all 2048 bucket headers
    /// unconditionally.
    fn scan_earliest(&self) -> Option<Tick> {
        let mut best: Option<Tick> = None;
        for level in 0..LEVELS {
            for word_idx in 0..SLOTS / 64 {
                let mut word = self.occupied[level][word_idx];
                while word != 0 {
                    let slot = word_idx * 64 + word.trailing_zeros() as usize;
                    word &= word - 1; // clear lowest set bit
                    for entry in &self.levels[level][slot] {
                        best = Some(match best {
                            Some(b) => b.min(entry.at),
                            None => entry.at,
                        });
                    }
                }
            }
        }
        best
    }

    /// Brute-force reference for [`Self::scan_earliest`], used by tests to prove
    /// the mask-guided version examines the same entry set.
    #[cfg(test)]
    fn scan_earliest_naive(&self) -> Option<Tick> {
        self.levels
            .iter()
            .flat_map(|level| level.iter())
            .flat_map(|bucket| bucket.iter())
            .map(|entry| entry.at)
            .min()
    }

    /// Move `now` forward to the next tick that holds an event, cascading
    /// coarser levels down as needed.
    ///
    /// The cursor must never pass a still-scheduled entry: `schedule` computes
    /// `entry.at - self.now` on an unsigned `Tick`, so an overtaking cursor
    /// underflows (debug) or wraps to a garbage delta and silently misfiles the
    /// event (release, where `overflow-checks` is off). Every step below is
    /// therefore bounded by `earliest`, the authoritative minimum over all
    /// entries.
    fn advance_cursor(&mut self) {
        let slot = (self.now & SLOT_MASK) as usize;

        // Prefer the next occupied level-0 slot in this window (O(SLOTS) = O(1)).
        if let Some(next) = self.next_occupied_level0(slot + 1) {
            let base = self.now & !SLOT_MASK;
            self.now = base + next as u64;
            return;
        }

        // Exhausted this level-0 window: open the next one — but only if no
        // event falls inside the window we are about to skip over.
        let base = self.now & !SLOT_MASK;
        let next_window = base.saturating_add(SLOTS as u64);
        if self.earliest.is_some_and(|e| e < next_window) {
            self.jump_to_earliest();
            return;
        }
        self.now = next_window;
        self.cascade(1);

        // Events may already sit in level-0 for this window (scheduled earlier).
        if let Some(next) = self.next_occupied_level0(0) {
            let base = self.now & !SLOT_MASK;
            self.now = base + next as u64;
            return;
        }

        // No level-0 work here — jump straight to the earliest scheduled tick.
        self.jump_to_earliest();
    }

    /// Move `now` to the earliest still-scheduled tick and funnel that entry
    /// down into level 0.
    ///
    /// This replaces a per-level forward slot scan that could not distinguish a
    /// slot in the current wheel revolution from the same slot one revolution
    /// later. `schedule` derives an entry's *level* from its delta but its
    /// *slot* from the absolute tick, so a future entry can occupy a slot index
    /// below the cursor's at the same level. The old scan read that as "level
    /// empty", advanced `now` past the entry, and stranded it — the cursor
    /// ended up ahead of a live event and the queue then delivered events out
    /// of order (silently, in release).
    ///
    /// `earliest` is exact — `insert` folds each new tick into it and
    /// `pop_earliest` refreshes it from `scan_earliest` — so jumping to it
    /// lands on the true next event and cannot overshoot.
    fn jump_to_earliest(&mut self) {
        let Some(target) = self.earliest else {
            return;
        };
        debug_assert!(
            target >= self.now,
            "earliest ({target}) is behind the cursor ({})",
            self.now
        );
        self.now = target;
        // Coarse to fine: the entry at `target` re-schedules with `delta == 0`,
        // so each cascade walks it down exactly one level into level 0.
        for level in (1..LEVELS).rev() {
            self.cascade(level);
        }
    }

    fn next_occupied_level0(&self, from_slot: usize) -> Option<usize> {
        let base_now = self.now & !SLOT_MASK;
        for slot in from_slot..SLOTS {
            // Mask check first: skips the `VecDeque` header deref for empty
            // slots, which is the whole cost of this loop in a sparse wheel.
            if !self.slot_occupied(0, slot) {
                continue;
            }
            let at = base_now + slot as u64;
            if self.levels[0][slot].front().is_some_and(|e| e.at == at) {
                return Some(slot);
            }
        }
        None
    }

    /// Cascade slot for `now` at `level` down into lower levels.
    fn cascade(&mut self, level: usize) {
        if level >= LEVELS {
            return;
        }
        let shift = (level as u32) * SLOT_BITS;
        let slot = ((self.now >> shift) as usize) & (SLOTS - 1);

        let entries = std::mem::take(&mut self.levels[level][slot]);
        // Mutation site 2 of 3: `take` leaves the bucket empty. `schedule`
        // re-marks each entry's new (level, slot) below.
        self.mark_empty(level, slot);
        for entry in entries {
            self.schedule(entry);
        }

        // When this level's slot wrapped to 0, pull from the next coarser level.
        if slot == 0 && level + 1 < LEVELS {
            self.cascade(level + 1);
        }
    }
}

#[inline]
fn level_for(delta: Tick) -> usize {
    if delta == 0 {
        return 0;
    }
    let bits = Tick::BITS - delta.leading_zeros();
    let level = ((bits - 1) / SLOT_BITS) as usize;
    level.min(LEVELS - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BinaryHeap;
    use std::time::Instant;

    /// Naive min-heap reference: `(tick, seq, event)`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RefEntry {
        at: Tick,
        seq: u64,
        event: Event,
    }

    impl PartialOrd for RefEntry {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for RefEntry {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            // BinaryHeap is a max-heap: invert so the earliest (at, seq) wins.
            other
                .at
                .cmp(&self.at)
                .then_with(|| other.seq.cmp(&self.seq))
        }
    }

    struct RefQueue {
        heap: BinaryHeap<RefEntry>,
        now: Tick,
        next_seq: u64,
    }

    impl RefQueue {
        fn new() -> Self {
            Self {
                heap: BinaryHeap::new(),
                now: 0,
                next_seq: 0,
            }
        }

        fn insert(&mut self, at: Tick, ev: Event) {
            assert!(at >= self.now);
            let seq = self.next_seq;
            self.next_seq = self.next_seq.wrapping_add(1);
            self.heap.push(RefEntry { at, seq, event: ev });
        }

        fn pop_earliest(&mut self) -> Option<(Tick, Event)> {
            let entry = self.heap.pop()?;
            self.now = entry.at;
            if self.heap.is_empty() {
                self.now = 0;
            }
            Some((entry.at, entry.event))
        }
    }

    #[test]
    fn empty_pop_returns_none() {
        let mut q = TimingWheel::new();
        assert!(q.is_empty());
        assert_eq!(q.pop_earliest(), None);
    }

    #[test]
    fn single_insert_pop() {
        let mut q = TimingWheel::new();
        q.insert(42, Event::new(7));
        assert_eq!(q.peek_earliest_tick(), Some(42));
        assert_eq!(q.pop_earliest(), Some((42, Event::new(7))));
        assert!(q.is_empty());
        assert_eq!(q.now(), 0);
        assert_eq!(q.peek_earliest_tick(), None);
    }

    #[test]
    fn peek_does_not_advance_cursor_or_block_intermediate_insert() {
        let mut q = TimingWheel::new();
        q.insert(100, Event::new(1));
        q.insert(200, Event::new(2));
        assert_eq!(q.peek_earliest_tick(), Some(100));
        assert_eq!(q.now(), 0);
        q.insert(50, Event::new(3));
        assert_eq!(q.peek_earliest_tick(), Some(50));
        assert_eq!(q.pop_earliest(), Some((50, Event::new(3))));
    }

    #[test]
    fn peek_min_with_unsorted_coarser_slot_order() {
        // Later absolute tick inserted first into the same coarser slot; peek
        // must still report the true minimum (append-only buckets).
        let mut q = TimingWheel::new();
        q.insert(300, Event::new(1));
        q.insert(260, Event::new(2));
        assert_eq!(q.peek_earliest_tick(), Some(260));
        assert_eq!(q.pop_earliest(), Some((260, Event::new(2))));
        assert_eq!(q.pop_earliest(), Some((300, Event::new(1))));
    }

    #[test]
    fn pops_in_tick_order() {
        let mut q = TimingWheel::new();
        q.insert(30, Event::new(3));
        q.insert(10, Event::new(1));
        q.insert(20, Event::new(2));
        assert_eq!(q.pop_earliest(), Some((10, Event::new(1))));
        assert_eq!(q.pop_earliest(), Some((20, Event::new(2))));
        assert_eq!(q.pop_earliest(), Some((30, Event::new(3))));
    }

    #[test]
    fn equal_tick_tie_break_is_insertion_order() {
        let mut q = TimingWheel::new();
        let tick = 100u64;
        q.insert(tick, Event::new(1));
        q.insert(tick, Event::new(2));
        q.insert(tick, Event::new(3));
        q.insert(tick + 1, Event::new(4));

        assert_eq!(q.pop_earliest(), Some((tick, Event::new(1))));
        assert_eq!(q.pop_earliest(), Some((tick, Event::new(2))));
        assert_eq!(q.pop_earliest(), Some((tick, Event::new(3))));
        assert_eq!(q.pop_earliest(), Some((tick + 1, Event::new(4))));
    }

    #[test]
    fn equal_tick_tie_break_interleaved_with_other_ticks() {
        let mut q = TimingWheel::new();
        q.insert(5, Event::new(10));
        q.insert(5, Event::new(11));
        q.insert(7, Event::new(20));
        q.insert(5, Event::new(12));

        assert_eq!(q.pop_earliest(), Some((5, Event::new(10))));
        assert_eq!(q.pop_earliest(), Some((5, Event::new(11))));
        assert_eq!(q.pop_earliest(), Some((5, Event::new(12))));
        assert_eq!(q.pop_earliest(), Some((7, Event::new(20))));
    }

    #[test]
    fn far_future_events_via_higher_levels() {
        let mut q = TimingWheel::new();
        q.insert(1, Event::new(1));
        q.insert(1 << 20, Event::new(2));
        q.insert(1 << 40, Event::new(3));
        assert_eq!(q.pop_earliest(), Some((1, Event::new(1))));
        assert_eq!(q.pop_earliest(), Some((1 << 20, Event::new(2))));
        assert_eq!(q.pop_earliest(), Some((1 << 40, Event::new(3))));
    }

    #[test]
    fn parity_vs_binary_heap_scripted() {
        let ops: &[(bool, Tick, u64)] = &[
            (true, 50, 1),
            (true, 10, 2),
            (true, 10, 3),
            (false, 0, 0),
            (true, 10, 4),
            (true, 1000, 5),
            (false, 0, 0),
            (false, 0, 0),
            (false, 0, 0),
            (true, 2000, 6),
            (false, 0, 0),
            (false, 0, 0),
        ];

        let mut wheel = TimingWheel::new();
        let mut refer = RefQueue::new();
        let mut now = 0u64;

        for &(is_insert, at, id) in ops {
            if is_insert {
                let at = at.max(now);
                let ev = Event::new(id);
                wheel.insert(at, ev);
                refer.insert(at, ev);
            } else {
                let w = wheel.pop_earliest();
                let r = refer.pop_earliest();
                assert_eq!(w, r);
                match w {
                    Some((t, _)) => {
                        now = t;
                        if wheel.is_empty() {
                            now = 0;
                        }
                    }
                    None => now = 0,
                }
            }
        }
    }

    /// Insert+drain must stay roughly linear: 1e5 events must not cliff vs 1e3
    /// (sorted `VecDeque::insert` was ~quadratic in bucket occupancy).
    ///
    /// Timed as the *minimum* of several repetitions, not a single sample.
    /// Scheduling noise is one-sided — it can only add time — and the two
    /// measurements are not equally exposed to it: the 1e5 run occupies a
    /// window three orders of magnitude longer than the 1e3 run, so under
    /// `cargo test --workspace`, where every crate's test binary runs at once,
    /// contention inflates the numerator alone. The one observed failure had a
    /// perfectly ordinary small sample (887us, against a 808-1048us band) and a
    /// large sample of 836ms against a 46-55ms band — 17x its usual value, from
    /// code that measured ~48ms in nineteen other runs.
    ///
    /// The minimum tightens this rather than relaxing it. A real quadratic
    /// cliff inflates *every* repetition, so it still trips the same unchanged
    /// threshold; a single unlucky time slice no longer can.
    #[test]
    fn insert_throughput_no_quadratic_cliff() {
        fn fill_and_drain(n: usize) -> std::time::Duration {
            let start = Instant::now();
            let mut q = TimingWheel::new();
            for i in 0..n {
                q.insert((i % 128) as u64, Event::new(i as u64));
            }
            while q.pop_earliest().is_some() {}
            start.elapsed()
        }

        fn best_of(n: usize, reps: usize) -> std::time::Duration {
            (0..reps)
                .map(|_| fill_and_drain(n))
                .min()
                .expect("REPS must be greater than zero")
        }

        const REPS: usize = 3;
        let t_small = best_of(1_000, REPS);
        let t_large = best_of(100_000, REPS);
        let ratio = t_large.as_secs_f64() / t_small.as_secs_f64().max(1e-9);
        // 100× more events: O(n) ⇒ ~100×; old O(n²/slots) ⇒ thousands×.
        assert!(
            ratio < 500.0,
            "insert/drain cliff: 1e5/1e3 time ratio {ratio:.1} \
             (small={t_small:?}, large={t_large:?}, best of {REPS})"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Random monotone workloads: wheel pop order == binary-heap pop order.
        #[test]
        fn prop_parity_vs_heap(
            ops in proptest::collection::vec(
                prop_oneof![
                    (0u8..200u8, any::<u64>()).prop_map(|(offset, id)| (true, offset, id)),
                    Just((false, 0u8, 0u64)),
                ],
                0..200,
            )
        ) {
            let mut wheel = TimingWheel::new();
            let mut refer = RefQueue::new();
            let mut now: Tick = 0;

            for (is_insert, offset, id) in ops {
                if is_insert {
                    let at = now.saturating_add(Tick::from(offset));
                    let ev = Event::new(id);
                    wheel.insert(at, ev);
                    refer.insert(at, ev);
                } else {
                    let w = wheel.pop_earliest();
                    let r = refer.pop_earliest();
                    prop_assert_eq!(w, r);
                    match w {
                        Some((t, _)) => {
                            now = t;
                            if wheel.is_empty() {
                                now = 0;
                            }
                        }
                        None => now = 0,
                    }
                }
            }

            loop {
                let w = wheel.pop_earliest();
                let r = refer.pop_earliest();
                prop_assert_eq!(&w, &r);
                if w.is_none() {
                    break;
                }
            }
        }
    }

    // ---------------------------------------------------------------------
    // Cursor-monotonicity safety net
    //
    // `prop_parity_vs_heap` above draws insert offsets from `0u8..200`, so a
    // whole run stays under ~40k ticks and every entry lives at level 0 or 1.
    // That is below the first wheel-window wrap, so it structurally cannot
    // exercise `skip_empty_and_cascade`'s forward-only slot scan. These tests
    // cover the regime it misses: ticks large enough that an entry's slot index
    // at some level wraps below the cursor's slot index at that same level.
    // ---------------------------------------------------------------------

    /// The cursor must never advance past the earliest still-scheduled tick.
    ///
    /// This is the invariant `schedule` depends on: it computes
    /// `entry.at - self.now` as an unsigned subtraction, so a cursor that
    /// overtakes a live entry underflows (debug) or wraps to a garbage level
    /// (release, where `overflow-checks` is off).
    #[test]
    fn cursor_never_overtakes_live_event() {
        let mut wheel = TimingWheel::new();
        // `at` sits above `now` but its level-3 slot index (8) is far below
        // the cursor's level-3 slot index (252), so a forward-only scan of
        // level 3 skips it and declares the level exhausted.
        let now_seed: Tick = 4_229_946_473;
        let target: Tick = 4_432_430_104;
        // A far-future keeper so the queue never empties; draining it fully
        // would reset the cursor to 0 and mask the defect.
        let keeper: Tick = 5_000_000_000_000;
        wheel.insert(now_seed, Event::new(1));
        wheel.insert(keeper, Event::new(3));

        let (first, _) = wheel.pop_earliest().expect("first event");
        assert_eq!(first, now_seed);
        // Inserted *after* the cursor moved, so `delta` puts it at level 3 —
        // slot 8 of the next level-3 window, while the cursor sits at slot 252
        // of the current one.
        wheel.insert(target, Event::new(2));
        assert!(
            wheel.now() <= target,
            "cursor overtook a live event: now={} but earliest scheduled at={}",
            wheel.now(),
            target
        );
        let (second, ev) = wheel.pop_earliest().expect("second event");
        assert_eq!(
            second, target,
            "events delivered out of order: got tick {second} before {target}"
        );
        assert_eq!(ev, Event::new(2));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Same parity property as `prop_parity_vs_heap`, but with offsets
        /// spanning multiple wheel windows so cascade and cursor-skip paths
        /// actually run.
        #[test]
        fn prop_parity_vs_heap_large_ticks(
            ops in proptest::collection::vec(
                prop_oneof![
                    (0u64..3_000_000_000u64, any::<u64>())
                        .prop_map(|(offset, id)| (true, offset, id)),
                    Just((false, 0u64, 0u64)),
                ],
                0..120,
            )
        ) {
            let mut wheel = TimingWheel::new();
            let mut refer = RefQueue::new();

            for (is_insert, offset, id) in ops {
                if is_insert {
                    let at = wheel.now().saturating_add(offset);
                    let ev = Event::new(id);
                    wheel.insert(at, ev);
                    refer.insert(at, ev);
                } else {
                    let w = wheel.pop_earliest();
                    let r = refer.pop_earliest();
                    prop_assert_eq!(w, r);
                }
                if let Some(min) = wheel.scan_earliest_naive() {
                    prop_assert!(
                        wheel.now() <= min,
                        "cursor {} overtook earliest live tick {}",
                        wheel.now(),
                        min
                    );
                }
            }

            loop {
                let w = wheel.pop_earliest();
                let r = refer.pop_earliest();
                prop_assert_eq!(&w, &r);
                if w.is_none() {
                    break;
                }
            }
        }
    }

    // ---------------------------------------------------------------------
    // Occupancy-mask safety net
    //
    // `scan_earliest` trusts `occupied` to tell it which buckets to look in.
    // If any future edit mutates `levels` without updating the mask, the queue
    // starts silently dropping events instead of panicking — the worst failure
    // mode available here. These tests pin the invariant directly, and check
    // the mask-guided scan against the brute-force scan it replaced.
    // ---------------------------------------------------------------------

    #[test]
    fn occupancy_mask_starts_clear() {
        let wheel = TimingWheel::new();
        wheel.assert_occupancy_invariant();
        assert_eq!(wheel.scan_earliest(), None);
        assert_eq!(wheel.scan_earliest_naive(), None);
    }

    #[test]
    fn occupancy_mask_survives_cascade() {
        // Far-future ticks land in coarse levels and must cascade down through
        // `mem::take` (mutation site 2). Deltas are chosen to straddle several
        // level boundaries (2^8, 2^16, 2^24, 2^32).
        let mut wheel = TimingWheel::new();
        for (i, delta) in [1u64, 300, 70_000, 20_000_000, 5_000_000_000]
            .into_iter()
            .enumerate()
        {
            wheel.insert(delta, Event::new(i as u64));
        }
        wheel.assert_occupancy_invariant();

        let mut popped = Vec::new();
        while let Some((at, ev)) = wheel.pop_earliest() {
            popped.push((at, ev.id));
            wheel.assert_occupancy_invariant();
            assert_eq!(
                wheel.scan_earliest(),
                wheel.scan_earliest_naive(),
                "masked scan diverged from naive scan after popping {at}"
            );
        }
        assert_eq!(
            popped,
            vec![
                (1, 0),
                (300, 1),
                (70_000, 2),
                (20_000_000, 3),
                (5_000_000_000, 4)
            ]
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// After every operation: mask agrees with buckets, and the mask-guided
        /// `scan_earliest` returns exactly what a full brute-force scan returns.
        #[test]
        fn occupancy_mask_and_scan_match_brute_force(
            ops in prop::collection::vec(
                (any::<bool>(), 0u64..2_000_000_000u64, any::<u64>()),
                1..120,
            )
        ) {
            let mut wheel = TimingWheel::new();
            for (is_insert, delta, id) in ops {
                if is_insert {
                    // Anchor on the wheel's own cursor so the monotone-insert
                    // precondition always holds.
                    let at = wheel.now().saturating_add(delta);
                    wheel.insert(at, Event::new(id));
                } else {
                    let _ = wheel.pop_earliest();
                }
                wheel.assert_occupancy_invariant();
                prop_assert_eq!(wheel.scan_earliest(), wheel.scan_earliest_naive());
                // The cached `earliest` must also stay truthful, since
                // `peek_earliest_tick` hands it straight to the engine. Note
                // this checks pre-existing cache logic, not the mask — if only
                // this line fails, look at `insert`/`pop_earliest`, not at
                // `occupied`.
                prop_assert_eq!(wheel.peek_earliest_tick(), wheel.scan_earliest_naive());
            }
        }
    }
}
