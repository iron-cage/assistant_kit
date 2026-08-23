# Guide: Topic Sessions

Fork the current Claude Code conversation into one or more isolated "topic" sessions, each in its own hyphenated topic directory, and return to any of them later.

## Prerequisites

Each fact below was verified against the cited source before this guide was written.

| # | Prerequisite Fact | Verification Source |
|---|-------------------|---------------------|
| 1 | `clr topic` exists and is a dispatchable subcommand | `src/cli/mod.rs` — `KNOWN_SUBCOMMANDS` includes `"topic"`; `src/lib.rs` — `Some( "topic" ) => dispatch_topic( &tokens )` |
| 2 | A topic's directory is always `<base>/-<name>` — the hyphen is unconditional | `src/cli/builder.rs` — `resolve_effective_dir()`: `base.join( format!( "-{sub}" ) )` |
| 3 | `<base>` is `--dir` when given, otherwise the current working directory | `src/cli/builder.rs` — `resolve_effective_dir()`: `base_dir.unwrap_or_else( \|\| std::env::current_dir() ... )` |
| 4 | The topic directory is created on a real (non-`--dry-run`) invocation | `src/cli/builder.rs` — `if !cli.dry_run { let _ = std::fs::create_dir_all( &effective ); }` |
| 5 | First use of a topic name clones the current session into it; repeat use continues that topic's own conversation | `tests/docs/cli/command/11_topic.md` — IT-4 (`# session-transplant:` plan line) and IT-5 (`-c "`, no transplant line) |
| 6 | An explicit `--topic NAME` disables slug generation entirely | `src/cli/topic.rs` — `if cli.topic.is_some() { dispatch_run( &tokens[ 1 .. ] ); }` |
| 7 | Auto-generated slugs are disambiguated against the disk with a `-2`, `-3`, … counter | `src/cli/topic.rs` — `disambiguate_slug()` |
| 8 | Slugs are capped at 40 characters, cut back to a whole-word boundary | `src/cli/topic.rs` — `MAX_SLUG_LEN : usize = 40` |

**Placeholder Values used below** — resolve each fresh, they are not fixed constants:

| Placeholder | Meaning | Discovery command |
|-------------|---------|-------------------|
| `<topic>` | A topic name you choose | `clr topics` (lists names already taken in this base) |
| `<base>` | Directory the topic directory is created under | `pwd` (default), or the value you pass to `--dir` |

## Phase 1 — Confirm the command surface

Read-only; no State-Check Sandwich needed.

```sh
# What topic accepts, and how --topic's default differs from run/ask
clr topic help

# The six CLAUDE_* storage paths for the current directory (the clone source)
clr scope
```

## Phase 2 — Fork the current session into a named topic

State-changing: creates `<base>/-<topic>` and copies a session file into its storage.

```sh
# 1. HELP
clr topic help

# 2. BEFORE — plan mode reports what would happen, and the dir does not exist yet
clr topic --dry-run --topic <topic> "Why do two hosts spend the same refresh token?"
# expect: a path ending in /-<topic>, and a "# session-transplant:" plan line
ls -d ./-<topic> 2>/dev/null || echo "absent — as expected"

# 3. ACTION
clr topic --topic <topic> "Why do two hosts spend the same refresh token?"

# 4. AFTER — same plan command; the transplant is now gone and continuation takes over
clr topic --dry-run --topic <topic> "check"
# expect: NO "# session-transplant:" line, and `-c "` present — the topic has its own history
ls -d ./-<topic>
```

The before/after pair uses the identical `--dry-run` invocation, so the delta — transplant planned, then transplant no longer needed — is observable from the same command that performs the work (GD003 preferred variant).

## Phase 3 — Continue an existing topic

```sh
# BEFORE — confirm the topic already holds a conversation
clr topic --dry-run --topic <topic> "check"   # expect `-c "`, no transplant line

# ACTION — no re-clone; picks up where that topic left off
clr topic --topic <topic> "What did we rule out so far?"

# AFTER — still continuing, still no transplant
clr topic --dry-run --topic <topic> "check"
```

## Phase 4 — Several parallel topics from one session

Each call clones the *same* source conversation into a *different* topic, so the three lines of work stay independent.

```sh
# BEFORE
clr topics

# ACTION
clr topic --topic rt-race    "Investigate the token race"
clr topic --topic pool-drain "Investigate pool eligibility"
clr topic --topic w003-dark  "Why did w003 stop reporting?"

# AFTER — three new topics, each with its own session
clr topics
```

## Phase 5 — Auto-named topics

Omit `--topic` and the name is derived from the message.

```sh
# BEFORE — see the slug that would be generated, without creating anything
clr topic --dry-run "Investigate the flaky concurrency-gate test"
# -> ./-investigate-the-flaky-concurrency-gate   (<=40 chars, cut at a word boundary)

# ACTION
clr topic "Investigate the flaky concurrency-gate test"

# AFTER — the same message now resolves to a NEW name, not the existing topic
clr topic --dry-run "Investigate the flaky concurrency-gate test"
# -> ./-investigate-the-flaky-concurrency-gate-2
clr topics
```

**Auto-naming never resumes.** A repeated message creates a second, independent topic via the counter suffix — it does not return you to the first. Auto-naming is for one-shot spawns; if you intend to come back, name the topic explicitly with `--topic`, which is also the only form whose path you can compute from the name alone.

## Phase 6 — Throwaway topics in the global topic home

Keeps scratch work out of the project tree. `--global` resolves `<base>` to `$CLR_TOPIC_HOME` (default `$TMPDIR/clr-topic`).

```sh
# 1. HELP
clr topics help

# 2. BEFORE — deterministic path, computed without touching the disk
clr topics --path <topic> --global
ls -d "$( clr topics --path <topic> --global )" 2>/dev/null || echo "absent — as expected"

# 3. ACTION
clr topic --global --topic <topic> "Audit the refresh predicate arms"

# 4. AFTER
ls -d "$( clr topics --path <topic> --global )"
clr topics --global
```

Because `--path` is a pure computation of `<base>/-<name>` with no disk probing, the same name always yields the same absolute path — that is what makes a global topic recoverable from its name alone in a later shell.

## Phase 7 — Seed a topic from a different project

```sh
# BEFORE
clr topic --dry-run --topic port-fix --from <source-project-dir> "Port this fix over here"
# expect: a "# session-transplant:" line sourcing from <source-project-dir>

# ACTION
clr topic --topic port-fix --from <source-project-dir> "Port this fix over here"

# AFTER
clr topic --dry-run --topic port-fix "check"   # expect `-c "`, no transplant line
```

`<source-project-dir>` is any directory holding a qualifying session; discover candidates with `clr scope --dir <source-project-dir>`.

## Verification

The end goal — several independent conversations forked from one source, each resumable by name — is confirmed when all four hold:

```sh
# 1. Every topic you created is listed, each reporting its own session count
clr topics

# 2. Each topic's path is recoverable from its name alone, with no disk probing
clr topics --path <topic>

# 3. Re-entering a topic continues rather than re-clones
clr topic --dry-run --topic <topic> "check"   # `-c "` present, no transplant line

# 4. Topics are genuinely isolated — distinct storage directories, not one shared session
clr scope --dir ./-<topic>
```

Point 3 is the one that actually proves isolation worked: a transplant line reappearing after the first real call means the topic never got its own session and is silently re-cloning the source each time.

## Open Decisions

- **Where global topics live.** `$CLR_TOPIC_HOME` defaults to `$TMPDIR/clr-topic`, which on most systems is cleared on reboot. If you need global topics to survive a reboot, set `CLR_TOPIC_HOME` to a durable path yourself — this guide does not choose one for you, because the right location depends on whether you treat topics as scratch or as retained work.
- **Auto-named topic cleanup.** Nothing in `clr` removes topic directories. Whether `-*` topic dirs are pruned, committed, or gitignored is a per-project decision; they are hyphen-prefixed, so a repository following the workspace convention already ignores them.

## Related

| Type | Path | Relationship |
|------|------|--------------|
| command | [`../cli/command/11_topic.md`](../cli/command/11_topic.md) | `topic` specification — parameters, algorithm, exit codes |
| command | [`../cli/command/12_topics.md`](../cli/command/12_topics.md) | `topics` specification — listing and path resolution |
| command | [`../cli/command/09_scope.md`](../cli/command/09_scope.md) | `scope` — the six `CLAUDE_*` storage paths for a directory |
| param | [`../cli/param/028_topic.md`](../cli/param/028_topic.md) | `--topic` — the named topic directory this guide is built on |
| param | [`../cli/param/087_global.md`](../cli/param/087_global.md) | `--global` — resolves the base to `$CLR_TOPIC_HOME` |
| user story | [`../cli/user_story/030_topic_creation.md`](../cli/user_story/030_topic_creation.md) | Acceptance criteria for topic creation |
