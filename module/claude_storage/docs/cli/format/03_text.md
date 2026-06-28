# Format :: 3. Text

### Scope

- **Purpose**: Specify the Text export format.
- **Responsibility**: Structure, rendering rules, and output conventions for Text export.
- **In Scope**: Output structure, content ordering, file conventions.
- **Out of Scope**: Parameter specs (→ `param/`), command behavior (→ `command/`).

### Description

Minimal plain-text transcript containing only human-readable conversation content. Strips all thinking blocks, tool use, and tool results — shows only what the user typed and what the assistant replied with text. Suitable for quick reading, sharing, or feeding into text-processing pipelines. Renderer: `write_text_entry()` in `claude_storage_core/src/export.rs`.

### Trigger

Activated by `format::text` on `.export`.

### Structure

```
Session: {session_id}
Path: {storage_path}
Entries: {total_entries}

---

[User] {timestamp}
{user message content}

---

[Assistant] {timestamp}
{text content only}

---
```

### Content Filtering

| Block Type | Included | Reason |
|------------|----------|--------|
| Text | Yes | Core conversation content |
| Thinking | No | Internal reasoning, not part of conversation |
| Tool Use | No | Implementation detail, not conversational |
| Tool Result | No | Implementation detail, not conversational |

Only `ContentBlock::Text` variants are rendered for assistant entries. All other block types are silently skipped.

### Characteristics

- **Extension:** `.txt`
- **Entry prefix:** `[Role] timestamp` on one line (no markdown heading syntax)
- **Separator:** horizontal rule (`---`) between entries and after header
- **Header fields:** plain `Label: value` (no bold, no backticks)
- **No markdown:** output contains no heading syntax (`#`), no bold (`**`), no code blocks
- **Timestamps not included in header** — only session ID, path, and entry count

### Source

`claude_storage_core/src/export.rs` — `write_text_entry()`, `export_session()`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 6 | [`.export`](../command/06_export.md) | Activated when `format::text` is supplied |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |
