# Format: Markdown

## Description

Human-readable conversation export with full metadata, thinking blocks, and tool use. Designed for reading in any markdown renderer (GitHub, VS Code, Obsidian) or plain text. Default format when `format::` is omitted. Renderer: `write_markdown_entry()` in `claude_storage_core/src/export.rs`.

## Trigger

Activated by `format::markdown` on `.export`, or by omitting `format::` entirely (this is the default format).

## Structure

```
# Session: {session_id}

**Path**: `{storage_path}`
**Entries**: {total_entries}
**Created**: {first_timestamp}
**Last Updated**: {last_timestamp}

---

## Entry 1 - User
*{timestamp}*

{user message content}

---

## Entry 2 - Assistant
*{timestamp}*

<details>
<summary>Thinking ({token_count} tokens)</summary>

{thinking content}
</details>

{text content}

**Tool Use**: `{tool_name}`
```json
{tool_input}
```

**Tool Result**:
```
{tool_output}
```

---
```

### Content Blocks

| Block Type | Rendering | Included |
|------------|-----------|----------|
| Text | Plain text paragraph | Always |
| Thinking | Collapsible `<details>` with token count in summary | When present |
| Tool Use | Bold label + tool name in backticks + JSON code block | When present |
| Tool Result | Bold label + code block | When present |

### Characteristics

- **Extension:** `.md`
- **Entry headings:** H2 (`## Entry N - Role`) with sequential numbering starting at 1
- **Timestamps:** italic on own line below entry heading
- **Separator:** horizontal rule (`---`) between entries and after header
- **Metadata fields:** bold label with value (`**Path**: ...`)
- **Default format** when `format::` is omitted

## Source

`claude_storage_core/src/export.rs` — `write_markdown_entry()`, `export_session()`

### Cross-References

- [params.md § format::](../params.md#parameter--5-format) — parameter definition and validation
- [types.md § ExportFormat](../types.md#exportformat) — type constants and parsing
- [testing/param/format.md](../testing/param/format.md) — test cases EC-1, EC-4, EC-7
