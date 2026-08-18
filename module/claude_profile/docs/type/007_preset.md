# Type: Preset

### Scope

- **Purpose**: Define Preset — a named bundle of save-time defaults for a known foreign provider.
- **Responsibility**: Documents the value's recognized names, what a preset may and may not default, and its ephemeral nature.
- **In Scope**: Recognized values, default-filling boundary, gating rule, case handling.
- **Out of Scope**: The Kimi bundle's full contract (→ [feature/073](../feature/073_kimi_provider_preset.md)); parameter surface (→ [cli/param/074_preset.md](../cli/param/074_preset.md)).

### Definition

A save-time-only convenience value naming a bundle of defaults for `backend`/`base_url`/`inference_provider`. Exactly one recognized value today: `kimi` (Moonshot). Deliberately not a provider registry — each new preset is an explicit design addition, not a data entry.

A preset only ever fills values structurally identical for every account of that provider; genuinely per-account values (`api_key::`, `redirect_model::`) are never defaulted. Redirect-only defaults apply only when the *resolved* backend is `redirect` — `preset::kimi backend::anthropic` behaves as if no preset were given.

Ephemeral: consumed entirely during `.account.save`; no runtime existence afterward — nothing is persisted under the preset's name.

### Validation

- Recognized values only (`kimi`, matched case-insensitively); any other non-empty value exits 1 naming the valid set.
- Explicit `backend::`/`base_url::`/`inference_provider::` always override the preset's defaults.

### Relationships

Pre-fills [Backend (006)](006_backend.md) and [Provider (005)](005_provider.md) on [Account (001)](001_account.md) construction; full Kimi semantics in [feature/073](../feature/073_kimi_provider_preset.md).

### Serialization

None — not persisted (see Definition).
