# ATTACHMENT: MCP Instructions Delta

### Scope

- **Purpose**: Specify the `mcp_instructions_delta` payload, which records changes to the instruction blocks contributed by MCP servers.
- **Responsibility**: Authoritative instance for the `mcp_instructions_delta` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `mcp_instructions_delta`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "mcp_instructions_delta"` · **4,840 lines** (1.19% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `addedBlocks` | array | always |
| `addedNames` | array | always |
| `removedNames` | array | always |

Captured example — the `attachment` object only:

```json
{
  "type": "mcp_instructions_delta",
  "addedNames": [
    "claude-in-chrome"
  ],
  "addedBlocks": [
    "## claude-in-chrome\n**IMPORTANT: If the Chrome browser tools are deferred (must be load…"
  ],
  "removedNames": []
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**No `isInitial` field.** Unlike `skill_listing` and `agent_listing_delta`, this delta stream has no full-roster marker, so reconstructing current MCP instructions requires folding from the start of the session.

**`addedBlocks` carries text, `addedNames` carries identifiers.** Both are universal; a consumer tracking only which servers contributed can read `addedNames` and skip the block bodies.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
