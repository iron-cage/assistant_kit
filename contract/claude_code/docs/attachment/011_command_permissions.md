# ATTACHMENT: Command Permissions

### Scope

- **Purpose**: Specify the `command_permissions` payload, which records the tool allow-list granted for a command.
- **Responsibility**: Authoritative instance for the `command_permissions` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `command_permissions`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "command_permissions"` · **8,539 lines** (2.10% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `allowedTools` | array | always |

Captured example — the `attachment` object only:

```json
{
  "type": "command_permissions",
  "allowedTools": []
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`allowedTools` is the sole field** and is universal.

**Security-relevant.** Together with the `permission-mode` envelope, this is the in-log record of what a command was authorized to invoke.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
