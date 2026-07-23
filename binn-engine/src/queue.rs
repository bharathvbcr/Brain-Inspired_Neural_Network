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

    /// Append into the hierarchical slot for `entry.at` (O(1)).
    ///
    /// Same-tick order is FIFO via `push_back`. Absolute tick order across a
    /// coarser multi-tick slot is recovered on cascade into level-0, where each
    /// current-window slot is a single absolute tick.
    fn schedule(&mut self, entry: Entry) {
        let delta = entry.at - self.now;
        let level = level_for(delta);
        let shift = (level as u32) * SLOT_BITS;
        let slot = ((entry.at >> shift) as usize) & (SLOTS - 1);
        self.levels[level][slot].push_back(entry);
    }

    /// Full scan for the minimum scheduled tick (used when the cached min is drained).
    fn scan_earliest(&self) -> Option<Tick> {
        self.levels
            .iter()
            .flat_map(|level| level.iter())
            .flat_map(|bucket| bucket.iter())
            .map(|entry| entry.at)
            .min()
    }

    /// Move `now` forward to the next tick that may hold a level-0 event,
    /// cascading higher levels when a wheel window wraps.
    ///
    /// Empty coarser slots are skipped in O(LEVELS · SLOTS) = O(1) time so a
    /// lone far-future event does not scan every intervening tick.
    fn advance_cursor(&mut self) {
        let slot = (self.now & SLOT_MASK) as usize;

        // Prefer the next occupied level-0 slot in this window (O(SLOTS) = O(1)).
        if let Some(next) = self.next_occupied_level0(slot + 1) {
            let base = self.now & !SLOT_MASK;
            self.now = base + next as u64;
            return;
        }

        // Exhausted this level-0 window: open the next window and cascade.
        let base = self.now & !SLOT_MASK;
        self.now = base.saturating_add(SLOTS as u64);
        self.cascade(1);

        // Events may already sit in level-0 for this window (scheduled earlier).
        if let Some(next) = self.next_occupied_level0(0) {
            let base = self.now & !SLOT_MASK;
            self.now = base + next as u64;
            return;
        }

        // No level-0 work here — jump via coarser levels.
        self.skip_empty_and_cascade();
    }

    fn next_occupied_level0(&self, from_slot: usize) -> Option<usize> {
        let base_now = self.now & !SLOT_MASK;
        for slot in from_slot..SLOTS {
            let at = base_now + slot as u64;
            if self.levels[0][slot].front().is_some_and(|e| e.at == at) {
                return Some(slot);
            }
        }
        None
    }

    /// Scan coarser levels for the next non-empty slot, jump `now` there, and
    /// cascade that slot (and coarser wraps) into lower levels.
    fn skip_empty_and_cascade(&mut self) {
        for level in 1..LEVELS {
            let shift = (level as u32) * SLOT_BITS;
            let slot_span = 1u64 << shift;
            let start_slot = ((self.now >> shift) as usize) & (SLOTS - 1);

            // Top level spans the full `u64` timeline (shift+SLOT_BITS == 64).
            let (wheel_base, wheel_span) = if shift + SLOT_BITS >= Tick::BITS {
                (0u64, None)
            } else {
                let span = slot_span << SLOT_BITS;
                (self.now & !(span - 1), Some(span))
            };

            for s in start_slot..SLOTS {
                if !self.levels[level][s].is_empty() {
                    self.now = wheel_base.saturating_add((s as u64).saturating_mul(slot_span));
                    self.cascade(level);
                    return;
                }
            }

            match wheel_span {
                Some(span) => self.now = wheel_base.saturating_add(span),
                None => break, // top level fully scanned
            }
        }
    }

    /// Cascade slot for `now` at `level` down into lower levels.
    fn cascade(&mut self, level: usize) {
        if level >= LEVELS {
            return;
        }
        let shift = (level as u32) * SLOT_BITS;
        let slot = ((self.now >> shift) as usize) & (SLOTS - 1);

        let entries = std::mem::take(&mut self.levels[level][slot]);
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

        let t_small = fill_and_drain(1_000);
        let t_large = fill_and_drain(100_000);
        let ratio = t_large.as_secs_f64() / t_small.as_secs_f64().max(1e-9);
        // 100× more events: O(n) ⇒ ~100×; old O(n²/slots) ⇒ thousands×.
        assert!(
            ratio < 500.0,
            "insert/drain cliff: 1e5/1e3 time ratio {ratio:.1} (small={t_small:?}, large={t_large:?})"
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
}
