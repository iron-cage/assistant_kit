# Test: `delegate`

Integration test planning for the `delegate` command. See [command/16_delegate.md](../../../../docs/cli/command/16_delegate.md) for specification.

`delegate` sends one prompt to one live topic, chosen by policy, as a print-mode `clr run`
child. Cases here (FW-1..FW-8) cover the draw, the guards around it, and the shape of the
command that would be spawned. They share a file and a fixture with
[`broadcast`](17_broadcast.md)'s FW-9..FW-16, because both commands share one argument
parser and one target-enumeration path — testing them apart would duplicate the fixture
without duplicating the coverage.

**Every case runs `--dry-run`.** No case spawns Claude Code, spends a token, or waits on a
child: the assertions are on the resolved base, the chosen target(s), and the exact `cmd:`
line, which is the whole of what this command decides. What the child then does is
`clr run`'s own contract, tested by `tests/cli_args_test.rs` and friends.

**Isolation contract.** `run_cli_in_dir_isolated` pins cwd to a **canonicalized** tempdir —
the fork rule hashes the canonical physical base, so a symlinked `/tmp` would silently
change every expected `UUIDv5` — and scrubs every `CLR_*` topic variable before re-adding
only `CLAUDE_HOME` and `CLR_TOPIC_REGISTRY_DIR`. Fork topics are seeded by writing the
registry line *and* the `UUIDv5`-named session file; dir topics by creating `<base>/-<name>`
*and* a session under that directory's own encoded storage. Seeding the two halves
separately is what makes FW-9's zero-session case constructible.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FW-1 | One live topic → one `cmd:` line naming it, `topic:` reports name and mode | Draw |
| FW-2 | `--seed N` twice over an unchanged list → the same topic both times | Reproducibility |
| FW-3 | Seeds 0..3 over 4 topics reach all 4 — the draw is a draw, not a constant | Reproducibility |
| FW-4 | `--pick random` accepted; the `pick:` line echoes the policy in force | Policy |
| FW-5 | `--pick whatever` → exit 1, stderr naming both valid values | Error Handling |
| FW-11 | Empty base → exit 1 and `no live topics`, not a silent success | Error Handling |
| FW-12 | No message → exit 1 and `requires a message` | Error Handling |
| FW-14 | `-j` on `delegate` → exit 1 naming `clr broadcast` | Error Handling |
| FW-15 | `--` ends option parsing; a hyphen-leading token becomes message text | Parsing |
| FW-16 | `delegate --help` and the top-level help both describe it | Help |

FW-11, FW-12, FW-14, and FW-16 each assert both commands in one case — the guard is shared
code, and a per-command copy would assert the same branch twice.

## Test Coverage Summary

- Draw: 1 test (FW-1)
- Reproducibility: 2 tests (FW-2, FW-3)
- Policy: 1 test (FW-4)
- Error Handling: 4 tests (FW-5, FW-11, FW-12, FW-14)
- Parsing: 1 test (FW-15)
- Help: 1 test (FW-16)

**Total:** 10 tests (6 exclusive to `delegate`, 4 shared with `broadcast`)

**Implemented by:** `tests/forward_command_test.rs`
