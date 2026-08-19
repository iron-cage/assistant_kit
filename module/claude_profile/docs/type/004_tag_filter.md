# Type: Tag Filter

### Scope

- **Purpose**: Define Tag Filter — the per-Identity include/exclude tag set pair that gates rotation eligibility.
- **Responsibility**: Documents the filter's structure, eligibility predicate, empty-set semantics, and write-time validation.
- **In Scope**: Structure, predicate, defaults, contradiction rejection, which selection paths it binds.
- **Out of Scope**: Tag value rules (→ [Tag (003)](003_tag.md)); gate ordering and force-bypass doctrine (→ [algorithm/004](../algorithm/004_eligibility_gates.md) Gate 11); file format (→ [schema/009](../schema/009_identity_filter_json.md)); CLI surface (→ [feature/076](../feature/076_identity_tag_filter.md)).

### Definition

A pair of tag sets owned by one [Identity (002)](002_identity.md):

- `include` — tags an account must **all** carry to be eligible
- `exclude` — tags an account must carry **none** of

Eligibility predicate over an account's tag set `T`: `T ⊇ include ∧ T ∩ exclude = ∅`.

Scope of enforcement: automatic selection only — rotation, auto-switch, and next-account recommendation. Explicit `.account.use name::X` is never filtered (naming an account is explicit intent), mirroring how provider mismatch (Gate 10) binds only the automatic path. No `force::1` bypass: a filter is a "which pool" concern, not a "who may act" concern — same doctrine as Gate 10.

### Validation

- Both sets contain valid Tags per [Tag (003)](003_tag.md) rules.
- `include ∩ exclude` must be empty — a contradictory filter is rejected at write time (exit 1 naming the overlapping tags).
- Empty `include` = no requirement; empty `exclude` = nothing blocked; absent filter (no file for the Identity) = permit-all — exactly today's behavior, giving a zero-migration adoption path.
- Untagged accounts fail any non-empty `include` (they lack the tags) — by design.
- Typo guard: a write whose `include` set matches zero currently-tagged accounts succeeds but must warn (a typo here silently empties the rotation pool — the primary operational hazard of this type).

### Relationships

Owned by [Identity (002)](002_identity.md); evaluated against [Account (001)](001_account.md) tag sets by rotation eligibility Gate 11 ([algorithm/004](../algorithm/004_eligibility_gates.md)); orthogonal to [Provider (005)](005_provider.md) selection — both gates apply independently. CLI get/set/clear surface: [feature/076](../feature/076_identity_tag_filter.md).

### Serialization

One JSON file per Identity in the credential store — `_filter_{hostname}_{user}`, sibling convention to the active marker ([schema/005](../schema/005_active_marker.md)): `{"include": [...], "exclude": [...]}`, sets sorted. Store-resident so filters sync across machines and are centrally administrable. Full format: [schema/009](../schema/009_identity_filter_json.md).
