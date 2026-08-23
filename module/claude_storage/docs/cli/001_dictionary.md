# Dictionary

### Scope

- **Purpose**: Define terms used in `claude_storage` CLI documentation.
- **Responsibility**: Canonical definitions for domain vocabulary.
- **In Scope**: CLI-specific terms, storage concepts, parameter terminology.
- **Out of Scope**: Implementation details (→ source code), API contracts (→ contract docs).

Definitions for terms used in `claude_storage` CLI documentation.

For conceptual hierarchy diagrams (containment, threading, agent session layout) see [storage/readme.md](../../../../contract/claude_code/docs/storage/readme.md#conceptual-model).

For the complete four-level taxonomy (Project / Conversation / Session / Entry) with pairwise relationship descriptions and ASCII containment diagram, see [taxonomy/readme.md](../../../../contract/claude_code/docs/taxonomy/readme.md).

### Core Terms

### Active Project

The project directory most recently modified within the current scope. Bare `clg .projects` shows all projects in the bidirectional neighborhood (scope::around), listed by recency. Computed by grouping sessions by project path and sorting projects by last-session write time. Distinct from Active Session: Active Project aggregates all conversations in one directory; Active Session is one JSONL file (one session) within that directory.

---

### Active Session

The most-recently modified session (JSONL file) within the current scope. Computed by sorting all sessions in scope by last-write time and taking the latest. The session belongs to the Active Project. `clg .projects` operates at the project level (not session level); use `.show` with a specific session ID to inspect the active session directly.

---

### Agent Session

A sub-agent conversation spawned during a main session. Two storage layouts coexist (per-project, neither deprecated): **flat** (older projects) where `agent-*.jsonl` files are siblings of the main session at project root, and **hierarchical** (newer projects) where agents live in `{session-uuid}/subagents/agent-{agentId}.jsonl` with optional `.meta.json` sidecars. Agents have `isSidechain: true` in their entries. The `sessionId` field in agent entries references the parent session UUID. Use `agent::1` in `.projects` to filter to agent sessions only. See [storage/readme.md](../../../../contract/claude_code/docs/storage/readme.md#conceptual-model) for layout diagrams.

---

### Conversation

The user-facing logical interaction unit within a project. Each conversation represents one complete thread of interaction between the user and Claude Code.

**Storage correspondence**: A conversation maps to one main session (root JSONL file) together with all agent sessions it spawned. In the current implementation this is a 1:1 mapping with Session Family — one root `.jsonl` file = one conversation. When the conversation chain detection algorithm is implemented (task 021), a conversation will be able to span multiple sequential session files that represent logically connected work.

**Conversation vs Session**: Session is the storage-layer artifact (one JSONL file); Conversation is the user-facing concept. Users see conversations; sessions are an implementation detail. Commands show conversation counts in output headers; session UUIDs are secondary display information.

**Agent sessions**: Each conversation owns zero or more agent sessions. Agents are sub-sessions spawned during the conversation; they are part of the conversation but not themselves conversations.

See also: [Session](#session), [Session Family](#session-family), [Entry](#entry), [taxonomy/readme.md](../../../../contract/claude_code/docs/taxonomy/readme.md).

---

### Conversation Chain

A sequence of two or more sessions within a project that represent logically connected work — where the user continued the same line of thinking across multiple `--new-session` invocations. No explicit storage link exists between sessions in a chain; the relationship must be inferred from temporal proximity, content context, or other heuristics.

Chain detection is the algorithm that groups sessions into conversations when more than one session file represents a single logical interaction. Until task 021 is implemented, each session (Session Family) is treated as its own conversation.

See also: [Conversation](#conversation), [Session](#session), [taxonomy/readme.md](../../../../contract/claude_code/docs/taxonomy/readme.md).

---

### Current Project

The project whose path-encoded directory in `~/.claude/projects/` matches the current working directory (`cwd`). When no `project::` parameter is provided, commands default to the current project. If no matching project exists, commands report "no history found."

---

### Entry

A single line in a JSONL session file representing one conversation turn. Each Entry is a storage envelope — fields like `uuid`, `parentUuid`, `timestamp`, `sessionId`, `isSidechain`, `cwd`, `gitBranch` — wrapping a `message` field that holds the Claude API Message payload (`role`, `content`, and for assistant entries `model` and `usage`).

`entry.type` and `entry.message.role` carry the same `"user"/"assistant"` values but belong to different layers: `type` is the storage envelope classifier; `role` is the Claude API Message field.

Entries are append-only: once written to a `.jsonl` file they are never modified.

See [jsonl/readme.md](../../../../contract/claude_code/docs/jsonl/readme.md) for the full field schema and content block types.

---

### Main Session

A top-level Claude Code conversation, as opposed to an agent session. Stored as a UUID-named or topic-named `.jsonl` file. `isSidechain: false` in its entries.

---

### Path Encoding

The algorithm Claude Code uses to convert a filesystem path into a safe directory name for `~/.claude/projects/`. **Every** non-alphanumeric character becomes a hyphen (`-`) — the real algorithm is a blanket `${path//[^a-zA-Z0-9]/-}` substitution, so path separators, underscores, and dots are all indistinguishable in the result. Example: `/home/alice/my_app` and `/home/alice/my/app` both encode to `-home-alice-my-app`. Path-encoded IDs are accepted by the `project::` parameter.

The encoding is therefore **lossy and not invertible**. `decode_path` applies a heuristic, and callers that need a real path fall back to a filesystem-guided walk when the heuristic's guess does not exist on disk (`src/cli/scope.rs` `decode_storage_base`). When neither resolves, the displayed path is a guess — `.projects` marks such rows `⚠ gone`. See `Fix(BUG-366)` in `claude_storage_core::encode_path` for the character class, and issue-024/029/030/031/032/035 for the ambiguity cases this creates.

See [storage/readme.md](../../../../contract/claude_code/docs/storage/readme.md) for the encoding specification.

---

### Project

A directory on the filesystem that has been opened in Claude Code. Each project has a corresponding directory in `~/.claude/projects/` (path-encoded or UUID-named) containing all session JSONL files for that project.

---

### Scope

The discovery boundary for project and conversation listing. Controls which projects are searched: `local` (current project only), `relevant` (all ancestor projects up to `/`), `under` (all descendant projects), `around` (bidirectional: ancestors + current + descendants), or `global` (all projects in storage). Applies to `.projects` (scope default: `around`), `.search`, and `.count`. (`.list`, deprecated, also specified this, scope default `global`, before its role migrated to `.projects`.)

---

### Session Directory

A hyphen-prefixed directory inside a base project directory used as the working directory for a specific Claude Code session topic. Takes the form `{base}/-{topic}` (e.g., `/home/user/project/-default_topic`). Claude Code stores conversation history keyed to this directory path, not to the parent base directory. Use `.session.dir` to compute the path and `.session.ensure` to create it.

---

### Session Topic

The human-readable name component of a session directory. Stored without the leading `-` in the `topic::` parameter but always prefixed with `-` in the filesystem path (e.g., topic `default_topic` → directory suffix `-default_topic`). Multiple topics can coexist under the same base directory, each maintaining independent conversation history.

---

### Fork-Mode Topic

A named session INSIDE a base directory's own storage, rather than a `-{topic}` sibling directory. The topic name maps deterministically to a session file via UUIDv5: `{storage}/{UUIDv5( canonical base path + NUL + name )}.jsonl`. Used only by `.session.path topic::` — every other `topic::` consumer keeps the Session Topic dir-suffix sense above. Because all fork-mode topics share the base directory's storage (and therefore its path-keyed prompt cache), forking preserves cache where a Session Directory switch invalidates it. The name-to-UUID mapping is one-way; the `clr` runner keeps a name registry for listing. See [`command/16_session_path.md`](command/16_session_path.md) § Topic Sense Collision.

---

### Conversation Topic

The first user message text of a conversation, used as its one-line display preview. Rendered by `.projects show_topic::1` — newlines flattened to spaces, trimmed, truncated to 90 characters. Distinct from Session Topic (above): a Session Topic names the working directory a session is keyed to; a Conversation Topic summarizes what the conversation is about.

---

### Strategy

The resume/fresh decision for `.session.ensure`. `resume` means an existing conversation history was found in storage for the session directory; `fresh` means no history was found and the session starts clean. Can be auto-detected from storage (default) or forced via `strategy::resume|fresh`.

---

### Session

The storage-layer artifact for one Claude Code interaction: a single `.jsonl` file containing all entries for that interaction. Session IDs are either UUID v4 strings (e.g., `feed0001-0000-4000-8000-000000000001`) or human-readable topic names (e.g., `-default_topic`, `-commit`). The session ID is the filename stem without the `.jsonl` extension.

**Session vs Conversation**: Session is the implementation detail (JSONL file on disk); Conversation is the user-facing concept. In the current implementation one session = one conversation, but the conversation chain detection algorithm (task 021) will allow one conversation to span multiple sessions.

A session is a container of Entries. Entries are appended as the conversation progresses and are never modified. Entries within a session are linked by `parentUuid` into a thread (see [storage/readme.md](../../../../contract/claude_code/docs/storage/readme.md#conceptual-model)).

Session files have no explicit cross-file links — no `parentUuid` or `continuedFrom` field points from one session to another. Any cross-session relationships (Conversation Chains) must be inferred externally.

---

### Session Family

A root (main) session together with all agent sessions it spawned, treated as a single display unit. In the current implementation, one Session Family corresponds to one Conversation — each root `.jsonl` file is one conversation, and its agent sessions are part of that conversation. Families are not shown by bare `.projects`, whose default `detail::projects` overview reports only a per-project agent count. Under `detail::sessions`, `.projects` shows one family per root: the root line carries an inline `[N agents: type breakdown]` suffix and the agents are collapsed; adding `show_tree::1` tree-indents the agents under their root. An orphan family is one whose root session file has been deleted; it is shown with a `?` marker. A childless root (no agents) shows no bracket suffix.

---

### Session Filter

The `session::` parameter matches sessions by case-insensitive substring of the session ID (filename stem). For example, `session::commit` matches both `-commit.jsonl` and `auto-commit.jsonl`. This is distinct from `session_id::`, which is used for direct access to a specific session.

---

### Storage Root

The root directory where Claude Code stores all data. Default: `~/.claude/`. Can be overridden with the `path::` parameter in `.status`, or with the `CLAUDE_STORAGE_ROOT` environment variable for test isolation.

---

### Output Toggle

A boolean parameter (0 or 1) enabling an optional output section. The CLI uses per-block toggles: `show_stat::` for statistics footer in `.show`, `show_tokens::` for token usage in `.show`/`.status`, `show_tree::` for agent tree display in `.projects`. See [Output Control group](param_group/01_output_control.md).
