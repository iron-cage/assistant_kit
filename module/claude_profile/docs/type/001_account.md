# Type: Account

### Scope

- **Purpose**: Define the Account aggregate — the central Domain Type all `clp` operations revolve around.
- **Responsibility**: Documents Account identity rules, owned attributes, construction, and validation; delegates attribute-specific semantics to the owning type/schema/feature docs.
- **In Scope**: Aggregate identity, owned fields, construction paths, name validation split by backend.
- **Out of Scope**: On-disk layout (→ [schema/002](../schema/002_account_json.md), [schema/001](../schema/001_credentials_json.md)); lifecycle transitions (→ [state_machine/001](../state_machine/001_account_lifecycle.md)); switching mechanics (→ [feature/004](../feature/004_account_use.md)).

### Definition

A named credential profile. Identity is the unique `name` within the credential store — for `anthropic`-backend accounts the name is the account's email address; for `redirect`-backend accounts it is an arbitrary caller-chosen label (e.g. `kimi_k3`). The aggregate owns:

| Attribute | Type instance / doc |
|-----------|---------------------|
| `backend` | [Backend (006)](006_backend.md) |
| `inference_provider` | [Provider (005)](005_provider.md) |
| `tags` | [Tag (003)](003_tag.md) |
| credential payload | [Token (009)](009_token.md) |
| quota measurement | [Quota Snapshot (008)](008_quota_snapshot.md) |
| `owner`, `claim_lock`, `reserve` | [Identity (002)](002_identity.md), [feature/070](../feature/070_account_claim_and_reservation_control.md) |
| `base_url`, `redirect_model` | [feature/071](../feature/071_redirect_backend_accounts.md) |
| `role` (superseded) | folded into Tag — see [003](003_tag.md) migration note |

Constructed by `.account.save`; mutated by ownership/claim/renewal operations; destroyed by `.account.delete`.

### Validation

- `name` must be non-empty and unique in the store; save over an existing name replaces that account's snapshot.
- `anthropic`-backend names must be email-shaped; `redirect`-backend names are exempt (arbitrary labels permitted).
- `redirect` backend requires `api_key::` and `redirect_model::` at save; `base_url` must be present (explicit or preset-filled).
- Aggregate consistency: attribute writes go through the store's read-merge helpers — partial writes never drop sibling fields.

### Relationships

Selected for rotation only after passing all eligibility gates ([algorithm/004](../algorithm/004_eligibility_gates.md)); listed by `.accounts`; one Account per Identity may be "current" per machine via the active marker ([schema/005](../schema/005_active_marker.md)).

### Serialization

`{name}.json` (metadata) + `{name}.credentials.json` (secret payload) in the credential store — formats owned by [schema/002](../schema/002_account_json.md) and [schema/001](../schema/001_credentials_json.md).
