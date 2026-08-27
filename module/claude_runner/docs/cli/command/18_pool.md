# CLI Command: pool

### Description

Make sure `N` anonymous topics exist under a base. Creates the pool-named topics that are missing — `t1`, `t2`, `t3` … — by running one print-mode `clr run` child per missing name, and creates nothing when the target is already met.

-- **Parameters:** `--count`, `--prefix`, `--topic-mode`, `--dir`/`--to`, `-g`/`--global`, `--concurrency`/`-j`, `--message`, `-n`/`--dry-run`
-- **Exit Codes:** 0 (everything missing was created, or nothing was missing) | 1 (a child failed, or an error before any child ran)

### Syntax

```sh
clr pool [OPTIONS] <N>
clr pool [OPTIONS] --count <N>
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`--count <N>`](../param/093_count.md) | usize | — | How many pool topics must exist; also accepted as the single positional |
| [`--prefix <P>`](../param/094_prefix.md) | string | `t` | Pool name prefix; may not be empty, contain `/` or a newline, start with `-`, or end in a digit |
| [`--topic-mode <fork\|dir>`](../param/088_topic_mode.md) | enum | `fork` | Mechanism for the topics created |
| [`--dir <PATH>`](../param/008_dir.md), `--to <PATH>` | path | CWD | Base directory to create the topics under |
| [`-g`/`--global`](../param/087_global.md) | flag | off | Use the global topic home as the base instead of CWD |
| [`-j`/`--concurrency <N>`](../param/092_concurrency.md) | usize | 4 | Most children in flight at once; clamped to `1..=` the number being created |
| `--message <TEXT>` | string | `ready` | Seed prompt for each new topic |
| `-n`/`--dry-run` | flag | off | Print what would be created, create nothing |
| `-h`/`--help` | — | — | Print `pool` subcommand help and exit 0 |

**Base resolution** is identical to [`delegate`](16_delegate.md)'s — same `claude_topic_core::identity::topic_base()` precedence: `--dir` if given, else the global topic home if `--global`, else CWD.

**Algorithm (6 steps):**
1. Resolve the base directory, and reject an unusable `--prefix` before touching the disk.
2. Enumerate the **live** topics under the base — `claude_topic_core::enumerate_live()`.
3. Compute the missing names — `claude_topic_core::pool::missing_names()`, which counts only pool-pattern names, fills gaps before extending the range, and treats one index as one slot across both mechanisms. An empty result exits 0.
4. Build one planned `Topic` per missing name in the selected mode. `Topic::path` is a computed path in both mechanisms, so a not-yet-created topic goes through the same lock and spawn path as an existing one.
5. When `CLR_TOPIC_LOCK` is on, take each planned topic's advisory run-path lock in **this** process. A name already held elsewhere is dropped with a stderr note; every name held elsewhere exits 1.
6. Run `clr run --dir <base> --topic <NAME> --topic-mode <MODE> --message <TEXT>` for each remaining name through `claude_runner_core::fanout::run_bounded()`, then report per name and exit 0 only when every child exited 0.

### Output Format

Normal form — one stderr line stating the plan, then one line per name on stdout:

```sh
$ clr pool 4
[Runner] creating 4 pool topic(s) under /home/alice/project — each is a full Claude Code session, 4 at a time
created: t1 (fork)
created: t2 (fork)
created: t3 (fork)
created: t4 (fork)
```

A second run of the same command creates nothing and says so on stderr, leaving stdout empty:

```sh
$ clr pool 4
[Runner] /home/alice/project already holds 4 topic(s) with prefix 't' — nothing to create
```

Dry-run form — six `key: value` lines plus one `cmd:` line per name, nothing spawned:

```sh
$ clr pool --dry-run 4
base: /home/alice/project
prefix: t
mode: fork
target: 4
existing: 1
create: 3
concurrency: 3
cmd: clr run --dir /home/alice/project --topic t2 --topic-mode fork --message "ready"
cmd: clr run --dir /home/alice/project --topic t3 --topic-mode fork --message "ready"
cmd: clr run --dir /home/alice/project --topic t4 --topic-mode fork --message "ready"
```

`existing` counts pool-named topics only, and the reported `concurrency` is the clamped value actually used.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Every missing topic was created; or nothing was missing; or `--dry-run` completed; or `--help` printed |
| 1 | A child failed (count named on stderr); or a missing or non-numeric count, an unusable `--prefix`, an invalid `--topic-mode`, a non-numeric `--concurrency`, an unknown option, a flag missing its value, a second positional argument, unresolvable session storage, or every planned name held by another run |

### Examples

```sh
# See what four topics would cost — free, spawns nothing
clr pool --dry-run 4

# Create them
clr pool 4

# Again — creates nothing, exits 0
clr pool 4

# A named pool: worker1 … worker8
clr pool --prefix worker 8

# Legacy directory topics instead of fork topics
clr pool --topic-mode dir --count 3

# One at a time, to keep the token-spend rate down
clr pool -j 1 --count 6

# Provision, then use
clr pool 4 && clr broadcast "read the plan and tell me your first question"

# Count what a pool would create, without creating it
clr pool --dry-run 10 | grep -c '^cmd: '
```

### Notes

**`--count` is a target, never an increment.** "Make sure four exist", not "add four more" — which is the only form usable from a script that may run twice, and the second run is the one nobody is watching. Verify: `clr pool --dry-run 4`, then `clr pool 4`, then `clr pool --dry-run 4` again — the third command reports `create: 0`.

**The target is counted against live topics, not all topics.** A pool name whose session file was deleted is *missing*, and gets refilled. Counting against the full set instead would let `clr pool --count 4` report success while [`broadcast`](17_broadcast.md) reached only three of the four — a partial fan-out that looks complete, which is the failure this command family exists to prevent. Verify: delete one pool topic's session file, then `clr pool --dry-run 4` names it as the one to create.

**Only pool-pattern names count.** A base holding ten richly-named topics has zero pool topics, and asking for four gets four. Anything else would make the meaning of `N` depend on unrelated work that happens to share the directory. `t01` is not a pool name either: a leading zero does not round-trip through `format!( "{prefix}{index}" )`, and admitting it would make the mapping many-to-one.

**Gaps are filled before the range is extended.** With `t1` and `t3` present, a target of four creates `t2` and `t4`, not `t4` and `t5`. A pool is a set of slots, and a deleted topic leaves a slot rather than a permanent hole.

**One index is one slot, across both mechanisms.** A `t1` held in fork mode and a `t1` held in dir mode are two topics ([`claude_topic_core/docs/feature/002`](../../../../claude_topic_core/docs/feature/002_topic_enumeration.md)) but one slot, so `clr pool --topic-mode dir --count 2` on a base holding a fork-mode `t1` creates only `t2`. Creating both would put two topics in one slot and make the pool's own count ambiguous.

**Creating a topic means running one.** There is no way to make a topic exist without a session in it, and no way to make a session without invoking Claude Code — so this is the one command in the CLI whose non-dry-run path costs money by construction, one real turn per topic created. `--dry-run` prints the entire plan for free and is the first example in `--help` for that reason.

**The seed message is deliberately trivial.** Its only job is to make the session exist; the topic's first real instruction arrives later through [`delegate`](16_delegate.md) or [`broadcast`](17_broadcast.md). A long seed prompt would be paid for once per topic and then be irrelevant to every turn after it. `--message` overrides it when the pool needs a shared briefing.

**Why anonymous names at all.** [`topic`](11_topic.md) names a topic after the message that opened it — descriptive, disambiguated by a counter, and meaningful to read back. That is the right name when the topic is *about* something. A pool topic is not about anything: it is somewhere for work to go, and naming it after its first message would be actively misleading, since the second message is unlikely to be about the same thing.

**`--count 0` is a no-op, not an error.** `clr pool "$N"` from a script that computed `N == 0` has asked for nothing, and failing there would force the caller to special-case a value that already means "do nothing".

**A stray second positional is rejected, not joined.** `clr pool 2 hello` exits 1 and suggests `--message "hello"`. `pool` takes a number; silently reading a stray word as prose would hide the typo that produced it — the opposite of [`delegate`](16_delegate.md)/[`broadcast`](17_broadcast.md), where every positional *is* message text.

### Referenced Command Group

Evaluated against every existing command under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify; `pool` opens Singleton Group 16. Nearest candidates are [`broadcast`](17_broadcast.md) and [`topic`](11_topic.md), and it fails the test against both for different reasons: `dispatch_pool()` is its own entry in `src/lib.rs`'s top-level match with zero cross-calls to `dispatch_broadcast()`, and its parameter set is disjoint at the ends that matter — `--count`/`--prefix` exist nowhere else, and `pool`'s single positional is a number where `broadcast`'s positionals are message text. Against `topic` the divergence is the naming rule itself: `topic` derives a name from the message, `pool` derives it from an index, and no value of any flag turns one into the other. `pool` shares `child_command()`/`claim_locks()`/`run_bounded()` with the forwarding pair, but a shared helper is an internal module, not a shared dispatch function — the same distinction that keeps [`topics`](12_topics.md) out of `topic`'s group over `claude_topic_core`.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 17 | [`broadcast`](17_broadcast.md) | The command `pool` provisions for — same transport, same concurrency bound, applied to names that exist |
| 16 | [`delegate`](16_delegate.md) | Draws from the pool `pool` fills |
| 11 | [`topic`](11_topic.md) | The other way to create a topic — named after its message rather than after an index |
| 12 | [`topics`](12_topics.md) | Shows what `pool` created, and the session counts it counted |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 6 | [Running Commands](../param_group/06_running_commands.md) | Subset — `--dir` only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 33 | [033_topic_forwarding.md](../user_story/033_topic_forwarding.md) | Developer |

---

**Category:** Forwarding / provisioning
**Complexity:** 6
**API Requirement:** Yes (one real Claude Code session per topic created)
**Idempotent:** Yes (the target is a target — a met target creates nothing)
**Risk Level:** Medium (spends one turn per topic created; `--dry-run` is free and shows the full plan)
