# Domain Dictionary

Definitions for terms used in `claude_storage` CLI documentation.

For conceptual hierarchy diagrams (containment, threading, agent session layout) see [002_storage_organization.md](../../../../docs/claude_code/002_storage_organization.md#conceptual-model).

## Core Terms

### Active Project

The project directory most recently modified within the current scope. Displayed by bare `clg .projects` (no arguments) as a single summary block showing aggregated session count and last message — not a list. Computed by grouping sessions by project path, sorting projects by last-session write time, and returning the latest. Distinct from Active Session: Active Project aggregates all sessions in one directory; Active Session is one JSONL file within that directory.

---

### Active Session

The most-recently modified session within the current scope. Computed by sorting all sessions in scope by last-write time and taking the latest. After Task 016, bare `clg .projects` shows Active Project instead of Active Session.

---

### Agent Session

A sub-agent conversation spawned during a main session. Two storage layouts coexist (per-project, neither deprecated): **flat** (older projects) where `agent-*.jsonl` files are siblings of the main session at project root, and **hierarchical** (newer projects) where agents live in `{session-uuid}/subagents/agent-{agentId}.jsonl` with optional `.meta.json` sidecars. Agents have `isSidechain: true` in their entries. The `sessionId` field in agent entries references the parent session UUID. Use `agent::1` in `.list` or `.projects` to filter to agent sessions only. See [002_storage_organization.md](../../../../docs/claude_code/002_storage_organization.md#conceptual-model) for layout diagrams.

---

### Current Project

The project whose path-encoded directory in `~/.claude/projects/` matches the current working directory (`cwd`). When no `project::` parameter is provided, commands default to the current project. If no matching project exists, commands report "no history found."

---

### Entry

A single line in a JSONL session file representing one conversation turn. Each Entry is a storage envelope — fields like `uuid`, `parentUuid`, `timestamp`, `sessionId`, `isSidechain`, `cwd`, `gitBranch` — wrapping a `message` field that holds the Claude API Message payload (`role`, `content`, and for assistant entries `model` and `usage`).

`entry.type` and `entry.message.role` carry the same `"user"/"assistant"` values but belong to different layers: `type` is the storage envelope classifier; `role` is the Claude API Message field.

Entries are append-only: once written to a `.jsonl` file they are never modified.

See [004_jsonl_format.md](../../../../docs/claude_code/004_jsonl_format.md) for the full field schema and content block types.

---

### Main Session

A top-level Claude Code conversation, as opposed to an agent session. Stored as a UUID-named or topic-named `.jsonl` file. `isSidechain: false` in its entries.

---

### Path Encoding

The algorithm Claude Code uses to convert a filesystem path into a safe directory name for `~/.claude/projects/`. Slashes (`/`) become hyphens (`-`); no other transformation is applied. Example: `/home/user1/pro` → `-home-user1-pro`. Path-encoded IDs are accepted by the `project::` parameter.

See [002_storage_organization.md](../../../../docs/claude_code/002_storage_organization.md) for the encoding specification.

---

### Project

A directory on the filesystem that has been opened in Claude Code. Each project has a corresponding directory in `~/.claude/projects/` (path-encoded or UUID-named) containing all session JSONL files for that project.

---

### Scope

The discovery boundary for project and session listing. Controls which projects are searched: `local` (current project only), `relevant` (all ancestor projects up to `/`), `under` (all descendant projects), or `global` (all projects in storage). Applies to `.projects` (scope default: `under`), `.list` (scope default: `global`), `.search`, and `.count`. Mirrors the `scope` concept in `kbase` for consistent mental model across tools.

---

### Session Directory

A hyphen-prefixed directory inside a base project directory used as the working directory for a specific Claude Code session topic. Takes the form `{base}/-{topic}` (e.g., `/home/user/project/-default_topic`). Claude Code stores conversation history keyed to this directory path, not to the parent base directory. Use `.session.dir` to compute the path and `.session.ensure` to create it.

---

### Session Topic

The human-readable name component of a session directory. Stored without the leading `-` in the `topic::` parameter but always prefixed with `-` in the filesystem path (e.g., topic `default_topic` → directory suffix `-default_topic`). Multiple topics can coexist under the same base directory, each maintaining independent conversation history.

---

### Strategy

The resume/fresh decision for `.session.ensure`. `resume` means an existing conversation history was found in storage for the session directory; `fresh` means no history was found and the session starts clean. Can be auto-detected from storage (default) or forced via `strategy::resume|fresh`.

---

### Session

A single Claude Code conversation for a project, stored as one `.jsonl` file. Session IDs are either UUID v4 strings (e.g., `8d795a1c-c81d-4010-8d29-b4e678272419`) or human-readable topic names (e.g., `-default_topic`, `-commit`). The session ID is the filename stem without the `.jsonl` extension.

A session is a container of Entries. Entries are appended as the conversation progresses and are never modified. Entries within a session are linked by `parentUuid` into a thread (see [002_storage_organization.md](../../../../docs/claude_code/002_storage_organization.md#conceptual-model)).

---

### Session Family

A root (main) session together with all agent sessions it spawned, treated as a single display unit. At `verbosity::1`, `.projects` shows one family per root: the root line carries an inline `[N agents: type breakdown]` suffix and the agents are collapsed. At `verbosity::2+` agents are tree-indented under their root. An orphan family is one whose root session file has been deleted; it is shown with a `?` marker. A childless root (no agents) shows no bracket suffix.

---

### Session Filter

The `session::` parameter matches sessions by case-insensitive substring of the session ID (filename stem). For example, `session::commit` matches both `-commit.jsonl` and `auto-commit.jsonl`. This is distinct from `session_id::`, which is used for direct access to a specific session.

---

### Storage Root

The root directory where Claude Code stores all data. Default: `~/.claude/`. Can be overridden with the `path::` parameter in `.status`, or with the `CLAUDE_STORAGE_ROOT` environment variable for test isolation.

---

### Verbosity

A 0-5 integer controlling output detail level. `0` = minimal/machine-readable, `1` = standard summary (default), `2` = detailed with counts, `3` = verbose with full fields, `4-5` = reserved. The `v` alias is available for all commands that accept `verbosity`.
