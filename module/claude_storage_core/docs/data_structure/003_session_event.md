# Data Structure: Session Event

### Scope

- **Purpose**: Model every kind of line a session JSONL file holds, not only the two a conversation is made of, so a reader can reconstruct how the session's *context* was assembled.
- **Responsibility**: Documents the `SessionEvent` envelope, the `EventKind` line taxonomy, the `Attachment` payload taxonomy, and the two rules that keep this schema from colliding with `Entry`.
- **In Scope**: Envelope field semantics, the line kinds and attachment subtypes modelled, forward-compatibility behaviour, the error contract.
- **Out of Scope**: Conversation message content (→ `data_structure/001_storage_hierarchy.md`, `Entry`), token accounting (→ `SessionStats` in `../../src/stats.rs`), the harness's static system prompt, which never appears in the JSONL at all.

### Abstract

`Entry` models the `user` and `assistant` lines, and `Session::entries()` deliberately drops everything else — its "Graceful Degradation Design" note. That policy is correct for the readers it serves (export, search, statistics), and this type does not change it.

What it leaves unread is a second stream of lines describing how the session's context was assembled: which tools were deferred, which agents and skills were offered, how much of the token budget the harness reported remaining. `SessionEvent` parses the same file with a wider schema to reach them.

Two rules hold the two schemas apart:

- **A `user`/`assistant` line is recognized, never re-parsed.** `EventKind::User` and `EventKind::Assistant` carry no message payload; `Entry::from_json_line` remains the single parser for conversation content. Duplicating that schema here would mean two models of one line, drifting independently. `SessionEvent::is_conversation()` is how a caller routes those lines to the right parser.
- **An unrecognized line is data, not an error.** Claude Code's format grows between releases, so an unknown `type` becomes `EventKind::Other` rather than a parse failure. A reader written against one version is therefore never broken by a newer one — the added kind is counted and skipped, not lost silently.

### Structure

**SessionEvent** — the envelope Claude Code repeats on most lines. Every field but `session_id` and `kind` is optional, because the short envelope kinds (`mode`, `ai-title`, `last-prompt`) carry only a session id and their own payload.

- `session_id`: conversation this line belongs to.
- `uuid` / `parent_uuid`: line identity and threading.
- `timestamp`: ISO 8601.
- `cwd`: working directory the session runs in.
- `version`: the Claude Code version that wrote the line. Constant per release, and the key under which any version-specific static context would be cached.
- `is_sidechain`: whether the line belongs to a subagent conversation rather than the main one.
- `kind`: the line taxonomy below.

**EventKind** — one variant per envelope `type`:

| `type` | Variant | Payload |
|--------|---------|---------|
| `user` | `User` | none by design |
| `assistant` | `Assistant` | none by design |
| `system` | `System` | `subtype`, `duration_ms`, `message_count` |
| `attachment` | `Attachment` | see below |
| `mode` | `Mode` | `mode` |
| `permission-mode` | `PermissionMode` | `permission_mode` |
| `last-prompt` | `LastPrompt` | `leaf_uuid` |
| `ai-title` | `AiTitle` | `title` |
| `queue-operation` | `QueueOperation` | `operation` |
| *anything else* | `Other` | the declared `type` string |

Observed `system` subtypes are `turn_duration`, `compact_boundary`, and `away_summary`; the subtype is kept as a string rather than an enum because it is a growing set with no payload of its own beyond the two counters.

**Attachment** — one variant per `attachment.type`. These are what the harness injects into a session's context:

| `attachment.type` | Variant | Retained fields |
|-------------------|---------|-----------------|
| `total_tokens_reminder` | `TotalTokensReminder` | `remaining` |
| `deferred_tools_delta` | `DeferredToolsDelta` | `added`, `removed`, `readded`, `pending_mcp_servers` |
| `agent_listing_delta` | `AgentListingDelta` | `added`, `removed`, `is_initial` |
| `mcp_instructions_delta` | `McpInstructionsDelta` | `added`, `removed` |
| `skill_listing` | `SkillListing` | `names`, `skill_count`, `is_initial` |
| `invoked_skills` | `InvokedSkills` | `skills` (name + resolution path) |
| `task_reminder` | `TaskReminder` | `item_count` |
| `task_status` | `TaskStatus` | `task_id`, `task_type`, `status`, `description`, `output_file_path` |
| `command_permissions` | `CommandPermissions` | `allowed_tools` |
| `queued_command` | `QueuedCommand` | `prompt`, `command_mode` |
| `edited_text_file` | `EditedTextFile` | `filename` |
| `compact_file_reference` | `CompactFileReference` | `filename`, `display_path` |
| `file` | `File` | `filename`, `display_path` |
| `date_change` | `DateChange` | `new_date` |
| *anything else* | `Other` | the declared `type` string |

**InvokedSkill** — `name` plus the `path` the skill resolved from, e.g. a user-settings or project-local prefix.

### Design Decisions

- **Content blobs are dropped.** Several attachments carry a human-readable `content` or `addedLines` field restating the structured field beside it — `invoked_skills` repeats each skill's entire text, `file` repeats the file's. Only the structured field is retained: the blob is large, it is already on disk at the path the attachment names, and a reader needs to know the skill ran, not to re-read it.
- **`skill_count` is kept, not derived from `names`.** A disagreement between the two means the listing was truncated. Deriving the count from the vector's length would erase that signal.
- **`readdedNames` is a distinct list.** A tool deferred, removed, then deferred again appears only there. Folding it into `added` would be lossy in the other direction — a reader could no longer tell a first deferral from a re-deferral.
- **The token budget is parsed out of prose.** `total_tokens_reminder` reports the remaining budget only as text (`<total_tokens>N tokens left</total_tokens>`), never as a numeric field, and it is *not* derivable by summing usage — it is the harness's own number. The wording is not a stable contract, so text carrying no parseable number yields `None` rather than a wrong value.
- **Counts and durations reject negatives.** JSON numbers arrive as `f64`; a negative or non-finite value in a field that is semantically a count is discarded rather than wrapped.

### Edge Cases

- **Missing envelope fields**: absent optional fields become `None`, absent `sessionId` becomes an empty string, absent `isSidechain` becomes `false`. None of these fail the parse — the short envelope kinds legitimately omit most of the envelope.
- **`attachment` line with no `attachment` object**: degrades to `EventKind::Other { kind: "attachment" }` rather than a parse failure.
- **Attachment with no `type`**: becomes `Attachment::Other` with an empty kind string.
- **Non-string elements in a name array**: dropped individually. One unexpected element must not erase the names beside it.
- **Structural failures are still errors**: a line that is not valid JSON, is not a JSON object, or carries no `type` field returns `Error::Parse`. Forward compatibility covers unknown kinds, not malformed data — accepting these would make `Other` meaningless.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/event.rs` | `SessionEvent`, `EventKind`, `Attachment`, `InvokedSkill` |
| source | `../../src/entry.rs` | `Entry` — the conversation-line parser this type defers to |
| source | `../../src/session.rs` | `Session::entries()` — the narrow reader whose skip policy is unchanged |
| test | `../../tests/event_test.rs` | Envelope, line kind, attachment subtype, and forward-compatibility tests |
| doc | `../data_structure/001_storage_hierarchy.md` | Storage → Project → Session → Entry model |
| doc | `../algorithm/001_path_encoding.md` | How a session file's project directory name is derived |

### Sources

| File | Notes |
|------|-------|
| Session JSONL files | Line kinds and attachment subtypes enumerated from observed sessions; field names follow Claude Code's own JSON keys |
