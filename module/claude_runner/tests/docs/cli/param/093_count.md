# Test: `--count` (pool target)

Integration test planning for the `--count` parameter. See
[param/093_count.md](../../../../docs/cli/param/093_count.md) for the parameter specification.

`--count` is how many pool topics must exist after [`pool`](../command/18_pool.md) runs. Its one
substantive property — that it names a **target**, never an increment — gets four separate cases
rather than one, because each fails to a different wrong implementation, and three of the four
would pass against the other three's target implementation:

| Wrong implementation | Caught by | Passes |
|----------------------|-----------|--------|
| Appends `N` names unconditionally | PL-3 | PL-1, PL-4, PL-5, PL-6 |
| Recreates the whole `1..=N` range | PL-4 | PL-1, PL-3 |
| Extends past the highest index instead of filling holes | PL-5 | PL-1, PL-3, PL-4 |
| Counts against `enumerate()` rather than `enumerate_live()` | PL-6 | PL-1, PL-3, PL-4, PL-5 |

PL-6 is the load-bearing one. Counting the full set would let a pool name whose session file was
deleted count as present forever, so `clr pool --count 4` would report success while
[`broadcast`](../command/17_broadcast.md) reached only three — a partial fan-out that looks
complete, which is the exact failure this command family exists to prevent.

Cases are owned by [command/18_pool.md](../command/18_pool.md) (PL-1..PL-18) and implemented in
`tests/pool_command_test.rs`; this file maps the parameter to them.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PL-1 | Empty base, `--count 3` → exactly `t1`, `t2`, `t3` | Target |
| PL-2 | `clr pool 2` and `clr pool --count 2` produce byte-identical stdout | Positional |
| PL-3 | A met target plans nothing — `create: 0`, no `cmd:` lines | Idempotence |
| PL-4 | A partial pool tops up only the difference | Idempotence |
| PL-5 | `t1` + `t3` with target 4 → `t2` and `t4`, not `t4` and `t5` | Idempotence |
| PL-6 | A pool name whose session file is gone is planned again | Live Filter |
| PL-13 | A missing count exits 1; a non-numeric count exits 1 quoting it | Guards |
| PL-14 | `--count 0` exits 0 with `create: 0` and no `cmd:` lines | Guards |
| PL-15 | A second positional exits 1, suggesting `--message` | Guards |

## Test Coverage Summary

- Target: 1 test (PL-1)
- Positional: 1 test (PL-2)
- Idempotence: 3 tests (PL-3, PL-4, PL-5)
- Live Filter: 1 test (PL-6)
- Guards: 3 tests (PL-13, PL-14, PL-15)

**Total:** 9 cases, all shared with [command/18_pool.md](../command/18_pool.md)

**Implemented by:** `tests/pool_command_test.rs::pl01`–`pl06`, `pl13`–`pl15`

**Related:** `claude_topic_core/tests/pool_test.rs` — `missing_names()` itself: gap-filling
order, and one index as one slot across both mechanisms.

---

### PL-2: The positional form is the same command

- **Given:** an empty base
- **Commands:** `clr pool --dry-run 2`; `clr pool --dry-run --count 2`
- **Expected behavior:** byte-identical stdout. Asserting on the whole output rather than on the
  planned names catches a positional path that reaches the same *names* through a different
  default — a divergent `prefix:` or `concurrency:` line would slip past a names-only check
- **Exit:** 0 both
- **Source:** [param/093_count.md](../../../../docs/cli/param/093_count.md)

---

### PL-5: Gaps are filled before the range extends

- **Given:** a base holding live `t1` and `t3` (no `t2`)
- **Command:** `clr pool --dry-run --count 4`
- **Expected behavior:** plans exactly `t2` and `t4`. A pool is a set of slots — a deleted topic
  leaves a slot, not a permanent hole — so `t4` and `t5` would be wrong twice over: it strands
  index 2 forever and grows the highest index past the target on every rerun
- **Exit:** 0
- **Source:** [param/093_count.md](../../../../docs/cli/param/093_count.md)

---

### PL-6: A dead pool name counts as missing

- **Given:** a base whose registry names `t1` and `t2` but where `t2`'s session file is absent
- **Command:** `clr pool --dry-run --count 2`
- **Expected behavior:** `existing: 1` and one `cmd:` line naming `t2`. This is what keeps
  `clr pool --count N && clr broadcast` reaching exactly `N` — the two commands must agree on
  what counts as a topic, and `broadcast` counts live ones
- **Exit:** 0
- **Source:** [param/093_count.md](../../../../docs/cli/param/093_count.md)

---

### PL-14: Zero is a no-op, not an error

- **Command:** `clr pool --dry-run --count 0`
- **Expected behavior:** exit 0, `create: 0`, no `cmd:` lines. `clr pool "$N"` from a script that
  computed `N == 0` has asked for nothing, which is a thing to do — failing there would force
  every caller to special-case a value that already means "do nothing"
- **Exit:** 0
- **Source:** [param/093_count.md](../../../../docs/cli/param/093_count.md)

---

### PL-15: A stray second positional is rejected, not joined

- **Command:** `clr pool --dry-run 2 hello`
- **Expected behavior:** exit 1, stderr quoting `hello` and suggesting `--message`. `pool` takes
  a number; reading a stray word as prose would hide the typo that produced it — the opposite of
  `delegate`/`broadcast`, where every positional *is* message text
- **Exit:** 1
- **Source:** [param/093_count.md](../../../../docs/cli/param/093_count.md)
