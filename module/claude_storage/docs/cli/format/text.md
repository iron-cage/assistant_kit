# Format: Text

### Purpose

Minimal plain-text transcript containing only human-readable conversation content. Strips all thinking blocks, tool use, and tool results — shows only what the user typed and what the assistant replied with text. Suitable for quick reading, sharing, or feeding into text-processing pipelines.

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

### Related

- [params.md § format::](../params.md#parameter--5-format) — parameter definition and validation
- [types.md § ExportFormat](../types.md#exportformat) — type constants and parsing
- [testing/param/format.md](../testing/param/format.md) — test case EC-3
