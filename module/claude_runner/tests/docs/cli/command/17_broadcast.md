# Test: `broadcast`

Integration test planning for the `broadcast` command. See [command/17_broadcast.md](../../../../docs/cli/command/17_broadcast.md) for specification.

`broadcast` sends one prompt to every live topic, at most `--concurrency` at a time. Cases
here (FW-6..FW-10, FW-13) cover coverage of the target set, the two invariants that decide
what a target *is*, the live filter, and the concurrency clamp. They share a file and a
fixture with [`delegate`](16_delegate.md)'s FW-1..FW-5, and share FW-11/12/14/16 with it
outright — both commands run on one argument parser and one enumeration path.

**Every case runs `--dry-run`** and asserts on the emitted `cmd:` lines, for the same reason
given in [`delegate`](16_delegate.md): what each child then does is `clr run`'s contract,
not this command's. The concurrency *primitive* — the ceiling actually being respected,
input ordering surviving out-of-order completion, failure isolation, no pipe-buffer
deadlock — is tested where it lives, in `claude_runner_core/tests/fanout_test.rs`
(tfo01–tfo12), against real child processes rather than through this CLI.

**Isolation contract** is identical to [`delegate`](16_delegate.md)'s — see there.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FW-6 | Three live topics (2 fork + 1 dir) → three `cmd:` lines, `topics: 3` | Coverage |
| FW-7 | Every emitted command carries `--topic-mode` beside `--topic` | Invariant |
| FW-8 | One name held in BOTH modes → two commands, one per mode | Invariant |
| FW-9 | A registry name whose session file is absent is not a target | Live Filter |
| FW-10 | A `-name` directory with no session storage is not a target | Live Filter |
| FW-13 | `-j 1` echoed as 1; `--concurrency 50` over 2 topics clamped to 2 | Concurrency |
| FW-11 | Empty base → exit 1 and `no live topics`, not a silent success | Error Handling |
| FW-12 | No message → exit 1 and `requires a message` | Error Handling |
| FW-14 | `--pick` on `broadcast` → exit 1 naming `clr delegate` | Error Handling |
| FW-16 | `broadcast --help` and the top-level help both describe it | Help |

FW-7 and FW-8 are the two halves of one invariant
(`claude_topic_core/docs/invariant/002_mode_travels_with_name.md`) and are kept apart on
purpose: FW-7 proves the flag is *emitted*, FW-8 proves the flag is what makes two topics
of one name reachable. A regression that dropped `--topic-mode` would fail both; a
regression that deduplicated targets by name would fail only FW-8.

FW-9 and FW-10 are likewise the two halves of the live filter — one per mechanism — because
`sessions > 0` is computed differently for each: a fork topic's count comes from its single
`UUIDv5` session file's existence, a dir topic's from a `read_dir` of that directory's own
storage. FW-10 is also the regression guard for fan-out reaching `-daemon/`, `-gate/`, and
`./-NNNN_*` scratch directories, which are indistinguishable from dir topics by name alone.

## Test Coverage Summary

- Coverage: 1 test (FW-6)
- Invariant: 2 tests (FW-7, FW-8)
- Live Filter: 2 tests (FW-9, FW-10)
- Concurrency: 1 test (FW-13)
- Error Handling: 3 tests (FW-11, FW-12, FW-14)
- Help: 1 test (FW-16)

**Total:** 10 tests (6 exclusive to `broadcast`, 4 shared with `delegate`)

**Implemented by:** `tests/forward_command_test.rs`

**Related:** `claude_runner_core/tests/fanout_test.rs` — the concurrency primitive itself
(ceiling, ordering, failure isolation, large-output deadlock), against real children.
