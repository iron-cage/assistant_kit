# Parameter :: 39. `latest::`

### Scope

- **Purpose**: Specify the `latest::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `latest::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Selects the most recently modified qualifying session file in the resolved storage.

**Type:** Boolean

**Fundamental Type:** Boolean

**Constraints:**
- Accepts `0`/`1` (unilang Boolean)
- Mutually exclusive with `session::` and `topic::` in `.session.path` — more than one selector is an argument error (exit 1)

**Default:** `0` — but the *behavior* is the effective default: `.session.path` with no selector at all behaves identically to `latest::1`. The explicit form exists for script readability and forward compatibility, not because it changes anything today.

**Commands:** `.session.path`

**Purpose:** Names the only disk-reading selector of `.session.path`. "Latest" means: among the storage's `*.jsonl` files, excluding `agent-*` files and zero-length files, the one with the newest modification time. An empty result is exit `2` (`no sessions in {storage}` on stderr), distinguishing "nothing to resolve" from a usage error.

**Examples:**
```bash
# Explicit — identical to the bare default form
claude_storage .session.path path::/home/user/project latest::1

# Error: two selectors
claude_storage .session.path latest::1 topic::review   # exit 1, mutually exclusive
```

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 15 | [`.session.path`](../command/15_session_path.md) | effective default | Most recently modified non-`agent-` non-empty `.jsonl`; exit 2 when none |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
