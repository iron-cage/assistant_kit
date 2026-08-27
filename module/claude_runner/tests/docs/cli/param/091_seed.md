# Test: `--seed` (reproducible delegate draw)

Integration test planning for the `--seed` parameter. See
[param/091_seed.md](../../../../docs/cli/param/091_seed.md) for the parameter specification.

`--seed` fixes `clr delegate`'s draw so the same seed over the same topic list always reaches
the same topic. Testing it needs two cases, not one, and they pull in opposite directions —
which is the whole point. A draw that always returns `t1` is perfectly reproducible and
completely broken, so proving stability (FW-2) without also proving spread (FW-3) would pass
against a `fn pick() { topics[0] }`.

FW-3 uses four seeds over four topics and asserts all four are reached. That is a deliberate
choice of a *sufficient* rather than a *statistical* check: it fails a constant function and any
mapping that collapses the range, without asserting a distribution the implementation never
promised.

Cases are owned by [command/16_delegate.md](../command/16_delegate.md) (FW-1..FW-5) and
implemented in `tests/forward_command_test.rs`; this file maps the parameter to them.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FW-2 | The same seed twice over the same base reaches the same topic | Stability |
| FW-3 | Four seeds over four topics reach all four | Spread |
| FW-14 | `--seed` on `broadcast` exits 1 naming `clr delegate` | Lane |

## Test Coverage Summary

- Stability: 1 test (FW-2)
- Spread: 1 test (FW-3)
- Lane: 1 test (FW-14)

**Total:** 3 cases, all shared with [command/16_delegate.md](../command/16_delegate.md)

**Implemented by:** `tests/forward_command_test.rs::fw02`, `fw03`, `fw14`

---

### FW-2: The same seed is stable across runs

- **Given:** a base holding four live fork topics
- **Command:** `clr delegate --dry-run --seed 7 "x"`, run twice
- **Expected behavior:** both runs report the same `topic:` line. Stability across *process*
  boundaries is what makes a seed useful — reproducing yesterday's delegation is the use case,
  and a seed stable only within one process would not deliver it
- **Exit:** 0 both times
- **Source:** [param/091_seed.md](../../../../docs/cli/param/091_seed.md)

---

### FW-3: Different seeds reach different topics

- **Given:** the same four-topic base
- **Command:** `clr delegate --dry-run --seed N "x"` for four distinct seeds
- **Expected behavior:** the four runs between them name all four topics. This is the case that
  fails a constant draw — FW-2 alone cannot tell a working seed from an implementation that
  ignores it and always picks the first topic
- **Exit:** 0 for all four
- **Source:** [param/091_seed.md](../../../../docs/cli/param/091_seed.md)

---

### FW-14: `--seed` is rejected on `broadcast` by name

- **Command:** `clr broadcast --dry-run --seed 7 "x"`
- **Expected behavior:** stderr names `clr delegate`. There is no draw to seed when every live
  topic is a target, so the flag could only be inert configuration that reads as if it did
  something
- **Exit:** 1
- **Source:** [param/091_seed.md](../../../../docs/cli/param/091_seed.md)
