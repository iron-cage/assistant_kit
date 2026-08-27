# Test: `pool`

Integration test planning for the `pool` command. See [command/18_pool.md](../../../../docs/cli/command/18_pool.md) for specification.

`pool` makes sure `N` anonymous topics exist under a base. Cases here (PL-1..PL-18) cover
the target computation and its three filters, the two prefix guards, the mode invariant,
the argument guards, and dispatch. Unlike [`delegate`](16_delegate.md)/[`broadcast`](17_broadcast.md),
`pool` addresses topics that do *not* exist yet, so every case is about what gets planned
rather than about which existing topics get reached.

**Every case runs `--dry-run`** and asserts on the emitted `key: value` and `cmd:` lines.
This is a stronger requirement here than for the forwarding pair: `clr pool` without
`--dry-run` starts one real Claude Code session per missing name — the one command in this
crate whose non-dry-run path costs money by construction. What the children then do is
`clr run`'s contract, covered by `topic_fork_test.rs` (F01–F03, F13) against a stubbed
`claude`; the concurrency primitive is covered by `claude_runner_core/tests/fanout_test.rs`
(tfo01–tfo12) against real child processes.

**Isolation contract** is identical to [`delegate`](16_delegate.md)'s — the same `TopicBase`
fixture, cwd a canonicalized tempdir, environment re-adding only `CLAUDE_HOME` and
`CLR_TOPIC_REGISTRY_DIR`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PL-1 | Empty base, `--count 3` → exactly `t1`, `t2`, `t3` | Target |
| PL-2 | `clr pool 2` and `clr pool --count 2` produce byte-identical stdout | Target |
| PL-3 | A base already holding the target plans nothing — `create: 0`, no `cmd:` lines | Idempotence |
| PL-4 | A partially-filled pool tops up only the difference | Idempotence |
| PL-5 | `t1` + `t3` with target 4 → `t2` and `t4`, not `t4` and `t5` | Idempotence |
| PL-6 | A pool name whose session file is gone is planned again | Live Filter |
| PL-7 | Richly-named and non-round-tripping names (`auth-refactor`, `bench`, `t01`) do not count | Counting |
| PL-8 | `--prefix worker` counts against `worker*` alone, ignoring `t1`/`t2` | Prefix |
| PL-9 | A digit-trailing prefix exits 1, naming the ambiguity | Guards |
| PL-10 | An empty prefix and one containing `/` both exit 1 | Guards |
| PL-11 | `--topic-mode dir` plans dir-mode topics; `fork` is the default | Mode |
| PL-12 | A fork-mode `t1` occupies dir mode's `t1` slot too | Mode |
| PL-13 | A missing count exits 1; a non-numeric count exits 1 quoting it | Guards |
| PL-14 | `--count 0` exits 0 with `create: 0` and no `cmd:` lines | Guards |
| PL-15 | A second positional exits 1, suggesting `--message` | Guards |
| PL-16 | The seed message defaults to `ready`; `--message` overrides it | Seed |
| PL-17 | `-j` is echoed, and clamped to the number being created | Concurrency |
| PL-18 | `pool` is dispatched, has its own `--help`, and is listed in `clr help` | Dispatch |

PL-3 through PL-6 are four separate proofs of one property — that `--count` names a target
rather than an increment — and are kept apart because each fails to a different regression.
PL-3 catches an implementation that appends unconditionally; PL-4 catches one that recreates
the whole range; PL-5 catches one that extends past the highest index instead of filling the
hole a deleted topic left; PL-6 catches one that counts against `enumerate()` rather than
`enumerate_live()`. PL-6 is the load-bearing one: counting the full set would let
`clr pool --count 4` report success while [`broadcast`](17_broadcast.md) reached only three,
which is the silent partial fan-out this command family exists to prevent.

PL-7 and PL-8 are the two halves of "which existing topics count". PL-7 fixes the boundary
from outside the pool — arbitrary names, a dir-mode name, and `t01`, which does not
round-trip through `format!( "{prefix}{index}" )` and so is not a pool name at all. PL-8
fixes it from inside — two prefixes are two independent pools over the same base.

PL-11 and PL-12 are the two halves of the mode invariant
(`claude_topic_core/docs/invariant/002_mode_travels_with_name.md`) as it applies to names
that do not exist yet. PL-11 proves the selected mode reaches the child; PL-12 proves one
index is one *slot* rather than one topic per mechanism — a regression that keyed the count
on `(name, mode)` would pass PL-11 and fail PL-12 by planning a second `t1`.

## Test Coverage Summary

- Target: 2 tests (PL-1, PL-2)
- Idempotence: 3 tests (PL-3, PL-4, PL-5)
- Live Filter: 1 test (PL-6)
- Counting: 1 test (PL-7)
- Prefix: 1 test (PL-8)
- Mode: 2 tests (PL-11, PL-12)
- Guards: 5 tests (PL-9, PL-10, PL-13, PL-14, PL-15)
- Seed: 1 test (PL-16)
- Concurrency: 1 test (PL-17)
- Dispatch: 1 test (PL-18)

**Total:** 18 tests

**Implemented by:** `tests/pool_command_test.rs`

**Related:** `claude_topic_core/tests/pool_test.rs` — the naming rules themselves
(`validate_prefix`, `pool_index`, `missing_names`), unit-tested without a CLI;
`claude_runner_core/tests/fanout_test.rs` — the concurrency primitive, against real children.
