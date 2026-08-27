# Test: `--pick` (delegate selection policy)

Integration test planning for the `--pick` parameter. See
[param/090_pick.md](../../../../docs/cli/param/090_pick.md) for the parameter specification.

`--pick` chooses which candidate set `clr delegate` draws from: `idle` (the default — topics
with no turn in flight) or `random` (the full live set). The CLI's contract is narrow — accept
the two values, echo the one in force, reject anything else, and reject the flag outright on
`broadcast` — because the policy *itself* is not implemented here. The busy/idle split and the
all-busy fallback live in `claude_topic_core::select_with`, taking an injected process list, and
are unit-tested there against that list rather than against whatever happens to be running on
the machine. That split is deliberate: a test that shells out and hopes no `claude` is running
is a test that fails on a busy laptop and passes on a quiet one.

Cases are owned by [command/16_delegate.md](../command/16_delegate.md) (FW-1..FW-5) and
implemented in `tests/forward_command_test.rs`; this file maps the parameter to them.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FW-4 | `--pick random` accepted, echoed as `pick: random` | Value |
| FW-5 | An unknown policy exits 1 with stderr naming both valid values | Guards |
| FW-14 | `--pick idle` on `broadcast` exits 1 naming `clr delegate` | Lane |
| FW-1 | The default (`idle`) reaches exactly one live topic | Default |

## Test Coverage Summary

- Value: 1 test (FW-4)
- Guards: 1 test (FW-5)
- Lane: 1 test (FW-14)
- Default: 1 test (FW-1)

**Total:** 4 cases, all shared with [command/16_delegate.md](../command/16_delegate.md)

**Implemented by:** `tests/forward_command_test.rs::fw01`, `fw04`, `fw05`, `fw14`

**Related:** `claude_topic_core/tests/select_test.rs` — the selection policy itself: which
topics count as idle, and the fallback to the full set when every one is busy.

---

### FW-4: `--pick random` is accepted and reported

- **Given:** a base holding one live fork topic
- **Command:** `clr delegate --dry-run --pick random "go"`
- **Expected behavior:** stdout carries `pick: random` — a run reports how it chose, so a
  delegation that reached an unexpected topic can be told apart from one that used an unexpected
  policy
- **Exit:** 0
- **Source:** [param/090_pick.md](../../../../docs/cli/param/090_pick.md)

---

### FW-5: An unknown policy is rejected, not silently defaulted

- **Command:** `clr delegate --dry-run --pick whatever "go"`
- **Expected behavior:** stderr names both valid values (`idle` and `random`). Falling back to
  `idle` would hide the typo and produce a draw the caller did not ask for — indistinguishable,
  from the output, from the policy having worked
- **Exit:** 1
- **Source:** [param/090_pick.md](../../../../docs/cli/param/090_pick.md)

---

### FW-14: `--pick` is rejected on `broadcast` by name

- **Command:** `clr broadcast --dry-run --pick idle "x"`
- **Expected behavior:** stderr names `clr delegate`. `broadcast` has no selection to make — it
  takes every live topic — so accepting and ignoring the flag would imply a filter that does not
  exist
- **Exit:** 1
- **Source:** [param/090_pick.md](../../../../docs/cli/param/090_pick.md)
