# Type: Identity

### Scope

- **Purpose**: Define Identity — the `user@host` value naming one acting user seat on one machine.
- **Responsibility**: Documents Identity composition, derivation, equality, and the account-coordination attributes keyed by it.
- **In Scope**: Value format, component derivation and fallbacks, equality semantics, what Identity owns.
- **Out of Scope**: Ownership gate logic (→ [algorithm/004](../algorithm/004_eligibility_gates.md) Gate 8); ownership lifecycle (→ [state_machine/004](../state_machine/004_ownership_lifecycle.md)); marker file format (→ [schema/005](../schema/005_active_marker.md)).

### Definition

The pair `{user}@{hostname}`, produced by `current_identity()`. It names *who is acting from where* — two users on one host are distinct Identities, one user on two hosts likewise. Not a machine: the host component alone never identifies an actor. Immutable value; equality is exact string equality.

Component derivation (shared with the active-marker filename, [schema/005](../schema/005_active_marker.md)):

- `user`: `$USER` → `$USERNAME` → `"user"`
- `hostname`: `$HOSTNAME` → `/etc/hostname` → `"local"`

An Identity owns, per credential store:

| Owned attribute | Mechanism |
|-----------------|-----------|
| account ownership claims | `owner` field on [Account (001)](001_account.md) — empty or matching Identity ⇒ `is_owned = true` |
| current-account pointer | per-machine active marker file ([schema/005](../schema/005_active_marker.md)) |
| rotation tag filter (planned) | [Tag Filter (004)](004_tag_filter.md) |

### Validation

- Components are never empty — the fallback chain guarantees a value.
- For filename embedding, components are sanitized: characters outside `[a-zA-Z0-9\-\.]` are replaced with `_`.
- The un-sanitized form (stored in `owner` fields) is compared verbatim — sanitization applies only at the filename boundary.

### Relationships

Consumed by Gate 8 (Foreign-owned) in [algorithm/004](../algorithm/004_eligibility_gates.md); written as the default `owner` by `.account.save`; targeted by `.accounts assignee::USER@MACHINE`.

### Serialization

Plain string in `owner` fields of `{name}.json`; sanitized-embedded in `_active_{host}_{user}` filenames; key for the planned per-Identity tag filter file.
