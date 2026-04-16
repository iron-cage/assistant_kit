# assistant — Vision

Coding agents have no standard programmatic interface — no public API, no documented calling
convention, no built-in way to integrate into automated workflows. Every team that uses them
seriously ends up writing the same glue code from scratch.

`assistant` changes that. The architecture is agent-agnostic — `dream` and `assistant` carry
no `claude_` prefix by design, ready to grow beyond Claude Code as new coding agents emerge.

## The Problem Space

The first target is Claude Code. Three friction points emerge when using it seriously:

**Account rotation is manual and fragile.** Each subscription has a 5-hour active-use window.
Switching between multiple subscriptions means manually editing `~/.claude/.credentials.json`
— one transient write failure and the session is gone.

**Session lifecycle is opaque.** Conversation history lives in `~/.claude/projects/` keyed by
a non-obvious escaped path. Answering "should this run continue or start fresh?" requires
digging through undocumented directory structures.

**Automation requires hand-rolled glue.** Any tool invoking Claude Code — a CI hook, a code
review bot, a scheduled job — needs to know which flags to pass, how to handle interactive
vs non-interactive modes, how to wire stderr vs stdout. There is no documented calling
convention.

## What We Built

A 13-crate layered Rust workspace:

```
Layer 3: assistant              Agent-agnostic super-app aggregating all CLI tools (ast)
             ↓
Layer 2: claude_profile  (clp)  Account management, token status, ~/.claude/ paths
         claude_storage  (clg)  CLI for exploring Claude Code session storage
         claude_runner   (clr)  Claude Code execution with session continuity
         claude_version  (clv)  Claude Code version manager
         claude_assets   (cla)  Install artifacts (rules, skills, commands) via symlinks
         dream                  Agent-agnostic library facade re-exporting all core crates
             ↓
Layer 1: claude_profile_core    Token status + account domain logic
         claude_version_core    Version detection, settings domain helpers
         claude_runner_core     ClaudeCommand builder + single process execution point
         claude_assets_core     Symlink-based artifact installer domain logic
             ↓
Layer 0: claude_core          Shared primitives: ClaudePaths, process utilities
*        claude_storage_core    Zero-dep JSONL parser for ~/.claude/
```

## Design Principles

**Atomic operations.** Account switching uses write-then-rename, not overwrite-in-place.
A crash mid-switch leaves either the old credentials or the new ones — never a half-written
file.

**One place for every decision.** `ClaudePaths` is the single authoritative source for all
`~/.claude/` paths. `ClaudeCommand::execute()` is the single process execution point. Both
are enforced by tests that name exactly which file violated the rule.

**Docs are the source of truth.** Each crate ships a `docs/` directory with feature,
invariant, and pattern doc instances. Every conformance checklist item has a named test that
must pass for the checkmark to stay.

## Current State

13 crates. All pass L3 (nextest + doc tests + clippy, zero warnings).

Validated in production:
- `claude_storage` parsing ~1,900 projects and 2,400 sessions (~7 GB of JSONL) in under
  30 seconds from warm cache
- Session continuation detection running inside live tooling
- `claude_runner` automating multi-step review workflows

## Open Problems

- **Rotation automation:** A daemon watching `TokenStatus` and rotating accounts
  automatically when `ExpiringSoon` fires — the piece that makes multi-subscription setups
  truly seamless.
- **Usage analytics:** Token spend by project, day, and conversation type built on the
  existing `claude_storage_core` statistics.
- **Conversation replay:** The JSONL format is fully parsed — extraction, summarisation,
  and cross-session comparison are straightforward to build.
- **CI integration:** `claude_runner` on pull requests as a structured code review step
  with output parseable enough to post as a comment.
- **crates.io publishing:** The privacy invariant is already enforced. Publishing Layer 0
  and Layer 1 crates is a straightforward next step once interfaces stabilise.

## Stack

Rust 2021, pedantic clippy with `missing_inline_in_public_items` and `std_instead_of_core`
as hard errors. Every public item documented. Every functional requirement traced to a named
test.
