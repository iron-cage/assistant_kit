# User Story 030: Topic Creation

### Scope

- **Persona**: Developer
- **Goal**: Start or continue a named, isolated workspace conversation with a single command — without inventing a `--topic` name or remembering whether that name was already used.

### User Story

> As a developer juggling several parallel lines of work in the same project,
> I want a single command that either starts a fresh named topic or continues an existing one,
> so I don't have to track topic directory names or remember whether a topic already exists.

### Acceptance Criteria

- **AC-001 (Auto-named topic):** `clr topic "message"` (no `--topic`) generates a concise slug from `message`, disambiguates it via a numeric counter suffix against BOTH freshness signals under the effective `--dir` — existing topic directories AND session storage already holding a qualifying session (probed under the canonical storage key, symlink/`..`-safe — Fix(BUG-542), Fix(BUG-543)) — and uses it as `--topic`'s value: always a fresh name on both signals.
- **AC-002 (Explicit topic name):** `clr topic --topic NAME "message"` uses `NAME` directly as `--topic`'s value, bypassing slug generation entirely.
- **AC-003 (Clone on first use):** The first invocation of a given topic name (auto-generated or explicit) has no session file yet in that topic directory — the runner clones the most recent session from `--from`'s effective source (default: cwd) into the new topic directory before spawn.
- **AC-004 (Continue on reuse):** A later invocation of the same explicit topic name finds a session file already present in that topic directory — the runner continues that topic's own accumulated conversation instead of re-cloning.
- **AC-005 (No new session-management code):** Clone-vs-continue behavior is entirely a consequence of the pre-existing `--topic` + `--from` session-transplant mechanism (`../param/076_from.md` § Behavior); `topic` contributes the slug-generation step plus AC-001's freshness probe — which reuses `builder.rs`'s own storage-lookup helpers (`session_exists`/`physical_abs`, Fix(BUG-542)) rather than reimplementing them, so auto-naming and the transplant mechanism's own "already has a session" definition can never drift apart.
- **AC-006 (Full parameter inheritance):** Every parameter accepted by `run`/`ask` is accepted by `topic` with an identical default, except `--topic` (see AC-001/AC-002).

**Mechanism:** `topic` is a pre-configured alias of `run` that overrides `--topic`'s default value only — see [`../command_group/01_run_ask.md`](../command_group/01_run_ask.md) § Representation Absorption Test.

### Primary Flags

| Flag | Role |
|------|------|
| `--topic <NAME>` | Explicit topic name; when omitted, `topic` auto-generates one from `MESSAGE` |
| `[MESSAGE]` | Positional message; also the source text for slug generation when `--topic` is omitted |

### Examples

```sh
# Auto-named: slug generated from the message, always fresh
clr topic "Investigate the flaky concurrency-gate test"

# Explicit name — first call clones the current session into the new topic
clr topic --topic auth-refactor "Start refactoring the auth module"

# Same explicit name — second call continues that topic's own conversation
clr topic --topic auth-refactor "What did we change so far?"
```

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 11 | [`topic`](../command/11_topic.md) | New command: `run`/`ask` alias with an auto-naming `--topic` default |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--topic` and `--from` are both Runner Control flags |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 28 | [`--topic`](../param/028_topic.md) | Named topic directory; `topic` overrides its default with a generated slug |
| 76 | [`--from`](../param/076_from.md) | Session source; drives the clone-vs-continue mechanism `topic` reuses unchanged |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 22 | [022_session_isolation_topic.md](022_session_isolation_topic.md) | `--topic` isolation mechanism `topic` builds its auto-naming on top of |
| 28 | [028_session_transplant.md](028_session_transplant.md) | Clone/continue mechanism `topic` reuses unchanged via `--from` |
