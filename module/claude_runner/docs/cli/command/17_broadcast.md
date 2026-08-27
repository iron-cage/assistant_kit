# CLI Command: broadcast

### Description

Send one prompt to every live topic under a base. Runs each as a print-mode `clr run` child, at most `--concurrency` at a time, and prints one attributed block per topic in listing order. Exits 0 only when every child exited 0.

-- **Parameters:** `--concurrency`/`-j`, `--dir`/`--to`, `-g`/`--global`, `--message`, `-n`/`--dry-run`
-- **Exit Codes:** 0 (every child succeeded) | 1 (any child failed, or an error before any child ran)

### Syntax

```sh
clr broadcast [OPTIONS] <MESSAGE>
clr broadcast [OPTIONS] --message <MESSAGE>
clr broadcast [OPTIONS] -- <MESSAGE...>
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`-j`/`--concurrency <N>`](../param/092_concurrency.md) | usize | 4 | Most children in flight at once; clamped to `1..=topic count` |
| [`--dir <PATH>`](../param/008_dir.md), `--to <PATH>` | path | CWD | Base directory to enumerate topics under |
| [`-g`/`--global`](../param/087_global.md) | flag | off | Use the global topic home as the base instead of CWD |
| `--message <TEXT>` | string | — | The prompt; also accepted as positional text, or after `--` |
| `-n`/`--dry-run` | flag | off | Print the topic list and the commands that would run, run nothing |
| `-h`/`--help` | — | — | Print `broadcast` subcommand help and exit 0 |

**Base resolution** and **message assembly** are identical to [`delegate`](16_delegate.md)'s — same `topic_base()` precedence, same join rule, same empty-message rejection.

**Algorithm (6 steps):**
1. Resolve the base directory.
2. Enumerate the **live** topics under it — `claude_topic_core::enumerate_live()`, sorted by name then mode. An empty result exits 1.
3. When `CLR_TOPIC_LOCK` is on, take each topic's advisory run-path lock in **this** process. A topic already held elsewhere is dropped from the batch with a stderr note, never waited on; every topic held elsewhere exits 1.
4. Build one `clr run --dir <base> --topic <NAME> --topic-mode <MODE> --message <TEXT>` command per remaining topic, from `current_exe()`.
5. Run them through `claude_runner_core::fanout::run_bounded()` with `--concurrency` workers — results come back in input order regardless of completion order.
6. Print one block per topic, then exit 0 if every child exited 0, else 1 after a stderr count.

### Output Format

Normal form — one stderr line stating the plan, then one `────` header per topic followed by that child's stdout:

```sh
$ clr broadcast "what are you working on?"
[Runner] broadcasting to 3 topic(s), 3 at a time
──── auth-refactor · fork · exit 0 ────
Rewriting the token refresh path.
──── docs · fork · exit 0 ────
Second pass over the CLI reference.
──── bench · dir · exit 0 ────
Nothing yet — waiting on the fixture.
```

Blocks come back in **listing order, never completion order**, so two runs of the same broadcast are directly comparable. Each child's stderr is relayed to stderr, interleaved after its block.

Dry-run form — three `key: value` lines plus one `cmd:` line per topic, nothing spawned:

```sh
$ clr broadcast --dry-run "status?"
base: /home/alice/project
topics: 3
concurrency: 3
cmd: clr run --dir /home/alice/project --topic auth-refactor --topic-mode fork --message "status?"
cmd: clr run --dir /home/alice/project --topic bench --topic-mode dir --message "status?"
cmd: clr run --dir /home/alice/project --topic docs --topic-mode fork --message "status?"
```

The reported `concurrency` is the clamped value actually used — `-j 50` over 3 topics reports 3.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Every child exited 0; or `--dry-run` completed; or `--help` printed |
| 1 | Any child exited non-zero (count named on stderr); or no live topics under the base, missing message, unknown option, a flag missing its value, a non-numeric `--concurrency`, a flag belonging to `clr delegate`, or every topic held by another run |

### Examples

```sh
# Ask every topic the same question
clr broadcast "what are you working on?"

# One at a time, to keep the token-spend rate down
clr broadcast -j 1 "run your test suite"

# See the full plan first — free, spawns nothing
clr broadcast --dry-run "run your test suite"

# Broadcast across the global topic home rather than this project
clr broadcast --global "any blockers?"

# A prompt that starts with a hyphen
clr broadcast -- --dry-run is just text here

# Fail the script if any topic failed
clr broadcast "run your test suite" || exit 1

# Count the topics a broadcast would reach
clr broadcast --dry-run x | grep -c '^cmd: '
```

### Notes

**The concurrency bound is a token-spend rate, not a scheduling detail.** Every child is a full Claude Code session, so `-j` sets how many API conversations run at once — an exposure to rate limits and cost, not just to CPU. The default of 4 is high enough for the parallelism to be the point and low enough that a twenty-topic base does not hit the API twenty-wide. Verify the clamp: `clr broadcast --dry-run -j 50 x | grep '^concurrency:'` reports the topic count, not 50.

**One failing topic is a failing broadcast.** A partial fan-out that reports success is indistinguishable from a complete one, so any non-zero child makes the whole command exit 1, with `[Runner] N of M topic(s) failed` on stderr. The successful blocks are still printed — the failure is reported, not substituted for the output.

**There is no timeout here.** `run_bounded` never kills a child: interrupting Claude Code mid-turn can leave a session file partly written, which corrupts the very topic the broadcast was meant to advance. A deadline belongs on the individual run instead — the child is an ordinary `clr run`, so its own `--timeout` applies. See [`claude_runner_core/docs/feature/007_bounded_fanout.md`](../../../../claude_runner_core/docs/feature/007_bounded_fanout.md).

**Output is buffered per child, not streamed.** Each child's stdout and stderr are drained concurrently and printed whole once it exits — which is what makes attributed blocks possible at all, and what keeps a child that writes more than a pipe buffer's worth from deadlocking. Nothing appears for a given topic until that topic is done.

**Locks are held by the parent, for the whole batch.** A child cannot decline a topic a sibling in the same batch already took, and only the parent knows the whole target set. `try_lock` never blocks: a topic held by an unrelated run is skipped with a note rather than waited on, because a fan-out that stalls on one busy topic has become a serial run with extra steps. An unusable lock directory warns and proceeds unlocked — the lock is advisory, and it must not turn a working fan-out into a failing one.

**Blocks are attributed by position, not by name.** `run_bounded` preserves input order, so outcomes are zipped with the target list. Keying by name would be wrong: the same name can legitimately be held in both mechanisms, so a name is not a unique key. Verify: a base holding one name in both modes broadcasts to two topics, and `clr broadcast --dry-run x | grep -c '^cmd: '` reports 2.

**Live topics only, and the mode always travels with the name** — identical to [`delegate`](16_delegate.md); see that command's Notes for both rules and how to verify them.

**Counterpart:** [`delegate`](16_delegate.md) is the same machinery with a pick in front of it — one topic instead of every one.

### Referenced Command Group

Evaluated against every existing command under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify; `broadcast` opens Singleton Group 15. See [`delegate`](16_delegate.md#referenced-command-group) for the worked test on the nearest candidate pair (`delegate` / `broadcast`): shared parser module, two distinct dispatch functions with zero cross-calls, and parameter sets that reject each other's flags by name rather than defaulting them differently.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 16 | [`delegate`](16_delegate.md) | Same machinery with a pick in front — one topic instead of every one |
| 12 | [`topics`](12_topics.md) | Lists what `broadcast` targets; shows the zero-session topics it skips |
| 15 | [`sessions`](15_sessions.md) | Also a fan-wide view of concurrent work, keyed by daemon-hosted session rather than by topic |
| 6 | [`ps`](06_ps.md) | Shows the children a broadcast has in flight, as ordinary running `claude` processes |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 6 | [Running Commands](../param_group/06_running_commands.md) | Subset — `--dir` only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 33 | [033_topic_forwarding.md](../user_story/033_topic_forwarding.md) | Developer |

---

**Category:** Forwarding / fan-out
**Complexity:** 7
**API Requirement:** Yes (one real Claude Code session per topic)
**Idempotent:** No
**Risk Level:** Medium (spends tokens in every live topic at once; `--dry-run` is free and shows the full plan)
