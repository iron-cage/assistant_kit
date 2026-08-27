# ENVELOPE: Assistant

### Scope

- **Purpose**: Specify the `assistant` envelope: a model turn, its embedded API message, and the provenance fields that attribute it to a skill, agent, plugin, or MCP tool.
- **Responsibility**: Authoritative instance for the `assistant` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `assistant` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "assistant"` · **Class A** (Full Envelope) · **2,314,741 lines** (45.84% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `message` | object | always |
| `requestId` | string | 2,292,393 of 2,314,741 (99.0%) |
| `slug` *(ambient)* | string | 2,283,571 of 2,314,741 (98.7%) |
| `entrypoint` *(ambient)* | string | 2,202,098 of 2,314,741 (95.1%) |
| `effort` | string | 880,466 of 2,314,741 (38.0%) |
| `agentId` *(ambient)* | string | 856,904 of 2,314,741 (37.0%) |
| `attributionAgent` | string | 800,615 of 2,314,741 (34.6%) |
| `attributionSkill` | string | 757,114 of 2,314,741 (32.7%) |
| `session_id` *(ambient)* | string | 260,880 of 2,314,741 (11.3%) |
| `isApiErrorMessage` | boolean | 13,707 of 2,314,741 (0.59%) |
| `error` | string | 6,999 of 2,314,741 (0.30%) |
| `apiErrorStatus` | number | 5,077 of 2,314,741 (0.22%) |
| `attributionPlugin` | string | 3,913 of 2,314,741 (0.17%) |
| `apiErrorIsTransient` | boolean | 178 of 2,314,741 (0.008%) |
| `errorDetails` | string | 117 of 2,314,741 (0.005%) |
| `attributionMcpServer` | string | 24 of 2,314,741 (0.001%) |
| `attributionMcpTool` | string | 24 of 2,314,741 (0.001%) |
| `isAbortedMidStream` | boolean | 1 of 2,314,741 (0.000%) |

Fields marked *(ambient)* are Class A envelope decorations that appear across several kinds rather than payload specific to this one. The nine common fields are omitted from the table; Class A membership fixes which of them are present — see [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md).

Captured example:

```json
{
  "parentUuid": "e2713364-5250-4262-b27a-a4d4a4e50b5a",
  "isSidechain": false,
  "type": "assistant",
  "uuid": "0d082884-8e5f-44a6-90ab-c23311df3f46",
  "timestamp": "2026-08-04T20:01:58.270Z",
  "message": {
    "id": "f130a673-0169-44e0-9045-8131aac33cd1",
    "container": null,
    "model": "<synthetic>",
    "role": "assistant",
    "stop_details": null,
    "stop_reason": "stop_sequence",
    "stop_sequence": "",
    "type": "message",
    "usage": {
      "…": "…"
    }
  },
  "requestId": "req_011CdiJd5HHmKZhN9iLW2su9",
  "error": "invalid_request",
  "errorDetails": "400 {\"type\":\"error\",\"error\":{\"type\":\"invalid_request_error\",\"message\":\"prompt is too lo…"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`message` is the raw API response object**, not a flattened string. Its `content` is an array of blocks — text, thinking, tool_use — each specified in [`../jsonl/`](../jsonl/readme.md). A consumer that reads `message.content` as a string loses every non-text block.

**`requestId` correlates a turn to its API call.** Present on 99.0% of lines; absent on locally-synthesized turns that never hit the API.

**Error turns are still `assistant` lines.** `isApiErrorMessage` marks a turn synthesized from a failure rather than returned by the model, with `apiErrorStatus`, `errorDetails`, and `apiErrorIsTransient` carrying the detail. A consumer counting model outputs must exclude these or it will count failures as responses.

**The attribution chain is optional and layered.** `attributionSkill`, `attributionAgent`, `attributionPlugin`, `attributionMcpServer`, and `attributionMcpTool` each appear only when that layer was responsible. `agentId` marks a turn produced inside a subagent rather than the main thread.

**`effort` records reasoning effort** on the turns where it was set — roughly a third of all assistant lines. Its absence is not a default value.

### Since

Observed 2.0.56 – 2.1.220 (20 distinct versions) — the full range present in the sampled store, so the floor is a sampling artifact and not a claim about introduction.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this kind satisfies |
| jsonl | [`../jsonl/003_assistant_entry.md`](../jsonl/003_assistant_entry.md) | Internal field detail of the assistant entry |
| jsonl | [`../jsonl/008_usage_object.md`](../jsonl/008_usage_object.md) | `message.usage` token accounting |
| behavior | [`../behavior/031_b31_subagent_tool_sets.md`](../behavior/031_b31_subagent_tool_sets.md) | Subagent tool sets underlying `agentId` turns |
