# agent_kit — Vision

Claude Code is the most capable AI coding assistant available. It also has no public API,
no programmatic interface, and no way to integrate it into automated workflows without
hand-rolling file parsing and shell invocations from scratch. Every team that uses it
heavily ends up writing the same glue code — or worse, not writing it and just clicking
through a terminal manually.

`agent_kit` changes that.

## The Problem Space

When you work with Claude Code seriously, you run into three distinct friction points:

**1. Account rotation is manual and fragile.**
Each Claude subscription has a 5-hour active-use window. If you run multiple subscriptions
(work, personal, team) to maintain uninterrupted availability, switching between them today
means manually editing `~/.claude/.credentials.json` — the kind of operation that, if done
wrong, logs you out entirely. One transient write failure and your session is gone. There
has to be a better way.

**2. Session lifecycle is opaque.**
Claude Code stores all conversation history in `~/.claude/projects/`, keyed by a
non-obvious escaped path. Should a new run resume an existing conversation or start fresh?
Where exactly does the history for `/home/user/project` live? Right now answering these
questions requires digging through undocumented directory structures. Programmatic code
that needs to decide "continue or restart?" has nothing to call.

**3. Automation requires hand-rolled glue.**
Any tool that wants to invoke Claude Code — a CI hook, a code review bot, a scheduled
analysis job — needs to know which flags to pass, how to set the right environment, how to
handle interactive vs non-interactive modes, how to wire stderr vs stdout. There is no
documented calling convention beyond reading the binary's help text.

## What We Built

`agent_kit` is a layered Rust workspace that solves each of these:

```
claude_profile      Account lifecycle management + full ~/.claude/ topology
claude_runner_core  Type-safe builder pattern for Claude Code invocations
claude_runner       CLI adapter — human interface over the builder
claude_storage_core Zero-dep JSONL parser for conversation history
claude_storage      CLI for exploring and analysing Claude's session storage
```

**Account rotation is now two lines:**
```rust
claude_profile::account::switch_account("work")?;
// ~/.credentials.json atomically replaced, _active marker updated
```

**Continuation detection is a function call:**
```rust
if claude_profile::check_session_exists(&working_dir) {
    // resume, not fresh
}
```

**Executing Claude Code is a builder:**
```rust
ClaudeCommand::new()
    .message("review this PR for security issues")
    .working_dir(&repo_path)
    .verbose(true)
    .execute()?;
```

## Architecture

The crates form two clean layers:

```
Layer 1 — Storage & Detection (zero process execution)
  claude_storage_core   ~/.claude/ JSONL parsing, token stats
  claude_profile        Account CRUD, token status, session paths
  claude_storage        CLI over storage_core

Layer 2 — Execution (owns Command::new("claude"))
  claude_runner_core    ClaudeCommand builder, single execution point
  claude_runner         CLI adapter with YAML schema for consumer workspace integration
```

The hard boundary between layers is enforced by a static analysis test: if `std::process::Command`
ever appears in `claude_profile`, the test suite fails immediately. The responsibility split is
not just a convention — it is a compiler-checked invariant.

## Design Philosophy

**Atomic operations by default.**
Account switching uses write-then-rename, not overwrite-in-place. Both files are in the
same `~/.claude/` directory, guaranteeing the same filesystem and therefore atomic rename.
A crash mid-switch leaves either the old credentials or the new ones — never a half-written
file.

**One place for every decision.**
`ClaudePaths` is the single authoritative source for all `~/.claude/` paths in the entire
workspace. `ClaudeCommand::execute()` is the single execution point for process spawning.
Neither convention is informal — both are enforced by tests that will tell you exactly
which file violated the rule if you drift.

**The docs are the source of truth.**
Each crate ships a `docs/` directory with feature, invariant, api, pattern, and data_structure
doc entity instances covering functional and non-functional requirements, vocabulary, and
architecture. Conformance checklists in doc instances are not aspirational —
every item has a named test that must pass for the checkmark to stay.

## Current State

The workspace is production-ready at L3 (nextest + doc tests + clippy, zero warnings).
All five crates pass. The account management layer (`claude_profile` 0.2) landed recently
and has full test coverage across all functional requirements.

What has been validated in production:
- `claude_storage` parsing ~1,900 projects and 2,400 sessions (~7 GB of JSONL) in under
  30 seconds from warm cache
- Session continuation detection running correctly inside live tooling
- The `claude_runner` builder being used to automate multi-step review workflows

What has been specified but not yet battle-tested:
- Account rotation across live subscriptions (the implementation is complete; the real-world
  rotation loop needs field time)
- Token expiry monitoring as an automation trigger

## Open Problems Worth Solving

The foundation is solid. The interesting work is ahead:

- **Rotation automation**: A daemon or hook that watches `TokenStatus` and rotates accounts
  automatically when `ExpiringSoon` fires — the piece that makes multi-subscription setups
  truly seamless.

- **Usage analytics**: `claude_storage_core` already parses token statistics per session.
  A tool that shows token spend by project, by day, by conversation type would make
  subscription management much more intelligent.

- **Conversation replay**: The JSONL format is fully parsed. A tool that extracts, summarises,
  or compares conversations across sessions would unlock entirely new workflows.

- **CI integration**: `claude_runner` running on pull requests as a structured code review
  step — with output parseable enough to post as a comment.

- **crates.io publishing**: The workspace was designed for this from day one. The privacy
  invariant is already enforced. Publishing `claude_profile` and `claude_runner_core` as
  public crates is a straightforward next step once the interfaces stabilize.

## The Stack

Rust 2021, pedantic clippy with `missing_inline_in_public_items` and `std_instead_of_core`
as hard errors. Every public item documented. Every functional requirement traced to a named
test.

The codebase is the kind you can read cold and understand — because the constraints that
make it hard to write are the same constraints that make it easy to maintain.
