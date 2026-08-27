# Feature: Topic Forwarding

### Scope

- **Purpose**: Let one prompt be sent somewhere other than the session in front of the user — to one topic chosen by policy (`clr delegate`), or to every live topic at once (`clr broadcast`) — without a terminal per topic and without the caller having to know which mechanism each topic uses; and let the topics being addressed be provisioned in one idempotent command (`clr pool`) rather than named one at a time.
- **In Scope**: The transport all three commands use, why it is a child process rather than the daemon, target selection and the live-topic filter, how the mode is carried to each child, lock ownership across a batch, result attribution and exit-code aggregation, and the target-not-increment provisioning that keeps the addressable set at a known size.
- **Out of Scope**: What a topic *is*, how names resolve, and the pool naming rules themselves (→ `claude_topic_core/docs/`, in particular `feature/004_topic_pool.md`), the concurrency primitive itself (→ `claude_runner_core/docs/feature/007_bounded_fanout.md`), creating a *named* topic (→ `docs/cli/command/11_topic.md`), listing them (→ `docs/cli/command/12_topics.md`), and reaching a daemon-hosted session by id (→ `docs/cli/command/14_chat.md`).

### Why This Exists

A developer with several topics in flight has no way to use them together.

Each topic is reachable only by making it the current one: `cd` to its directory, or pass
`--topic NAME --topic-mode MODE` and wait for the answer. So "ask whichever one is free" costs
a manual check of which are busy, and "ask all of them" costs one terminal per topic and one
prompt typed per terminal. Both are exactly the mechanical work the topics were created to
avoid.

The narrower framing — "add a fan-out command" — undersells it. The reason topics exist is
that a single conversation is a bottleneck; without forwarding, the *set* of topics is a
bottleneck too, because reaching it is serial by hand.

And the same argument applies one step earlier. Forwarding to a set of topics is only useful
if the set exists, and the only way to build one was `clr topic` — one invocation per topic,
each needing a name the user had to invent. For topics that exist to *hold work* rather than
to be *about* something, that naming step is pure friction, and doing it four times to get
four workers is the serial-by-hand problem again in a different place. Hence `clr pool`.

### The Two Forwarding Commands

Both answer the same question — *which topics, and then what* — and differ only in the answer's
size. That is why they share one module (`src/cli/forward.rs`) and one argument parser.

| | `delegate` | `broadcast` |
|---|---|---|
| Targets | exactly one, chosen by `--pick` | every live topic |
| Own flags | `--pick`, `--seed` | `--concurrency` / `-j` |
| Concurrency | 1 | `--concurrency`, clamped to the topic count |
| Output | the child's own, verbatim | one attributed block per topic |
| Exit code | the child's own | 0 only if every child exited 0 |

Everything else is shared: base resolution (`topic_base()`'s `--dir` > `--global` > CWD), the
live-topic filter, message assembly, `--dry-run`, and the child's shape.

### Why a Child Process, Not the Daemon

`clr chat` is the obvious transport — the daemon already hosts sessions and already round-trips
a prompt. It cannot be used.

`Request::Spawn` starts a session **in a directory**. It has no resume-by-session-id form. A
fork-mode topic's entire identity *is* a session id in the base's own storage, with no
directory of its own — so the daemon cannot host one at all. A daemon transport would therefore
reach only dir-mode topics and silently skip most of what `clr topics` lists: a fan-out that
looks completely successful while missing the majority of its targets.

So each target gets a print-mode `clr run` child, spawned from `current_exe()`. Same binary,
therefore the same topic-resolution rules — no version skew is possible between the fan-out and
what it fans out to.

```text
clr run --dir <resolved base> --topic <NAME> --topic-mode <fork|dir> --message <TEXT>
```

`--dir` carries the base the parent actually enumerated, rather than being re-derived in the
child from `--global` or the inherited cwd. An explicit `--dir` outranks both in `topic_base()`'s
precedence, so the child lands where the parent looked whatever its own environment says.

### The Mode Always Travels With the Name

Every child gets `--topic-mode` alongside `--topic`, never `--topic` alone.

A bare name is not a topic (`claude_topic_core/docs/invariant/002_mode_travels_with_name.md`).
When one name is held in both mechanisms, `effective_topic_mode`'s rule 4 — an existing
`<base>/-<name>` directory outranks fork mode — routes a bare `--topic` to the dir-mode one
every single time. The fork-mode topic sitting right beside it in the same enumeration is then
never reached, and nothing reports that: the broadcast prints a block for the name, the exit
code is 0, and half the targets never saw the prompt.

The same reasoning is why results are attributed **by position**, not by name. `run_bounded`
preserves input order, so outcomes are zipped with the target list. A name is not a unique key
here — two topics can legitimately share one.

### Live Topics Only

Both commands enumerate with `enumerate_live()`, so a topic holding zero sessions is not a
target. This does two jobs:

1. **It is the difference between continuing a conversation and starting one.** A topic with no
   session has nothing to continue — addressing it would *create* a conversation by forking the
   base. "Send this to my topics" is not a request to mint new ones.
2. **It keeps fan-out out of non-topic directories.** `topic_name_of` accepts any `-`-prefixed
   directory name, and this workspace's own convention marks generated and ignored directories
   the same way — `-daemon/`, `-gate/`, and every `./-NNNN_*` scratch directory look exactly
   like dir-mode topics from the base's point of view. They have no session storage, so the
   filter drops them. A strong heuristic, not a guarantee: a scratch directory someone once ran
   `claude` inside genuinely does have storage.

An empty result is an **error** here, exit 1 — unlike `clr topics`, where listing nothing is an
ordinary success. A forward with no target did not do what was asked, and exiting 0 would
report success for a prompt that went nowhere.

### Provisioning the Targets

`clr pool <N>` makes sure `N` anonymous topics — `t1`, `t2`, … — exist under the base. It is the
third command in this feature because it is the same machinery pointed the other way: where
`broadcast` addresses every name the live filter returns, `pool` addresses exactly the names
that filter *did not* return, and it reaches them through the identical transport — a
print-mode `clr run` child per name, through `run_bounded`, with `--topic-mode` beside every
`--topic`.

That reuse goes further than it looks. `Topic::path` is a *computed* path in both mechanisms,
so a `Topic` value is meaningful before the topic exists. `pool` builds one planned `Topic` per
missing name and hands it to the same `child_command()`, `describe_child()`, and `claim_locks()`
the forwarding pair use — no parallel "not yet created" code path, and therefore no way for the
two to drift on how a name becomes a command line.

**`--count` is a target, not an increment.** "Make sure four exist", never "add four more". This
is the whole difference between a command that is safe in a script that runs twice and one that
is not, and the second run is the one nobody is watching: an implementation that appended `N`
names would look perfect on a fresh base and double the pool silently every rerun.

**The target is counted against `enumerate_live()`, the same set `broadcast` targets.** This is
the load-bearing decision. Counting against `enumerate()` instead would let a pool name whose
session file was deleted count as present forever — so `clr pool --count 4` would report success
while `clr broadcast` reached three. That is a partial fan-out that looks complete, which is
precisely the failure mode this whole feature is built to prevent; it would be perverse for the
provisioning half to introduce it. Two filters therefore apply before the target is compared:
only names matching `format!( "{prefix}{index}" )` count, and only those holding a session.

**Gaps are filled before the range is extended**, and **one index is one slot across both
mechanisms** — a fork-mode `t1` occupies dir mode's `t1` too. A pool is a set of slots; a
deleted topic leaves a slot rather than a permanent hole, and two topics in one slot would make
the pool's own count ambiguous. Both rules live in `claude_topic_core::pool::missing_names()`,
which `pool` reports rather than defines.

**Creating a topic means running one.** There is no way to make a topic exist without a session
in it, and no way to make a session without invoking Claude Code — so this is the one command in
the CLI whose non-dry-run path costs money by construction, one real turn per topic created.
The seed message (`--message`, default `ready`) is deliberately trivial: its only job is to make
the session exist, and a long seed prompt would be paid for once per topic and then be
irrelevant to every turn after it. `--dry-run` prints the entire plan for free, and is the first
example in `clr pool --help` for that reason.

**Why anonymous names at all.** `clr topic` names a topic after the message that opened it —
descriptive, disambiguated by a counter, meaningful to read back. That is the right name when
the topic is *about* something. A pool topic is not about anything; it is somewhere for work to
go, and naming it after its first message would be actively misleading, since the second message
is unlikely to be about the same thing. The two commands are not variants of each other: no flag
value turns the message-derived naming rule into the index-derived one.

### Locks Are Held by the Parent

When `CLR_TOPIC_LOCK` is on, the parent takes every topic's advisory run-path lock and holds it
for the whole batch. This is true of `pool` too — the planned topics it is about to create are
locked exactly like existing ones. Two concurrent `clr pool` runs can both compute the same
missing set, since the count is read before the locks are taken; what they cannot do is both
*create* an index, because the second run's `try_lock` on that name fails and it skips the name
with a stderr note. A run whose every planned name is held elsewhere exits 1 rather than
reporting that it provisioned a pool it did not.

Two reasons it cannot be pushed into the children. `run_bounded` takes built `Command`s, not
closures, so there is no place inside a worker to hold a guard. And only the parent knows the
whole target set — a child cannot decline a topic that a sibling in the same batch already took.

`try_lock` never blocks. A topic held by an unrelated run is skipped with a stderr note rather
than waited on: a fan-out that stalls on one busy topic has become a serial run with extra
steps. An unusable lock directory warns and proceeds unlocked — the lock is advisory, and an
infrastructure problem must not turn a working fan-out into a failing one.

### Reporting

`delegate` relays the child's stdout, stderr, and exit code as its own, so a delegated failure
is indistinguishable from a local one — which is the point of delegating rather than reporting
on it.

`broadcast` prints one block per topic, headed `──── <name> · <mode> · exit <code> ────`.
Without attribution, twenty answers concatenated are one answer to a question nobody asked.
Blocks appear in **listing order, never completion order**, so two runs of the same broadcast
are directly comparable. It exits 0 only when every child exited 0: a partial fan-out that
reports success is indistinguishable from a complete one.

`pool` reports successes by **name only** — `created: t2 (fork)` — and does not print the
child's answer. The seed answer is throwaway by construction, so printing four of them would
bury the one thing worth reading: which names now exist. A *failure* does get its stderr
relayed, because that is not throwaway. Aggregation matches `broadcast`: exit 0 only when every
child exited 0, with the failing count on stderr.

### Deliberate Omissions

**No timeout.** `run_bounded` never kills a child. Interrupting Claude Code mid-turn can leave a
session file partly written — corrupting the very topic the broadcast was meant to advance. A
deadline belongs on the individual run, where the child is an ordinary `clr run` and its own
`--timeout` applies.

**No streaming.** Each child's output is buffered and printed whole once it exits. That is what
makes attributed blocks possible at all, and what keeps a child writing more than a pipe
buffer's worth from deadlocking.

**No `--pick` on `broadcast`, no `-j` on `delegate`.** Each is rejected by name, naming the
command it belongs to. `broadcast` has no choice to make, and `delegate` runs exactly one
child; accepting either flag would imply a behavior that does not exist.

### Verification

```sh
# The plan, without spending anything
clr broadcast --dry-run "status?"

# Every child carries its mode (the invariant above)
clr broadcast --dry-run x | grep -c -- '--topic-mode'

# Listed topics vs. targeted topics — the live filter, made visible
clr topics | tail -n +2 | wc -l
clr broadcast --dry-run x | grep -c '^cmd: '

# A fixed seed reaches a fixed topic
clr delegate --dry-run --seed 5 x | grep '^topic:'
clr delegate --dry-run --seed 5 x | grep '^topic:'

# Nowhere to send is an error, not an empty success
cd "$( mktemp -d )" && clr delegate --dry-run hi; echo $?   # 1

# The pool plan, for free
clr pool --dry-run 4

# A target, not an increment — the third command reports create: 0
clr pool --dry-run 4 | grep '^create:'
clr pool 4
clr pool --dry-run 4 | grep '^create:'

# Provision and use, in one line
clr pool 4 && clr broadcast "read the plan and tell me your first question"
```

Automated coverage: `tests/forward_command_test.rs` (16 cases, fw01–fw16) exercises selection,
reproducibility, the live filter, the mode invariant, both same-name-across-modes halves, the
concurrency clamp, cross-command flag rejection, and the empty-base and missing-message guards.
`tests/pool_command_test.rs` (18 cases, pl01–pl18) exercises the target computation and its two
filters, gap-filling, the one-index-one-slot rule, the prefix guards, and the argument guards —
with pl06 as the regression guard for counting against the live set rather than the full one.
Every case in both suites runs through `--dry-run`, so neither ever spawns Claude Code.

### Cross-References

| Doc | Relationship |
|-----|--------------|
| [`../cli/command/16_delegate.md`](../cli/command/16_delegate.md) | `clr delegate` command reference |
| [`../cli/command/17_broadcast.md`](../cli/command/17_broadcast.md) | `clr broadcast` command reference |
| [`../cli/command/18_pool.md`](../cli/command/18_pool.md) | `clr pool` command reference |
| [`../cli/param/090_pick.md`](../cli/param/090_pick.md) | `--pick` selection policy |
| [`../cli/param/091_seed.md`](../cli/param/091_seed.md) | `--seed` reproducible draw |
| [`../cli/param/092_concurrency.md`](../cli/param/092_concurrency.md) | `--concurrency` fan-out bound |
| [`../cli/param/093_count.md`](../cli/param/093_count.md) | `--count` as a target rather than an increment |
| [`../cli/param/094_prefix.md`](../cli/param/094_prefix.md) | `--prefix` and the rules that keep name-to-index one-to-one |
| [`../../../claude_topic_core/docs/feature/004_topic_pool.md`](../../../claude_topic_core/docs/feature/004_topic_pool.md) | The pool naming and gap-filling rules `pool` reports rather than defines |
| [`../cli/user_story/033_topic_forwarding.md`](../cli/user_story/033_topic_forwarding.md) | The user story, with acceptance criteria |
| [`../../../claude_runner_core/docs/feature/007_bounded_fanout.md`](../../../claude_runner_core/docs/feature/007_bounded_fanout.md) | The concurrency primitive both commands run on |
| [`../../../claude_topic_core/docs/invariant/002_mode_travels_with_name.md`](../../../claude_topic_core/docs/invariant/002_mode_travels_with_name.md) | Why `--topic-mode` accompanies every child |
| [`../../../claude_topic_core/docs/invariant/001_registry_non_authoritative.md`](../../../claude_topic_core/docs/invariant/001_registry_non_authoritative.md) | Why a missing registry entry hides a topic without breaking it |
| [`008_interactive_handoff.md`](008_interactive_handoff.md) | The other "reach a conversation from elsewhere" feature — by daemon handoff rather than by name |
