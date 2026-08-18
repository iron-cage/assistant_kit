# Type: Provider

### Scope

- **Purpose**: Define Provider — the inference provider identifier used in two distinct positions: per-account origin tag and machine-global selection.
- **Responsibility**: Documents the value's format, its two usage positions, the effective-default rule, and the boundary against Tag.
- **In Scope**: Value semantics, the two positions, empty-value defaulting, selection mechanics.
- **Out of Scope**: Gate 10 enforcement details (→ [algorithm/004](../algorithm/004_eligibility_gates.md)); Kimi behavioral contract (→ [feature/073](../feature/073_kimi_provider_preset.md)); selection command surface (→ [cli/command/009_provider.md](../cli/command/009_provider.md)).

### Definition

A free-form provider identifier (`anthropic`, `kimi`, `zhipu`, …) appearing in two positions:

1. **Account attribute** (`inference_provider` in `{name}.json`): records which provider the account's credentials belong to. Written verbatim at save — no auto-detection.
2. **Machine-global selection** (`provider` key in `~/.clr/config.toml`, written solely by `.provider.select`): the single active provider for automatic selection on this machine; default `anthropic`.

Rotation only ever selects accounts whose effective provider equals the selected provider (Gate 10 — unconditional, no `force::1` bypass): crossing a provider boundary means switching billing/auth context, which must never happen silently.

Provider is *behavioral*, unlike [Tag (003)](003_tag.md) which is purely selective: `inference_provider == "kimi"` triggers the 7 Kimi-tier env vars at switch time ([feature/073](../feature/073_kimi_provider_preset.md)). The two mechanisms are deliberately orthogonal — tags partition pools within/across whatever provider is selected.

### Validation

- When given explicitly at save, `inference_provider::` must be non-empty.
- Absent account field and absent selection both default to `anthropic` — an untagged account and a default machine agree (Gate 10 comparison uses these effective values).
- No closed value list: unknown providers are permitted (they simply form their own rotation pool and receive no provider-specific switch behavior).

### Relationships

Carried by [Account (001)](001_account.md); pre-filled by [Preset (007)](007_preset.md); gate input in [algorithm/004](../algorithm/004_eligibility_gates.md); tagging vocabulary owner for Gate 10 rotation grouping ([feature/072](../feature/072_inference_provider_selection.md)).

### Serialization

`inference_provider` string field in `{name}.json` ([schema/002](../schema/002_account_json.md)); `provider` key in `~/.clr/config.toml` ([schema/008](../schema/008_clr_prefs_json.md)).
