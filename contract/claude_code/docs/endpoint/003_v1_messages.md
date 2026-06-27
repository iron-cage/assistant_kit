# Endpoint: POST /v1/messages — Rate-Limit Header Side-Channel

### Scope

- **Purpose**: Extract per-account 5h/7d quota utilization from `anthropic-ratelimit-unified-*` response headers returned by the Anthropic inference endpoint.
- **Responsibility**: Wire contract for the rate-limit header side-channel: request format, required headers (including the versioned `anthropic-beta` value), response header names and value types, and the headers-on-all-responses behavior.
- **In Scope**: `anthropic-ratelimit-unified-*` header names, `anthropic-beta` version string, minimal request body, header-present-on-error-responses pattern.
- **Out of Scope**: Inference response body fields (model output, usage tokens, stop_reason); full messages API contract (→ Anthropic docs); OAuth usage JSON endpoint (→ `001_oauth_usage.md`).

### Request

```
POST https://api.anthropic.com/v1/messages
Authorization:     Bearer {access_token}
anthropic-beta:    oauth-2025-04-20
anthropic-version: 2023-06-01
Content-Type:      application/json

{"model":"claude-haiku-4-5-20251001","max_tokens":1,"messages":[{"role":"user","content":"quota"}]}
```

The request body is intentionally minimal — a single-token inference request. The response body is discarded; only response headers are consumed.

Workspace constants: `claude_quota::API_URL`, `ANTHROPIC_BETA`, `ANTHROPIC_VERSION`.

### `anthropic-beta` Is Required

Without the `anthropic-beta: oauth-2025-04-20` header, the API rejects the OAuth bearer token with HTTP 401 and the message "OAuth authentication is currently not supported". Rate-limit headers are absent on this error response.

The beta string is not documented in Anthropic's public API docs. It was discovered via:
```
strings $(which claude) | grep oauth
```

When Claude Code binary updates and live tests fail with auth errors, re-run this command to find the new value. Update `claude_quota::ANTHROPIC_BETA`.

### Response Headers

Rate-limit headers are present on **all** responses — including HTTP 4xx and 5xx. They must be read from both success (2xx) and error response objects.

| Header | Type | Range | Semantics |
|--------|------|-------|-----------|
| `anthropic-ratelimit-unified-5h-utilization` | f64 string | 0.0–1.0 | 5-hour session quota consumed (fraction) |
| `anthropic-ratelimit-unified-5h-reset` | u64 string | Unix seconds | When the 5-hour window resets |
| `anthropic-ratelimit-unified-7d-utilization` | f64 string | 0.0–1.0 | 7-day all-model quota consumed (fraction) |
| `anthropic-ratelimit-unified-7d-reset` | u64 string | Unix seconds | When the 7-day window resets |
| `anthropic-ratelimit-unified-status` | string | — | `"allowed"`, `"allowed_warning"`, `"rejected"` |

**Scale difference vs `/api/oauth/usage`**: Utilization here is 0.0–1.0 (fraction). The JSON endpoint (`001_oauth_usage.md`) returns 0.0–100.0 (percent). Convert: `remaining_pct = (1.0 - header_utilization) * 100`.

### Example Response Headers

HTTP 200 (alice@example.com, example response):

```
HTTP/1.1 200 OK
anthropic-ratelimit-unified-5h-utilization: 0.34
anthropic-ratelimit-unified-5h-reset: 1748001000
anthropic-ratelimit-unified-7d-utilization: 0.32
anthropic-ratelimit-unified-7d-reset: 1748131200
anthropic-ratelimit-unified-status: allowed
```

Response body (discarded by `fetch_rate_limits` — only headers are consumed):

```json
{"id":"msg_01...","type":"message","role":"assistant","content":[{"type":"text","text":"quota"}],"model":"claude-haiku-4-5-20251001","stop_reason":"end_turn","usage":{"input_tokens":10,"output_tokens":5}}
```

### Headers-on-Error Pattern

```rust
// ureq: extract response from both Ok and Err(Status(_, r)) variants
let resp = match req_result
{
  Ok( r ) | Err( ureq::Error::Status( _, r ) ) => r,
  Err( e ) => return Err( QuotaError::HttpTransport( e.to_string() ) ),
};
parse_headers( |name| resp.header( name ).map( str::to_string ) )
```

Network failures (`ureq::Error::Transport`) propagate as `QuotaError::HttpTransport`. HTTP status code failures (4xx, 5xx) still provide a response object with headers.

### Error Codes

| HTTP | Rate-limit headers present | Meaning |
|------|---------------------------|---------|
| 200 | Yes | Success |
| 400 | Yes | Malformed request body |
| 401 | No (when beta header absent) | OAuth not enabled — update `ANTHROPIC_BETA` |
| 401/403 | Yes | Token invalid — refresh required |
| 429 | Yes | Request rate limited |
| 5xx | Yes | Server error |

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../../../module/claude_quota/src/lib.rs` | `fetch_rate_limits`, `parse_headers`, `API_URL`, `ANTHROPIC_BETA`, `ANTHROPIC_VERSION` |
| doc | `../../../../module/claude_profile/docs/feature/013_account_limits.md` | `.account.limits` command — consumer of this endpoint |
| doc | [001_oauth_usage.md](001_oauth_usage.md) | Alternative quota source via GET JSON (no inference request, no `anthropic-beta` required) |
| doc | [002_oauth_account.md](002_oauth_account.md) | Account identity endpoint — also references this endpoint for rate-limit headers |
| doc | [../formats/007_json_response.md](../formats/007_json_response.md) | JSON response format produced by the upstream Messages API |
