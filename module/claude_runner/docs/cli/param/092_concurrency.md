# CLI Parameter: --concurrency

Bounds how many `clr run` children [`broadcast`](../command/17_broadcast.md) and
[`pool`](../command/18_pool.md) have in flight at once.

- **Type:** usize (positive integer)
- **Default:** `4`
- **Command:** [`broadcast`](../command/17_broadcast.md), [`pool`](../command/18_pool.md)
- **Group:** None — the fan-out commands' shared surface
- **JSON Key:** none (forwarding commands take no JSON config)

```sh
clr broadcast "task"                       # default: 4 at a time
clr broadcast -j 1 "task"                  # strictly serial
clr broadcast --concurrency 8 "task"       # 8 at a time, if there are 8 topics
clr broadcast --dry-run -j 50 "task"       # preview: 'concurrency:' shows the clamped value
clr pool -j 1 --count 6                    # provision six topics one at a time
```

**This is a token-spend rate, not a scheduling detail.** Every child is a full
Claude Code session, so `-j` sets how many API conversations run at once — an
exposure to rate limits and to cost, not just to CPU. The default of 4 is high
enough for the parallelism to be the point and low enough that a twenty-topic base
does not hit the API twenty-wide. Lower it when a broadcast is expensive; raise it
when the topics are cheap and the wall clock matters.

**Clamped to `1..=` the number of children.** A bound above the number of topics
(for `broadcast`) or of names being created (for `pool`) can never be reached, and a
bound of 0 would mean "run nothing", which is not what asking for a fan-out means.
Both are silently corrected, and the corrected value is what `--dry-run` reports and
what the `[Runner] broadcasting …` / `[Runner] creating …` line names. Verify:
`clr broadcast --dry-run -j 50 x | grep '^concurrency:'` reports the topic count, not
50; `clr pool --dry-run -j 50 --count 2 | grep '^concurrency:'` reports 2.

**Ordering does not depend on it.** Results come back in listing order regardless of
completion order or worker count, so `-j 1` and `-j 8` print the same blocks in the
same sequence — only the wall clock differs. That is what makes two runs of the same
broadcast directly comparable. Verify: `diff <( clr broadcast --dry-run -j 1 x ) <( clr broadcast --dry-run -j 9 x )`
differs only on the `concurrency:` line.

**Validation:** a value that does not parse as `usize` is rejected at parse time
(`Error: --concurrency must be a positive integer, got '<VALUE>'`). Negative values
fail this way too — there is no signed form.

**Rejected on `delegate`.** `clr delegate -j 2 x` exits 1 naming `clr broadcast`
rather than ignoring the flag — `delegate` runs exactly one child, so a concurrency
bound for it would be inert configuration that reads as if it did something. Verify:
`clr delegate -j 2 x; echo $?` prints 1.

**No timeout accompanies it.** The bound limits how many children run, never how
long one may take: killing Claude Code mid-turn can leave a session file partly
written, corrupting the topic the broadcast was meant to advance. A deadline belongs
on the individual run — the child is an ordinary `clr run`, so its own `--timeout`
applies. See [`claude_runner_core/docs/feature/007_bounded_fanout.md`](../../../../claude_runner_core/docs/feature/007_bounded_fanout.md).

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| integer | usize | usize | Parses as a non-negative integer; clamped to `1..=topic count` before use |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| — | None | `broadcast`-only | — |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 17 | [`broadcast`](../command/17_broadcast.md) | `4` | Workers in `run_bounded`; clamped to the topic count |
| 18 | [`pool`](../command/18_pool.md) | `4` | Same workers, clamped to the number of names being created |
| 16 | [`delegate`](../command/16_delegate.md) | N/A | Rejected by name — exactly one child, nothing to bound |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 33 | [033_topic_forwarding.md](../user_story/033_topic_forwarding.md) | Developer |
