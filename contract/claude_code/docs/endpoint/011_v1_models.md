# Endpoint: GET /v1/models

### Scope

- **Purpose**: Wire contract for the Anthropic `GET /v1/models` endpoint that returns the list of available Claude API models.
- **Responsibility**: Documents this endpoint's request shape, response schema, pagination model, and OAuth compatibility notes for workspace callers implementing a live model listing command.
- **In Scope**: URL, auth headers, query parameters, full response JSON schema, error codes, pagination cursor fields.
- **Out of Scope**: Model capability details (→ `../model/readme.md`); workspace model selection policy (→ `../model/012_workspace_defaults.md`); OAuth token acquisition (→ `004_oauth_token.md`); Bedrock/Vertex model IDs.

### Request

**URL**: `https://api.anthropic.com/v1/models`

**Method**: GET

**Auth note**: Anthropic's public docs show `X-Api-Key` for this endpoint. Workspace callers use OAuth bearer tokens — the same pattern as endpoint 003 (`POST /v1/messages`). OAuth compatibility inferred from shared bearer token mechanism; verify via live test before implementing a workspace caller.

**Headers**:

| Header | Value | Required |
|--------|-------|----------|
| `anthropic-version` | `2023-06-01` | yes |
| `Authorization` | `Bearer {oauth_access_token}` | yes (OAuth) |
| `anthropic-beta` | `oauth-2025-04-20` | yes (OAuth) |

**Query Parameters** (all optional):

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `after_id` | string | — | — | Cursor: return page immediately after this model ID |
| `before_id` | string | — | — | Cursor: return page immediately before this model ID |
| `limit` | integer | 20 | 1–1000 | Items per page |

### Response

**HTTP 200 — response body**:

| Field | Type | Description |
|-------|------|-------------|
| `data` | array of ModelInfo | Ordered list of models; most recently released first |
| `has_more` | boolean | True when additional pages exist in the requested direction |
| `first_id` | string | ID of the first item in `data`; use as `before_id` for previous page |
| `last_id` | string | ID of the last item in `data`; use as `after_id` for next page |

**ModelInfo object** (`data[]` items):

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Pinned model ID (e.g. `claude-opus-4-8`) |
| `type` | `"model"` | Always `"model"` |
| `display_name` | string | Human-readable name (e.g. `"Claude Opus 4.8"`) |
| `created_at` | RFC 3339 | Release datetime; may be epoch if release date unknown |
| `max_input_tokens` | integer | Maximum context window in tokens |
| `max_tokens` | integer | Maximum `max_tokens` value for this model |
| `capabilities` | ModelCapabilities | Per-capability support flags (see below) |

**ModelCapabilities object**:

| Field | Type | Description |
|-------|------|-------------|
| `batch.supported` | boolean | Supports the Batch API |
| `citations.supported` | boolean | Supports citation generation |
| `code_execution.supported` | boolean | Supports code execution tools |
| `context_management.supported` | boolean | Supports context management |
| `context_management.compact_20260112.supported` | boolean | Supports compact context management strategy |
| `effort.supported` | boolean | Supports the `effort` parameter |
| `effort.low/medium/high/max/xhigh.supported` | boolean | Supported effort levels |
| `image_input.supported` | boolean | Accepts image content blocks |
| `pdf_input.supported` | boolean | Accepts PDF content blocks |
| `structured_outputs.supported` | boolean | Supports structured output / JSON mode |
| `thinking.supported` | boolean | Supports thinking |
| `thinking.types.adaptive.supported` | boolean | Supports adaptive (always-on) thinking |
| `thinking.types.enabled.supported` | boolean | Supports explicit extended thinking |

### Error Codes

| HTTP | Meaning | Action |
|------|---------|--------|
| 401 | Invalid or expired OAuth token | Refresh token via endpoint 004 |
| 403 | Insufficient OAuth scope | Check org permissions |
| 429 | Rate limited | Back off and retry |
| 5xx | Anthropic server error | Back off and retry |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master endpoint table |
| doc | [../model/readme.md](../model/readme.md) | Known model catalog with workspace usage |
| doc | [../model/012_workspace_defaults.md](../model/012_workspace_defaults.md) | Role-to-model assignment for workspace callers |
| doc | [003_v1_messages.md](003_v1_messages.md) | POST /v1/messages — same OAuth bearer token pattern |
| doc | [004_oauth_token.md](004_oauth_token.md) | OAuth token refresh — provides the bearer token |
