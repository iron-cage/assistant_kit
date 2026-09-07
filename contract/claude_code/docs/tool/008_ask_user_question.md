# Tool: AskUserQuestion

Prompt user for input or clarification.

### Category

Interaction

### Description

Asks the user a question and waits for their response. Used when the agent needs additional information, confirmation, or clarification to proceed.

### Parameters

This doc previously had no Parameters section at all. v2.1.220's real schema is a structured multiple-choice form, not a bare free-text prompt:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `questions` | array (1-4 items) | ✅ | Questions to ask the user. Each item: `question` (string, the complete question), `header` (string, ≤12 chars, short chip/tag label), `options` (2-4 items, each `{label, description, preview?}` — no "Other" option needed, offered automatically), `multiSelect` (boolean, default `false` — allow selecting multiple options). |
| `annotations` | object | ❌ | Optional per-question free-text notes/preview content the user added to their selection, keyed by question text. |
| `metadata` | object | ❌ | Optional `{source}` field for tracking/analytics; not displayed to the user. |
| `answers` | object | — (response, not input) | User answers collected by the permission component; keyed by question text. |

**Verify:** `grep -ac -F "Very short label displayed as a chip" ~/.local/share/claude/versions/2.1.220` → 1; `grep -ac -F "Questions to ask the user"` → 1.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
