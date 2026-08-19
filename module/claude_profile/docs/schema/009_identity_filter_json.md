# Schema: Identity Tag Filter — `_filter_{host}_{user}`

### Scope

- **Purpose**: Define the per-Identity tag filter file format and naming convention.
- **Responsibility**: Documents the filter filename derivation, JSON shape, ordering guarantees, and absent-file semantics.
- **In Scope**: Filename derivation, content format, write-time invariants, sync behavior vs. `_active_*` markers.
- **Out of Scope**: Tag value rules (→ [type/003](../type/003_tag.md)); filter predicate and gate placement (→ [type/004](../type/004_tag_filter.md), [algorithm/004](../algorithm/004_eligibility_gates.md) Gate 11); write/clear CLI surface (→ [feature/076](../feature/076_identity_tag_filter.md)).

### File Location

```
{credential_store}/_filter_{hostname}_{user}
```

One file per Identity (`user@host`). Filename derivation and sanitization are identical to the active marker's ([schema/005](005_active_marker.md)): same `hostname`/`user` resolution chain, same `keep [a-zA-Z0-9\-\.]; replace all other chars with '_'` sanitize.

Example: hostname `w003`, user `user1` → `_filter_w003_user1`

### Fields

JSON object with exactly two keys:

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `include` | array of strings | yes (may be empty) | Tags an account must **all** carry to be eligible for this Identity's automatic selection |
| `exclude` | array of strings | yes (may be empty) | Tags an account must carry **none** of |

### Content Format

```json
{
  "include": ["ci", "kimi_pool"],
  "exclude": ["personal"]
}
```

- Every element is a valid Tag per [type/003](../type/003_tag.md) (charset `[a-z0-9_-]`, 1–64 chars).
- Both arrays are deduplicated and sorted at write time (deterministic files, clean diffs).
- `include ∩ exclude` is empty — a contradictory pair is rejected at write time, never persisted.
- Unknown keys are rejected at write time; readers ignore them (forward tolerance).

### Absent-File Semantics

No file for an Identity ≡ permit-all (`include=[]`, `exclude=[]`): automatic selection behaves exactly as before this schema existed. Deleting the file (`.identity.filter clear::1`) is the canonical way to disable filtering for an Identity.

### Sync Behavior

Unlike `_active_*` markers (machine-local state, excluded via the `_active_*` `.gitignore` pattern — [schema/005](005_active_marker.md)), filter files are **meant to sync** with the credential store across machines: the `_filter_` prefix deliberately does not match the `_active_*` ignore pattern, so store-level sync (including a git-managed store) carries filters everywhere. This is what makes filters centrally administrable via `.identity.filter identity::USER@MACHINE`.

### Features

| File | Relationship |
|------|-------------|
| [feature/076_identity_tag_filter.md](../feature/076_identity_tag_filter.md) | Feature spec with acceptance criteria — write/read/clear surface, Gate 11 consumption |
| [feature/075_account_tags.md](../feature/075_account_tags.md) | Account-side `tags` field the filter's sets are evaluated against |

### Schema

| File | Relationship |
|------|-------------|
| [005_active_marker.md](005_active_marker.md) | Sibling per-Identity store file — source of the filename derivation and sanitization rules |
| [002_account_json.md](002_account_json.md) | `{name}.json` `tags` field — the account-side data the predicate reads |
| [004_storage_root.md](004_storage_root.md) | Credential store path (parent directory) |
