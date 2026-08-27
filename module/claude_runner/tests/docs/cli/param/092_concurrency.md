# Test: `--concurrency` (fan-out bound)

Integration test planning for the `--concurrency` parameter. See
[param/092_concurrency.md](../../../../docs/cli/param/092_concurrency.md) for the parameter
specification.

`--concurrency` bounds how many `clr run` children are in flight at once, on both
[`broadcast`](../command/17_broadcast.md) and [`pool`](../command/18_pool.md), and is rejected
by name on `delegate`. Two layers are tested in two places, and keeping them apart is the point:

- **The CLI layer, here.** That the flag parses, that the *clamped* value is what gets reported,
  and that the flag is refused where it would be inert. These run through `--dry-run`, so no
  child is ever spawned.
- **The primitive, in `claude_runner_core/tests/fanout_test.rs`** (tfo01–tfo12). That the
  ceiling actually holds, that input order survives out-of-order completion, that one failing
  child does not take down its siblings, and that a child writing more than a pipe buffer's
  worth does not deadlock. Those need *real* concurrent child processes; asserting them through
  a dry-run would assert nothing at all.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FW-13 | `broadcast -j 1` echoed as 1; `--concurrency 50` over 2 topics clamped to 2 | Clamp |
| PL-17 | `pool -j` echoed; a bound above the number being created clamped down to it | Clamp |
| FW-14 | `-j` on `delegate` exits 1 naming `clr broadcast` | Lane |

## Test Coverage Summary

- Clamp: 2 tests (FW-13, PL-17) — one per accepting command
- Lane: 1 test (FW-14)

**Total:** 3 cases, shared with [command/17_broadcast.md](../command/17_broadcast.md) and
[command/18_pool.md](../command/18_pool.md)

**Implemented by:** `tests/forward_command_test.rs::fw13`, `fw14`;
`tests/pool_command_test.rs::pl17`

**Related:** `claude_runner_core/tests/fanout_test.rs` (tfo01–tfo12) — the ceiling actually
being respected, input ordering, failure isolation, and large-output deadlock, against real
child processes.

---

### FW-13: The reported bound is the clamped bound, both directions

- **Given:** a base holding two live topics
- **Commands:** `clr broadcast --dry-run -j 1 "x"`; `clr broadcast --dry-run --concurrency 50 "x"`
- **Expected behavior:** the first reports `concurrency: 1` (an explicit low bound is honoured
  as given), the second `concurrency: 2` (an unreachable bound is clamped to the topic count).
  Both directions matter: reporting the *requested* value would tell the user four children are
  in flight when two exist, and clamping a low bound upward would silently raise a token-spend
  rate the user deliberately lowered
- **Exit:** 0 both
- **Source:** [param/092_concurrency.md](../../../../docs/cli/param/092_concurrency.md)

---

### PL-17: The same clamp applies to the number being created

- **Given:** an empty base
- **Commands:** `clr pool --dry-run -j 1 --count 3`; `clr pool --dry-run --concurrency 50 --count 2`
- **Expected behavior:** the first reports `concurrency: 1`, the second `concurrency: 2` — the
  bound is clamped to the number of *names being created*, not to the number of topics that
  exist (zero in both cases). Clamping against the existing count would floor `pool` at 1 on
  every empty base, which is exactly the case it is built for
- **Exit:** 0 both
- **Source:** [param/092_concurrency.md](../../../../docs/cli/param/092_concurrency.md)

---

### FW-14: `-j` is rejected on `delegate` by name

- **Command:** `clr delegate --dry-run -j 2 "x"`
- **Expected behavior:** stderr names `clr broadcast`. `delegate` runs exactly one child, so a
  concurrency bound for it would be inert configuration that reads as if it did something
- **Exit:** 1
- **Source:** [param/092_concurrency.md](../../../../docs/cli/param/092_concurrency.md)
