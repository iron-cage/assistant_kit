# User Story 033: Topic Forwarding

### Scope

- **Persona**: Developer
- **Goal**: Send a prompt somewhere other than the session in front of me — to one topic that is free, or to all of them at once — without opening a terminal per topic and without having to remember which mechanism each topic uses; and get the topics to send to in one command rather than naming them one at a time.

### User Story

> As a developer with several topics in flight,
> I want to hand a prompt to one free topic, or ask every topic the same question at once,
> so that a busy main session is not a bottleneck, and a cross-cutting question does not cost me one terminal per topic.

Forwarding needs somewhere to forward *to*, and a base with no live topics is an error by
AC-11. So the story covers both halves: `pool` is the supply side (AC-14..AC-17) and
`delegate`/`broadcast` the demand side (AC-1..AC-13). The provisioning half is deliberately
anonymous — [`topic`](../command/11_topic.md) names a topic after the message that opened it,
which is the right name when the topic is *about* something and the wrong one when it is
merely somewhere for work to go.

### Acceptance Criteria

- **AC-1 (Delegate one):** `clr delegate "<MSG>"` picks one live topic under the base, runs the prompt there, and prints that run's output.
- **AC-2 (Relay the code):** `clr delegate` exits with the delegated child's own exit code — a delegated failure is indistinguishable from a local one.
- **AC-3 (Prefer free topics):** the default `--pick idle` draws only from topics with no turn in flight; when every topic is busy it falls back to the full set and says so on stderr rather than refusing.
- **AC-4 (Reproducible draw):** `--seed <N>` fixes the draw — the same seed over the same topic list always reaches the same topic.
- **AC-5 (Broadcast all):** `clr broadcast "<MSG>"` runs the prompt in every live topic under the base, one block of output per topic.
- **AC-6 (Attributed, ordered):** blocks are headed `──── <name> · <mode> · exit <code> ────` and appear in listing order, never completion order, so two runs of the same broadcast are comparable.
- **AC-7 (Bounded fan-out):** at most `--concurrency` children run at once (default 4), clamped to the topic count; the clamped value is what gets reported.
- **AC-8 (Any failure fails):** `clr broadcast` exits 0 only when every child exited 0; otherwise 1, with the failing count on stderr and the successful blocks still printed.
- **AC-9 (Live topics only):** a topic holding zero sessions is never a target — forwarding to it would start a conversation rather than continue one. `clr topics` still lists it.
- **AC-10 (Mode travels with the name):** every child is invoked with `--topic-mode` alongside `--topic`, so a name held in both mechanisms resolves to two distinct targets rather than silently collapsing to the dir-mode one.
- **AC-11 (Nothing to send to is an error):** a base with no live topics exits 1 naming the base — unlike `clr topics`, which reports an empty listing as an ordinary success.
- **AC-12 (Preview is free):** `--dry-run` prints the resolved base, the chosen topic(s), and the exact command per target, spawns nothing, and spends no tokens.
- **AC-13 (Flags stay in their lane):** `--pick`/`--seed` on `broadcast`, and `--concurrency`/`-j` on `delegate`, exit 1 naming the command each belongs to rather than being silently ignored.
- **AC-14 (Provision a pool):** `clr pool <N>` makes sure `N` anonymous topics — `t1`, `t2`, … — exist under the base, creating only the ones that are missing.
- **AC-15 (A target, not an increment):** running the same `clr pool <N>` twice creates nothing the second time, so it is safe in a script that may run again unwatched.
- **AC-16 (Counted the way forwarding counts):** the target is measured against *live* pool topics, so `clr pool --count 4 && clr broadcast "…"` reaches exactly four — a pool name whose session is gone is refilled rather than counted.
- **AC-17 (One index, one slot):** a name held in one mechanism occupies that index in both, so `--topic-mode` selects what gets created without ever putting two topics in one slot.

### Primary Flags

| Flag | Role |
|------|------|
| (none) | Forward under the current working directory as base |
| `--global` / `-g` | Use the global topic home as the base |
| `--dir <PATH>` / `--to <PATH>` | Use an explicit base; outranks `--global` |
| `--pick <idle\|random>` | *(delegate)* Which candidate set to draw from |
| `--seed <N>` | *(delegate)* Fix the draw for reproducibility |
| `--concurrency <N>` / `-j <N>` | *(broadcast, pool)* Most children in flight at once |
| `--count <N>` | *(pool)* How many pool topics must exist; also the single positional |
| `--prefix <P>` | *(pool)* Which pool to fill, and therefore what counts toward `--count` |
| `--topic-mode <fork\|dir>` | Mechanism — on every forwarded child, and for every topic `pool` creates |
| `--dry-run` / `-n` | Print the plan, run nothing |
| `--message <TEXT>`, `--` | Give the prompt explicitly, or end option parsing |

### Examples

```sh
# Get four topics to work with, then use them (AC-14)
clr pool 4 && clr broadcast "read the plan and tell me your first question"

# Run it again — creates nothing, exits 0 (AC-15)
clr pool 4

# Hand this off to whichever topic is free
clr delegate "summarize what changed today"

# Same draw as yesterday's run
clr delegate --seed 42 "run the test suite"

# Ask every topic the same question
clr broadcast "what are you working on?"

# See the whole plan first — free, spawns nothing (AC-12)
clr broadcast --dry-run "run your test suite"

# One at a time, to hold the token-spend rate down (AC-7)
clr broadcast -j 1 "run your test suite"

# Confirm the mode always travels with the name (AC-10)
clr broadcast --dry-run x | grep -c -- '--topic-mode'

# Confirm zero-session topics are listed but not targeted (AC-9)
clr topics | tail -n +2 | wc -l          # every topic
clr broadcast --dry-run x | grep -c '^cmd: '   # only the live ones

# Confirm the pool counts the way forwarding counts (AC-16)
clr pool --dry-run 4 | grep '^existing:'       # live pool topics only

# Fail the script if any topic failed (AC-8)
clr broadcast "run your test suite" || exit 1

# A prompt that starts with a hyphen
clr delegate -- --verbose means what here?
```

### Related Commands

| Command | Role |
|---------|------|
| `delegate` | Primary command — one topic, chosen by policy |
| `broadcast` | Primary command — every live topic, bounded concurrency |
| `pool` | Primary command — makes sure the topics the other two need exist |
| `topics` | Lists what both draw from, including the zero-session topics they skip |
| `topic` | The named alternative to `pool` — one topic per invocation, named after its message |
| `run` | What each child actually is — `clr run --topic <NAME> --topic-mode <MODE>` |

### Related Doc Instances

| File | Relationship |
|------|--------------|
| [`../command/16_delegate.md`](../command/16_delegate.md) | `clr delegate` command reference |
| [`../command/17_broadcast.md`](../command/17_broadcast.md) | `clr broadcast` command reference |
| [`../command/18_pool.md`](../command/18_pool.md) | `clr pool` command reference |
| [`../param/090_pick.md`](../param/090_pick.md) | `--pick` selection policy |
| [`../param/091_seed.md`](../param/091_seed.md) | `--seed` reproducible draw |
| [`../param/092_concurrency.md`](../param/092_concurrency.md) | `--concurrency` fan-out bound |
| [`../param/093_count.md`](../param/093_count.md) | `--count` as a target rather than an increment (AC-15) |
| [`../param/094_prefix.md`](../param/094_prefix.md) | `--prefix` and the rules that keep name-to-index one-to-one |
| [`../../../../claude_topic_core/docs/feature/004_topic_pool.md`](../../../../claude_topic_core/docs/feature/004_topic_pool.md) | The naming and gap-filling rules `pool` reports rather than defines |
| [`../../feature/009_topic_forwarding.md`](../../feature/009_topic_forwarding.md) | The feature this story exercises |
| [`../../../../claude_runner_core/docs/feature/007_bounded_fanout.md`](../../../../claude_runner_core/docs/feature/007_bounded_fanout.md) | The bounded fan-out both commands run on |
| [`../../../../claude_topic_core/docs/invariant/002_mode_travels_with_name.md`](../../../../claude_topic_core/docs/invariant/002_mode_travels_with_name.md) | Why `--topic-mode` accompanies every child (AC-10) |

### Related User Stories

| # | Title | Relationship |
|---|-------|--------------|
| 030 | [Topic Creation](030_topic_creation.md) | The named way to get a topic — one per invocation, named after its message; `pool` here is the anonymous, idempotent, N-at-a-time way |
| 031 | [Topic Discovery](031_topic_discovery.md) | Lists them; forwarding targets the live subset |
| 032 | [Hosted Session Chat](032_hosted_session_chat.md) | The other one-prompt-elsewhere shape — a daemon session by id, not a topic by name |
