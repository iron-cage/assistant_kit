# Test: User Story 033 — Topic Forwarding

Test planning for [user_story/033_topic_forwarding.md](../../../../docs/cli/user_story/033_topic_forwarding.md).

The story's seventeen acceptance criteria are verified by thirty-four cases across two
suites — the forwarding half (AC-1..AC-13) by `tests/forward_command_test.rs` (fw01–fw16),
the provisioning half (AC-14..AC-17) by `tests/pool_command_test.rs` (pl01–pl18) — planned
per command in [command/16_delegate.md](../command/16_delegate.md),
[command/17_broadcast.md](../command/17_broadcast.md), and
[command/18_pool.md](../command/18_pool.md). This file maps AC to case, so a criterion that
loses its coverage is visible as a blank row rather than as silence.

Every case in both suites runs `--dry-run`, so neither ever spawns Claude Code nor spends a
token. Two ACs are therefore verified indirectly, and say so below.

## AC → Case Map

| AC | Criterion | Case(s) | Notes |
|----|-----------|---------|-------|
| AC-1 | Delegate picks one live topic and runs the prompt there | fw01 | Asserts exactly one `cmd:` line, naming the topic `topic:` reported |
| AC-2 | Delegate exits with the child's own exit code | — | Not dry-run observable; the relay is three lines in `dispatch_delegate` (`print!`/`eprint!`/`exit`) over `FanoutOutcome`, whose exit-code fidelity — including signal deaths — is covered by `claude_runner_core/tests/fanout_test.rs` tfo04/tfo05 |
| AC-3 | `--pick idle` prefers free topics, falls back rather than refusing | fw04 | The policy is echoed and accepted; the busy/idle split and the fallback flag are unit-tested in `claude_topic_core` against an injected process list (`select_with`), where they are a pure function rather than a race against the machine |
| AC-4 | `--seed N` fixes the draw | fw02, fw03 | fw02 proves stability, fw03 proves it is still a draw — four seeds over four topics reach all four |
| AC-5 | Broadcast runs the prompt in every live topic | fw06 | Three topics across both mechanisms, three commands |
| AC-6 | Blocks are attributed and in listing order | fw06 | Order asserted on the `cmd:` sequence; the header format and its position-based zip are `report()`'s, exercised end-to-end by `fanout_test` tfo03's ordering guarantee |
| AC-7 | At most `--concurrency` children, clamped to the topic count | fw13 | Both directions — an explicit low bound echoed as given, an over-large bound clamped down. The ceiling actually holding is `fanout_test` tfo08, against real children |
| AC-8 | Any failing child fails the broadcast | — | Not dry-run observable; aggregation is `outcomes.iter().filter(!is_success).count()` over results whose per-child exit codes are covered by `fanout_test` tfo04/tfo05/tfo06 |
| AC-9 | Zero-session topics are never targets | fw09, fw10 | One per mechanism — the two halves compute `sessions` differently |
| AC-10 | Every child carries `--topic-mode` | fw07, fw08 | fw07 proves the flag is emitted; fw08 proves it is what keeps a name held in both modes from collapsing to one target |
| AC-11 | An empty base exits 1 | fw11 | Both commands in one case |
| AC-12 | `--dry-run` prints the plan and spawns nothing | fw01–fw15 | Structural: every case asserts on dry-run output, and a spawn would surface as a hang or as a real `claude` invocation in a `PATH`-less container |
| AC-13 | Cross-command flags are rejected by name | fw05, fw14 | fw14 covers both directions; fw05 covers an invalid value of a flag that *does* belong |
| AC-14 | `clr pool <N>` creates the missing pool topics | pl01, pl02, pl04 | pl01 names all of `t1..tN` on an empty base; pl02 fixes the positional form; pl04 proves only the missing ones are planned |
| AC-15 | The same command twice creates nothing the second time | pl03, pl05 | pl03 is the met-target case; pl05 is the harder one — with a hole in the range, "nothing to do" and "extend past the end" are both wrong answers |
| AC-16 | The target is counted against live pool topics | pl06, pl07 | pl06 is the load-bearing case: counting the full set would let `clr pool --count 4 && clr broadcast` reach three. pl07 fixes the other filter — non-pool names, including the non-round-tripping `t01`, are not pool topics |
| AC-17 | One index is one slot across both mechanisms | pl11, pl12 | pl11 proves the selected mode reaches the child; pl12 proves a fork-mode `t1` occupies dir mode's `t1` slot, so the two mechanisms never put two topics in one slot |

Nine `pool` cases verify no AC directly and are not listed above: pl08 (`--prefix` selects an
independent pool), pl09/pl10 (the prefix rejection rules), pl13/pl14/pl15 (the argument
guards), pl16 (the seed message), pl17 (concurrency, the same clamp AC-7 states for
`broadcast`), and pl18 (dispatch). They are command-level contracts planned in
[command/18_pool.md](../command/18_pool.md) rather than restatements of a user-visible
criterion — the story cares that a pool gets provisioned, not what `--prefix t1` does.

## Coverage Summary

- Fully covered by an integration case: 15 of 17 ACs
- Covered indirectly, with the direct test named: 2 of 17 (AC-2, AC-8 — both depend on a
  real child's exit code, which is `claude_runner_core`'s contract and is tested there
  against real processes rather than re-mocked here)

**Implemented by:** `tests/forward_command_test.rs` (fw01–fw16),
`tests/pool_command_test.rs` (pl01–pl18)

**Related:** `claude_runner_core/tests/fanout_test.rs` (tfo01–tfo12) — exit-code fidelity,
concurrency ceiling, input ordering, failure isolation, large-output deadlock;
`claude_topic_core/tests/pool_test.rs` — the naming and gap-filling rules `pool` reports.
