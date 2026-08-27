# ATTACHMENT: Agent Listing Delta

### Scope

- **Purpose**: Specify the `agent_listing_delta` payload, which records agent-type roster changes.
- **Responsibility**: Authoritative instance for the `agent_listing_delta` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `agent_listing_delta`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "agent_listing_delta"` · **17,559 lines** (4.31% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `addedLines` | array | always |
| `addedTypes` | array | always |
| `isInitial` | boolean | always |
| `removedTypes` | array | always |
| `showConcurrencyNote` | boolean | always |

Captured example — the `attachment` object only:

```json
{
  "type": "agent_listing_delta",
  "addedTypes": [
    "claude",
    "Explore",
    "general-purpose",
    "…"
  ],
  "addedLines": [
    "- claude: Catch-all for any task that doesn't fit a more specific agent. FleetView's de…",
    "- Explore: Fast read-only search agent for locating code. Use it to find files by patte…",
    "- general-purpose: General-purpose agent for researching complex questions, searching f…",
    "…"
  ],
  "removedTypes": [],
  "isInitial": true,
  "showConcurrencyNote": true
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All five fields are universal**, including `isInitial` — so, as with `skill_listing`, reconstruction can start from the most recent full roster rather than the session start.

**`showConcurrencyNote` is presentation state, not roster state.** Folding it into an agent-type set is a category error.

**Pairs with `deferred_tools_delta` and `mcp_instructions_delta`** as the three delta-encoded roster channels; only this one and `skill_listing` carry `isInitial`.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
