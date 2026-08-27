# Feature: Topic Selection

### Scope

- **Purpose**: Choose which topic a forwarded prompt should go to, preferring one that is not already mid-turn.
- **In Scope**: `Pick`, `Selection`, `is_busy`, `select`, `select_with`, `default_seed`.
- **Out of Scope**: Finding the candidates (→ [002](002_topic_enumeration.md)); actually sending the prompt (→ [`claude_runner`](../../../claude_runner/docs/cli/command/readme.md)); guaranteeing exclusive access once chosen (→ [005](005_topic_lock.md)).

### Why This Exists

The reason to hand a prompt to a topic instead of the session in front of you is
usually that the session in front of you is busy. A uniform draw over every topic
reproduces exactly that problem one level down: it will cheerfully pick a topic
that is mid-turn, and the second prompt queues behind the first.

So the default policy is `Idle` — draw uniformly from the topics that are *not*
mid-turn. `Random` is available for callers who want the literal semantics.

### The Busy Test

Busyness is judged against an already-collected process list, not per topic. One
`/proc` sweep answers the question for every candidate; a sweep per topic would also
give each candidate a different instant to be judged at, which is a different — and
worse — question than the one being asked.

| Mode | Busy when |
|------|-----------|
| Fork | Some live `claude` carries the topic's deterministic session id in argv — `--resume <id>` or `--session-id <id>`, either spelling, because the id is matched as a bare argument rather than as a flag's value |
| Dir | Some live `claude` is running in the topic's directory — there is no name-derived id to match on, so the directory is the identity |

Both are inference from a process list, not a claim of exclusion. Nothing here
prevents a turn from starting one instruction after the sweep read `/proc`; that is
what [005](005_topic_lock.md) is for.

### The Fallback

When every candidate is busy, `Idle` falls back to the full set rather than failing.
The caller asked for a topic, and "all of them are working" is a reason to *say so*,
not a reason to refuse. `Selection::all_busy` carries that fact back so it can be
reported — the prompt will queue, and the person who sent it should know.

### Seeds

The draw is `seed % len` — deliberately the most predictable mapping there is. A
seed exists so a pick can be reproduced and asserted on, not to supply entropy;
making the mapping unpredictable would defeat the only reason to expose the seed at
all.

`default_seed` is the one place entropy quality matters. It mixes the wall clock
with this process's id through `SplitMix64`'s finalizer rather than returning a raw
clock reading, because two `clr` invocations started by the same shell loop can read
the same nanosecond on a coarse clock — and would then draw the same topic.

### Two Entry Points

`select_with` takes the process list; `select` performs the sweep and delegates. The
split is what makes a draw assertable: with the list supplied, the outcome is a pure
function of topics, policy, seed, and processes, with nothing read from the machine
it runs on. `select` skips the sweep entirely under `Random`, where busyness is
ignored and the scan would be pure cost.

### Verification

```bash
cd module/claude_topic_core && ./verb/test
```

Or the single test binary, in-container:

```bash
cargo test -p claude_topic_core --test select_test
```

The cases that matter are tsl03 and tsl04: a uniform draw passes the `Random` case
and fails both of those, which is precisely the bug the policy exists to prevent.

On a real machine, the busy set is whatever `/proc` says:

```bash
clr ps                          # the live claude processes, as clr sees them
pgrep -a claude                 # the raw argv the busy test matches against
```

A fork topic is busy exactly when its id — `clr topics --file <name>`'s filename
stem — appears in that argv.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/select.rs` | Policy, busy test, draw, seed |
| doc | [002_topic_enumeration.md](002_topic_enumeration.md) | Where the candidates come from |
| doc | [005_topic_lock.md](005_topic_lock.md) | Turning an inference into exclusion |
| doc | [api/001_topic_surface.md](../api/001_topic_surface.md) | Full signature contract |
| test | `tests/select_test.rs` | Busy detection per mode, idle preference, seeded determinism |
