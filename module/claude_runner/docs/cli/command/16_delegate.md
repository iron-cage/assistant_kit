# CLI Command: delegate

### Description

Send one prompt to one topic instead of running it here. Enumerates the live topics under a base, picks one by policy, runs the prompt there as a print-mode `clr run` child, then relays that child's stdout, stderr, and exit code as its own — so a delegated failure is indistinguishable from a local one.

-- **Parameters:** `--pick`, `--seed`, `--dir`/`--to`, `-g`/`--global`, `--message`, `-n`/`--dry-run`
-- **Exit Codes:** the delegated child's own exit code | 1 (error before any child ran)

### Syntax

```sh
clr delegate [OPTIONS] <MESSAGE>
clr delegate [OPTIONS] --message <MESSAGE>
clr delegate [OPTIONS] -- <MESSAGE...>
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`--pick <idle\|random>`](../param/090_pick.md) | enum | `idle` | Which topic to draw — skip topics mid-turn, or draw from all of them |
| [`--seed <N>`](../param/091_seed.md) | u64 | clock+pid | Fix the draw so the same topic list always yields the same topic |
| [`--dir <PATH>`](../param/008_dir.md), `--to <PATH>` | path | CWD | Base directory to enumerate topics under |
| [`-g`/`--global`](../param/087_global.md) | flag | off | Use the global topic home as the base instead of CWD |
| `--message <TEXT>` | string | — | The prompt; also accepted as positional text, or after `--` |
| `-n`/`--dry-run` | flag | off | Print the chosen topic and the command that would run, run nothing |
| `-h`/`--help` | — | — | Print `delegate` subcommand help and exit 0 |

**Base resolution** is identical to `--topic`'s, computed by the same `claude_topic_core::identity::topic_base()`: `--dir` if given, else the global topic home if `--global`, else CWD. An explicit `--dir` outranks `--global`.

**Message assembly:** every positional token, every `--message` value, and everything after `--` are joined with single spaces in the order encountered. An all-whitespace result exits 1 — a forward with no prompt is not a forward.

**Algorithm (6 steps):**
1. Resolve the base directory.
2. Enumerate the **live** topics under it — `claude_topic_core::enumerate_live()`, which is `enumerate()` minus every topic holding zero sessions. An empty result exits 1 (see Notes).
3. Resolve the seed: `--seed` if given, else `claude_topic_core::default_seed()` (wall clock mixed with this process's pid through `SplitMix64`'s finalizer).
4. Draw one topic: under `--pick idle`, filter to topics with no turn in flight and draw `seed % len` from those, falling back to the full set when every topic is busy (reported on stderr, never a refusal); under `--pick random`, draw `seed % len` from the full set with no process scan at all.
5. When `CLR_TOPIC_LOCK` is on, take that topic's advisory run-path lock in **this** process. A topic already held elsewhere exits 1 rather than queueing.
6. Spawn `clr run --dir <base> --topic <NAME> --topic-mode <MODE> --message <TEXT>` from `current_exe()`, wait for it, relay its stdout to stdout and stderr to stderr, and exit with its exit code.

### Output Format

Normal form — one stderr note naming the chosen topic, then the child's own output verbatim:

```sh
$ clr delegate "summarize what changed today"
[Runner] delegating to 'auth-refactor' (fork)
Three commits touched the token refresh path…
```

Dry-run form — five `key: value` lines on stdout, nothing spawned:

```sh
$ clr delegate --dry-run --seed 42 "run the test suite"
base: /home/alice/project
pick: idle
seed: 42
topic: flaky-test (fork)
cmd: clr run --dir /home/alice/project --topic flaky-test --topic-mode fork --message "run the test suite"
```

The `cmd:` line is a description of what would run, not a shell-quoted line to paste — the message is shown as the single argument it is actually passed as.

### Exit Codes

| Code | Meaning |
|------|---------|
| *child's code* | A child ran; its exit code is this command's exit code |
| 0 | `--dry-run` completed, or `--help` printed |
| 1 | Error: no live topics under the base, missing message, unknown option, a flag missing its value, an invalid `--pick` value, a non-numeric `--seed`, a flag belonging to `clr broadcast`, or the chosen topic held by another run |

### Examples

```sh
# Hand this off to whichever topic is free
clr delegate "summarize what changed today"

# Reproduce yesterday's draw exactly
clr delegate --seed 42 "run the test suite"

# Ignore whether topics are busy
clr delegate --pick random "quick question"

# See where it would go without sending anything
clr delegate --dry-run "summarize what changed today"

# Delegate within the global topic home rather than this project
clr delegate --global "check the release notes"

# A prompt that starts with a hyphen
clr delegate -- --verbose means what here?

# Delegate, and act on the child's exit code
clr delegate "run the test suite" || echo "the delegated run failed"
```

### Notes

**An empty base is an error, not an empty success.** `clr topics` listing nothing is an ordinary result; `clr delegate` finding nothing is not — the prompt went nowhere, and exiting 0 would report success for work that never happened. Verify: `cd "$( mktemp -d )" && clr delegate --dry-run hi; echo $?` prints 1 and names the base.

**Live topics only.** A topic with no session has no conversation to continue — addressing it would *create* one by forking the base, and "delegate this" is not a request to mint a new conversation. The same filter is what keeps fan-out out of `-daemon/`, `-gate/`, and every `./-NNNN_*` scratch directory, all of which look exactly like dir-mode topics from the base's point of view and have no session storage. Treat that as a strong heuristic, not a guarantee: a scratch directory someone once ran `claude` inside genuinely does have storage. Verify: `clr topics` shows the full list including the ones skipped here; `clr delegate --dry-run x` shows only the live one it picked.

**The mode always travels with the name.** Every child is given `--topic NAME --topic-mode MODE`, never `--topic` alone — see [`invariant/002_mode_travels_with_name.md`](../../../../claude_topic_core/docs/invariant/002_mode_travels_with_name.md). A bare name is not a topic: when one name is held in both mechanisms, `effective_topic_mode`'s rule 4 routes a bare `--topic` to the dir-mode one every time. Verify: `clr topics` a base holding one name in both modes, then `clr delegate --dry-run x` — the `cmd:` line always carries `--topic-mode`.

**`--dir` is passed to the child, not re-derived.** The child receives the base the parent actually enumerated, because an explicit `--dir` outranks both `--global` and the inherited cwd in `topic_base`'s precedence. Without it a `--global` delegation would re-resolve in the child against its own environment.

**Why a child process rather than the daemon.** `clr chat` looks like the natural transport — the daemon already hosts sessions and already round-trips a prompt — but `Request::Spawn` starts a session in a directory and has no resume-by-session-id form. A fork-mode topic, whose entire identity *is* a session id in the base's own storage, cannot be daemon-hosted at all, so a daemon transport would silently skip most of what `clr topics` lists. The child is spawned from `current_exe()`, so it is the same binary with the same topic-resolution rules — no version skew is possible between the fan-out and what it fans out to.

**The seed is deliberately predictable.** The draw is `seed % len`, the most obvious mapping there is. A seed exists so a pick can be reproduced and asserted on, not to supply entropy; making it unpredictable would defeat the only reason to expose it. Entropy quality matters in `default_seed()` instead, which mixes clock and pid rather than returning a raw clock reading — two `clr delegate` calls from one shell loop can otherwise land in the same clock bucket and draw the same topic.

**`--pick idle` never refuses.** When every candidate is mid-turn it falls back to the full set and says so on stderr. The caller asked for a topic; "all of them are working" is a reason to report, not a reason to fail.

**Counterpart:** [`broadcast`](17_broadcast.md) is the same machinery with the pick removed — every live topic instead of one.

### Referenced Command Group

Evaluated against every existing command under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify; `delegate` opens Singleton Group 14. Nearest candidate is [`broadcast`](17_broadcast.md), which shares `parse_forward_args()` and `src/cli/forward.rs` but **not** a dispatch function: `dispatch_delegate()` and `dispatch_broadcast()` are two entries in `src/lib.rs`'s top-level match with zero cross-calls, and their parameter sets are disjoint at the ends that matter — `--pick`/`--seed` are rejected by name on `broadcast`, `--concurrency`/`-j` is rejected by name on `delegate`. Neither is reachable from the other by changing a default: no `--concurrency` value reduces `broadcast` to a single *chosen* topic, and no `--pick` value expands `delegate` to all of them. A shared argument parser is an internal module, not a shared dispatch function — the same distinction that keeps [`topics`](12_topics.md) out of `topic`'s group over `claude_topic_core`.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 17 | [`broadcast`](17_broadcast.md) | Same machinery without the pick — every live topic instead of one |
| 12 | [`topics`](12_topics.md) | Lists what `delegate` draws from; shows the zero-session topics `delegate` skips |
| 11 | [`topic`](11_topic.md) | Creates the topics `delegate` forwards to; `delegate`'s child is a `run` with `--topic` set |
| 14 | [`chat`](14_chat.md) | The other one-prompt-elsewhere command — addresses a daemon-hosted session by id rather than a topic by name |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 6 | [Running Commands](../param_group/06_running_commands.md) | Subset — `--dir` only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 33 | [033_topic_forwarding.md](../user_story/033_topic_forwarding.md) | Developer |

---

**Category:** Forwarding / delegation
**Complexity:** 6
**API Requirement:** Yes (the child runs a real Claude Code session)
**Idempotent:** No
**Risk Level:** Low (spends tokens in exactly one topic; `--dry-run` is free)
