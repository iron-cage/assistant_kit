# CLI Parameter: --seed

Fixes [`delegate`](../command/16_delegate.md)'s draw, so the same seed over the same
topic list always reaches the same topic.

- **Type:** u64 (non-negative integer)
- **Default:** absent — a fresh seed from the wall clock mixed with this process's pid
- **Command:** [`delegate`](../command/16_delegate.md)
- **Group:** None — `delegate`'s own surface
- **JSON Key:** none (forwarding commands take no JSON config)

```sh
clr delegate --seed 42 "task"                  # always the same topic, given the same list
clr delegate --seed 42 --dry-run "task"        # preview: the 'seed:' and 'topic:' lines
clr delegate "task"                            # absent: a fresh clock+pid seed each run
```

**The draw is `seed % len` — deliberately the most predictable mapping there is.**
A seed exists so a pick can be reproduced and asserted on, not to supply entropy;
making it unpredictable would defeat the only reason to expose it. Verify:
`clr delegate --dry-run --seed 5 x` twice against an unchanged base prints the same
`topic:` line both times.

**Same seed, same *list*.** The reproducibility is over the candidate set, not over
the base — `seed % len` shifts when `len` changes. Creating a topic, deleting one, or
(under [`--pick idle`](090_pick.md)) one topic becoming busy all change which topic a
fixed seed reaches. For a draw that is stable against busyness, pair it with
`--pick random`, whose candidate set is every live topic regardless of state.

**Where entropy actually matters.** With `--seed` absent, `claude_topic_core::default_seed()`
mixes the wall clock with this process's pid through `SplitMix64`'s finalizer rather
than returning a raw clock reading. The pid is what matters: two `clr delegate` calls
started by the same shell loop can read the same nanosecond on a clock with millisecond
granularity, and without the pid they would then draw the same topic.

**Validation:** a value that does not parse as `u64` is rejected at parse time
(`Error: --seed must be a non-negative integer, got '<VALUE>'`). Negative values fail
this way too — there is no signed form.

**Rejected on `broadcast`.** `clr broadcast --seed 1` exits 1 naming `clr delegate`
rather than ignoring the flag — `broadcast` makes no draw, so a seed for it would be
inert configuration that reads as if it did something. Verify:
`clr broadcast --seed 1 x; echo $?` prints 1.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| integer | u64 | u64 | Parses as a non-negative integer; used as `seed % candidate_count` |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| — | None | `delegate`-only | [`--pick`](090_pick.md) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 16 | [`delegate`](../command/16_delegate.md) | clock+pid | Fixes which member of the candidate set is drawn |
| 17 | [`broadcast`](../command/17_broadcast.md) | N/A | Rejected by name — no draw to fix |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 33 | [033_topic_forwarding.md](../user_story/033_topic_forwarding.md) | Developer |
