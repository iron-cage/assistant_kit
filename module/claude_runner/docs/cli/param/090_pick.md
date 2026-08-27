# CLI Parameter: --pick

Selects which topic [`delegate`](../command/16_delegate.md) draws from — `idle`
(only topics with no turn in flight) or `random` (every live topic, busy or not).

- **Type:** enum — `idle` | `random`
- **Default:** `idle`
- **Command:** [`delegate`](../command/16_delegate.md)
- **Group:** None — `delegate`'s own surface
- **JSON Key:** none (forwarding commands take no JSON config)

```sh
clr delegate "task"                        # idle: skip topics mid-turn
clr delegate --pick idle "task"            # the same, stated explicitly
clr delegate --pick random "task"          # draw from all of them, no /proc scan
clr delegate --pick random --dry-run "x"   # preview: the 'pick:' line echoes the policy
```

**Why `idle` is the default.** The reason to hand a prompt to a topic instead of
running it here is usually that the thing in front of you is busy. A uniform draw
over every topic reproduces exactly that problem one level down: it will cheerfully
pick a topic that is mid-turn, and the second prompt queues behind the first.
`random` remains available for callers who want the literal semantics.

**`idle` never refuses.** When every candidate is busy it falls back to the full
set and reports `[Runner] note: every topic is busy — falling back to the full set`
on stderr. The caller asked for a topic; "all of them are working" is a reason to
say so, not a reason to fail.

**How busy is judged.** One `/proc` sweep answers the question for every candidate,
so all of them are judged at the same instant — a sweep per topic would give each a
different one. A **fork** topic is busy when some live `claude` carries its
deterministic session id in argv (`--resume <id>` or `--session-id <id>`, either
spelling). A **dir** topic is busy when some live `claude` is running in its
directory: there is no name-derived id to match on, so the directory is the identity.

**`random` skips the sweep entirely.** Busyness is irrelevant to it, so scanning
`/proc` would be pure cost.

**Validation:** any value other than `idle` or `random` is rejected at parse time
(`Error: invalid --pick value '<VALUE>'` / `Expected: idle or random`).

**Rejected on `broadcast`.** `clr broadcast --pick ...` exits 1 naming `clr delegate`
rather than ignoring the flag — `broadcast` has no choice to make, so accepting a
selection policy would imply one that does not exist. Verify:
`clr broadcast --pick idle x; echo $?` prints 1.

**Interaction with [`--seed`](091_seed.md):** the policy decides the candidate set,
the seed decides which member of it. `--pick idle --seed 7` and `--pick random --seed 7`
can reach different topics on the same base whenever any topic is busy, because the
sets differ.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| enum | `Pick` (`claude_topic_core::select`) | &str | Exactly `idle` or `random`; anything else is a parse error |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| — | None | `delegate`-only | [`--seed`](091_seed.md) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 16 | [`delegate`](../command/16_delegate.md) | `idle` | Chooses the candidate set the draw runs over |
| 17 | [`broadcast`](../command/17_broadcast.md) | N/A | Rejected by name — no pick to make |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 33 | [033_topic_forwarding.md](../user_story/033_topic_forwarding.md) | Developer |
