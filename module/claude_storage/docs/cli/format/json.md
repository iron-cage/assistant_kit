# Format: JSON

## Description

Machine-readable export preserving the complete raw JSONL entry structure. Wraps all session entries in a JSON object with session metadata. Suitable for programmatic analysis, import into other tools, or archival. Renderer: `write_json_value()` in `claude_storage_core/src/export.rs`.

## Trigger

Activated by `format::json` on `.export`.

## Structure

```json
{
  "session_id": "{session_id}",
  "storage_path": "{storage_path}",
  "entries": [
    {
      "uuid": "...",
      "parentUuid": null,
      "timestamp": "...",
      "type": "user",
      "message": { "role": "user", "content": "..." },
      ...
    },
    {
      "uuid": "...",
      "parentUuid": "...",
      "timestamp": "...",
      "type": "assistant",
      "message": { "model": "...", "content": [...] },
      ...
    }
  ]
}
```

### Content Handling

| Aspect | Behavior |
|--------|----------|
| Entry fields | All original JSONL fields preserved verbatim |
| Pretty-printing | 2-space indentation, entries at 4-space indent |
| String escaping | `"`, `\`, `\n`, `\r`, `\t` escaped per JSON spec |
| Thinking blocks | Present in raw `content` array (not filtered) |
| Tool use/results | Present in raw `content` array (not filtered) |
| Entry separator | Comma between entries, no trailing comma |

### Characteristics

- **Extension:** `.json`
- **Top-level structure:** JSON object with `session_id`, `storage_path`, `entries` keys
- **Entries array:** each element is the complete parsed-and-re-serialized JSONL line
- **Streaming:** reads raw JSONL lines via `BufReader`, parses each with the internal JSON parser, then pretty-prints. Memory holds one entry at a time.
- **Validation:** parseable with `jq .` or any JSON parser

## Source

`claude_storage_core/src/export.rs` — `write_json_value()`, `export_session()`

### Cross-References

- [params.md § format::](../params.md#parameter--5-format) — parameter definition and validation
- [types.md § ExportFormat](../types.md#exportformat) — type constants and parsing
- [testing/param/format.md](../testing/param/format.md) — test case EC-2
