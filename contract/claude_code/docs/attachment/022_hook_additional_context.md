# ATTACHMENT: Hook Additional Context

### Scope

- **Purpose**: Specify the `hook_additional_context` payload, which records context a user-configured hook injected.
- **Responsibility**: Authoritative instance for the `hook_additional_context` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `hook_additional_context`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "hook_additional_context"` · **7 lines** (0.0017% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | array | always |
| `hookEvent` | string | always |
| `hookName` | string | always |
| `toolUseID` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "hook_additional_context",
  "content": [
    "The memory index at MEMORY.md is 196 lines, approaching the 200-line read limit. Compac…"
  ],
  "hookName": "PostToolUse:Edit",
  "toolUseID": "toolu_01FPJgZA4aXHbZqp9AjKkPNn",
  "hookEvent": "PostToolUse"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All four fields are universal** — `hookName`, `hookEvent`, `toolUseID`, and `content`, so every injection is fully attributable to a named hook on a named event for a specific tool call.

**Seven occurrences.** This is user-configured behavior, so its absence from a store says nothing about the mechanism's availability.

**The only payload kind sourced from user configuration** rather than from Claude Code itself.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
