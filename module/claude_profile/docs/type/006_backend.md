# Type: Backend

### Scope

- **Purpose**: Define Backend — the closed enum selecting an account's credential and traffic mechanism.
- **Responsibility**: Documents the two values, their behavioral split, and parse resilience.
- **In Scope**: Enum values, per-value contract summary, unknown-value handling.
- **Out of Scope**: Redirect env-var mechanics (→ [feature/071](../feature/071_redirect_backend_accounts.md)); CLI parameter surface (→ [cli/type/005_account_backend.md](../cli/type/005_account_backend.md), the CLI-layer projection of this type).

### Definition

Closed enum with exactly two values, determining how an [Account (001)](001_account.md) authenticates and where its traffic goes:

| Value | Credential capture at save | Switch-time contract |
|-------|---------------------------|----------------------|
| `anthropic` (default) | Live OAuth session snapshot | Restores OAuth credentials; removes all redirect env vars |
| `redirect` | `api_key::` payload verbatim | Writes `ANTHROPIC_BASE_URL`/`AUTH_TOKEN`/`MODEL` env vars ([feature/071](../feature/071_redirect_backend_accounts.md)); plus provider-specific tiers ([feature/073](../feature/073_kimi_provider_preset.md)) |

### Validation

- Only `anthropic` and `redirect` are accepted from explicit `backend::` input.
- Parse resilience: a missing or unrecognized stored value is treated as `anthropic` — reading never fails on legacy or hand-edited files.
- `redirect` imposes save-time requirements on the aggregate (see [Account (001)](001_account.md) Validation).

### Relationships

Gates which save path runs (OAuth capture vs key write); resolved value gates whether [Preset (007)](007_preset.md) redirect defaults apply; determines the name-validation rule split on [Account (001)](001_account.md).

### Serialization

`backend` string field in `{name}.json` ([schema/002](../schema/002_account_json.md)); absent field ≡ `anthropic`.
