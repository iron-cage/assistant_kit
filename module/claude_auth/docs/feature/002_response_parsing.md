# Feature: Response Parsing

### Scope

- **Purpose**: Specify how a token-refresh JSON response body is turned into a `TokenRefreshResult` without a JSON library.
- **Responsibility**: Define the required fields, the needle-matching strategy, type rejection rules, and error attribution.
- **In Scope**: Required fields (FR-1), needle construction (FR-2), string extraction (FR-3), integer extraction (FR-4, FR-5), error attribution (FR-6), availability (FR-7).
- **Out of Scope**: The network exchange that produces the body (→ `001_token_refresh.md`), signature contracts (→ `api/001_auth_surface.md`).

### Design

**Required fields:**

| ID | Requirement |
|----|-------------|
| FR-1 | Parsing requires exactly three fields — `access_token` (string), `refresh_token` (string), `expires_in` (unsigned integer). Any other field in the body is ignored. |

**Needle construction — the prefix-collision guard:**

| ID | Requirement |
|----|-------------|
| FR-2 | The search needle for a key is `"key":` **with the colon included**, never the bare key name |

This is the single subtlest rule in the crate. Anthropic's response contains both
`"refresh_token"` and `"token_type"`, and a consumer body may carry `"token"` alongside
`"access_token"`. Searching for a bare `"token"` would match inside `"access_token"` and
extract the wrong value with no error raised. Including the colon anchors the match to a
complete key immediately followed by its separator, so no key can match as a substring of a
longer one.

**String extraction:**

| ID | Requirement |
|----|-------------|
| FR-3 | After the needle, leading whitespace is skipped; the value must then open with `"`, and the extracted value runs to the next `"` |

A value not opening with `"` is a type error, not a missing field — but both are reported the
same way (see FR-6). No escape processing is performed: OAuth tokens are base64/URL-safe
alphabets, so a `\"` inside one would itself be the anomaly.

**Integer extraction:**

| ID | Requirement |
|----|-------------|
| FR-4 | A value opening with `"` is rejected — a quoted number is a type error, not a number |
| FR-5 | Otherwise leading ASCII digits are collected up to the first non-digit and parsed as `u64`; a failed parse is an error |

FR-4 exists because a permissive parser that stripped quotes would silently accept
`"expires_in": "3600"`, and the resulting expiry would be indistinguishable from a correct one.
Rejecting it surfaces a server contract change immediately.

**Error attribution:**

| ID | Requirement |
|----|-------------|
| FR-6 | Every parse failure returns `AuthError::ResponseParse( field )` naming the specific field that failed, never a generic message |

Absent field, wrong type, and unparseable digits all collapse to the same variant carrying the
same field name. The distinction is deliberately not modelled: a caller's response to any of
them is identical — the response is unusable — and the field name is what makes the failure
diagnosable from a log line.

**Availability:**

| ID | Requirement |
|----|-------------|
| FR-7 | `parse_response` is available with no feature enabled, and performs no I/O |

`now_ms` is a parameter rather than a clock read precisely so that this holds — see
[invariant/002_offline_parse_core.md](../invariant/002_offline_parse_core.md).

### Acceptance Criteria

Fully covered offline by `tests/auth_test.rs`:

| Test | Covers | Scenario |
|------|--------|----------|
| T01 | FR-1, FR-3, FR-5 | Valid body, all three fields present, fields and computed expiry correct |
| T02 | FR-6 | `access_token` absent → `ResponseParse("access_token")` |
| T03 | FR-6 | `refresh_token` absent → `ResponseParse("refresh_token")` |
| T04 | FR-6 | `expires_in` absent → `ResponseParse("expires_in")` |
| T05 | FR-4 | `"expires_in": "bad"` — quoted, not integer → `ResponseParse` |

```bash
cargo nextest run -p claude_auth
```

**Known coverage gap:** FR-2's prefix-collision guard has no dedicated regression test. T01–T05
would all still pass if the colon were dropped from the needle, because no fixture body
contains a key that is a proper prefix of another. A body carrying both `"token"` and
`"access_token"` would pin the behavior; until one exists, FR-2 is held by review only.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [001_token_refresh.md](001_token_refresh.md) | The network exchange that produces the body parsed here |
| doc | [invariant/002_offline_parse_core.md](../invariant/002_offline_parse_core.md) | Constraint keeping this feature network-free and always available |
| doc | [api/001_auth_surface.md](../api/001_auth_surface.md) | Signature and error contract for `parse_response` |
| source | `../../src/lib.rs` | `parse_response`, `parse_string_field`, `parse_u64_field` |
| test | `../../tests/auth_test.rs` | T01–T05 |
