# Defect — concurrent reference cells race on one git checkout, and two died silently

**Found:** 2026-08-23, running the six-cell reference set registered in
`PREREG_2026-08-22_REFERENCE_RERUN.md`.

**Status: not fixed.** The fix cannot be applied until the calibration set is
complete — see §3. The cells were recovered by a workaround.

---

## 1. What happened

The driver started `historical` seeds 5170001, 5170002 and 5170003 at the same
instant. Two failed **in the same second**:

```
2026-08-23T03:05:53Z START historical 5170002
2026-08-23T03:05:53Z FAIL  historical 5170002
```

```
Another git process seems to be running in this repository ...
subprocess.CalledProcessError: Command '['git', 'checkout', '--quiet', '--detach',
'd169b4e3049a3d5bff56c84a8b2f0c4e835aafda']' returned non-zero exit status 128
```

`reference.py:75` (`ensure_checkout`) runs `git checkout` against a single shared
clone at `results/shd_instrument_v4/reference-cache`, and
`prepare_seed_worktree` then runs `git worktree add` against the same clone.
Neither takes a lock. Concurrent cells contend on `index.lock` and the losers
die before training starts.

**This was my error in how the run was driven**, not a pre-existing bug being
triggered by normal use — the orchestration has no documented concurrency
contract either way. But the tooling has no guard, so the next person to run two
cells at once loses the same way.

## 2. Why it was not caught for two hours

The failure is instantaneous and the driver logged `FAIL` correctly — but nothing
was reading the driver log, and the *watcher* was waiting on artifact count and
process liveness. With one historical cell still training for another 2.5 hours,
`4/6 artifacts` and `driver alive` are indistinguishable from healthy progress.

A watcher that cannot tell "still working" from "two of these are already dead"
is the same shape as the checks this workspace keeps finding: it reports the same
thing whether or not the thing it watches is fine.

## 3. Why it is not fixed yet

The correct fix is a lock around the git operations in
`scripts/shd_calibration/reference.py`.

**That file is in `REFERENCE_SOURCE_PATHS`** (`runner.py:176-180`), which is
exactly the point of the narrow scope registered in
`AMENDMENT_2026-08-22_REFERENCE_FINGERPRINT_SCOPE_FORWARD.md`: a reference
artifact is fingerprinted over the code that can affect a reference run, and
`reference.py` can. Editing it now would move the narrow fingerprint and
**invalidate the four cells already produced**, which cost 22 CPU-hours.

So the fix waits until the set is complete. Recorded here so it is not lost.

## 4. The workaround, which is a workaround

The two failed cells were re-launched with a **180-second stagger**. The race is
only in the setup — checkout and worktree creation — not in training, so a gap
long enough for the first checkout to finish avoids it without touching
fingerprinted code. Both cells started and are training.

This is timing, not synchronisation. It will fail under a slower disk or a colder
cache, and it is not a substitute for the lock.

## 5. What to do after the calibration set lands

1. Take an exclusive file lock around `ensure_checkout` and
   `prepare_seed_worktree` in `reference.py`, so concurrent cells serialise
   through setup and then train in parallel.
2. Register the edit as a new-artifact provenance event, since it moves the
   narrow fingerprint — meaning **the six cells must be produced before it, or
   again after it**, not straddling.
3. Give the driver a watcher that fails loudly on a `FAIL` line rather than
   waiting out a count.
