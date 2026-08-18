# Type: Token

### Scope

- **Purpose**: Define Token — the per-account credential payload, split by backend kind.
- **Responsibility**: Documents the two payload kinds, their lifecycle difference, and identity.
- **In Scope**: Payload kinds, refresh applicability, expiry semantics.
- **Out of Scope**: OAuth refresh state machine (→ [state_machine/002](../state_machine/002_oauth_token_lifecycle.md)); refresh invocation (→ [subprocess/003](../subprocess/003_token_refresh_invocation.md)); file format (→ [schema/001](../schema/001_credentials_json.md)).

### Definition

The secret credential material belonging to one [Account (001)](001_account.md). Identity is the owning account; state is mutable (refreshed, re-captured, expired) — an entity, not a value. Two kinds by [Backend (006)](006_backend.md):

| Backend | Payload | Lifecycle |
|---------|---------|-----------|
| `anthropic` | OAuth access token + refresh token + expiry | issued → refreshed (repeatedly) → expired; refresh via isolated subprocess ([subprocess/003](../subprocess/003_token_refresh_invocation.md)) |
| `redirect` | Foreign API key stored as `accessToken` | static — never refreshed; "expiry" not applicable (displayed as expired/N-A in listings) |

At switch time the payload is written into live credentials (anthropic) or into `ANTHROPIC_AUTH_TOKEN` (redirect).

### Validation

- Payload must be non-empty at save (`api_key::` non-empty for redirect; a live OAuth session present for anthropic capture).
- Expiry is epoch-milliseconds; comparison against now decides Gate 6 (Expired) eligibility ([algorithm/004](../algorithm/004_eligibility_gates.md)).
- Secret hygiene: token values are never echoed verbatim by CLI output — display paths redact (length-only).

### Relationships

Owned by [Account (001)](001_account.md); expiry consumed by eligibility Gate 6; status surfaced by `.credentials.status`.

### Serialization

`{name}.credentials.json` in the credential store ([schema/001](../schema/001_credentials_json.md)).
