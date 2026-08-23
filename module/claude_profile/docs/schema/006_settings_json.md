# Schema: Session Settings — `~/.claude/settings.json`

### Scope

- **Purpose**: Define which fields in `~/.claude/settings.json` are read or written by `clp`, their semantics, and the write callers.
- **Responsibility**: Documents the `settings.json` fields that `clp` reads or writes and their write rules.
- **In Scope**: All `settings.json` fields that `clp` touches — `model`, `effortLevel`, and (redirect-backend accounts only) the `env.ANTHROPIC_BASE_URL` / `env.ANTHROPIC_AUTH_TOKEN` / `env.ANTHROPIC_MODEL` sub-keys, plus (redirect-backend accounts tagged `inference_provider: "kimi"` only) 7 additional Kimi-tier sub-keys: `env.ANTHROPIC_DEFAULT_OPUS_MODEL` / `_SONNET_MODEL` / `_HAIKU_MODEL` / `_FABLE_MODEL`, `env.CLAUDE_CODE_SUBAGENT_MODEL`, `env.CLAUDE_CODE_EFFORT_LEVEL`, `env.CLAUDE_CODE_AUTO_COMPACT_WINDOW`, plus (redirect-backend accounts tagged `inference_provider: "deepseek"` only) 6 additional DeepSeek-tier sub-keys sharing 4 of those same key names (`env.ANTHROPIC_DEFAULT_OPUS_MODEL` / `_SONNET_MODEL` / `_HAIKU_MODEL`, `env.CLAUDE_CODE_SUBAGENT_MODEL`, `env.CLAUDE_CODE_EFFORT_LEVEL`, `env.CLAUDE_CODE_AUTO_COMPACT_WINDOW` — no `_FABLE_MODEL`) with independently gated values (Feature 078). All other fields, including any other `env` sub-key, are owned by the Claude binary and must never be modified.
- **Out of Scope**: Full `settings.json` schema (not owned by clp); CLI rendering of these values.

### File Location

```
~/.claude/settings.json
```

Path via `ClaudePaths::settings_file()`. See [schema/003](003_file_topology.md).

### Format

Single-level JSON object (hand-rolled formatter in `settings_io.rs`, not `serde_json::to_string_pretty`). The `json_serialize_flat_object` formatter is exempt from the invariant/007 `to_string_pretty` rule because it already produces equivalent pretty output. Its `infer_type()` classifies any value starting with `{`/`[` as `StoredAs::Raw` and emits it verbatim rather than quoting it — nested objects (like `env`) already round-trip through this "flat" formatter unchanged; this is pre-existing general behavior, not something Feature 071 added.

### Fields Managed by `clp`

| Field | Type | Default | Semantics | Written by | Read by |
|-------|------|---------|-----------|-----------|---------|
| `model` | string or absent | absent | Session model shorthand (`"sonnet"`, `"opus"`, `"haiku"`, or full model ID). Controls which Claude model is used for interactive sessions. | `set_session_model()`, `switch_account()` (restores from `{name}.json` for anthropic accounts; REMOVES the key when switching to a `backend: "redirect"` account — a stale pin must not shadow `env.ANTHROPIC_MODEL`), `.model set::`, `.account.use set_model::`, `apply_model_override()` (Fix BUG-311: bidirectional), `set_session_effort()` init path | `get_session_model()`, `.usage`/`.accounts` `model::1`, `recommended_model()` in `format.rs` |
| `effortLevel` | string or absent | absent → initialized by `apply_model_override()` on first use | Effort level for interactive sessions (`"low"`, `"normal"`, `"high"`, `"max"`). Controls extended thinking depth. | `apply_model_override()` writes unconditionally: `"max"` for Opus, `"high"` for Sonnet/absent-tier (TSK-335); BUG-312 fallback `"high"` retained as unreachable safety net | `get_session_effort()` — footer `Current` line in `.usage`; `Next` line uses model-derived effort from `recommended_model()` |
| `env.ANTHROPIC_BASE_URL` | string or absent | absent | Redirect target base URL, read natively by the Claude binary at process startup to route all API traffic to a foreign endpoint instead of `api.anthropic.com`. | `switch_account()` — written when switching TO a `backend: "redirect"` account, from that account's `base_url` (see [schema/002](../schema/002_account_json.md)) | Claude binary itself (not read by `clp`) |
| `env.ANTHROPIC_AUTH_TOKEN` | string or absent | absent | Static API key sent as the auth credential for the redirect target. | `switch_account()` — written when switching TO a `backend: "redirect"` account, from that account's `accessToken` (see [schema/001](../schema/001_credentials_json.md)) | Claude binary itself (not read by `clp`) |
| `env.ANTHROPIC_MODEL` | string or absent | absent | Model identifier string sent to the redirect target (e.g. a Moonshot Kimi model ID) — the foreign backend's own model catalog, unrelated to `model`'s Anthropic shorthand. | `switch_account()` — written when switching TO a `backend: "redirect"` account, from that account's `redirect_model` (see [schema/002](../schema/002_account_json.md)) | Claude binary itself (not read by `clp`) |
| `env.ANTHROPIC_DEFAULT_OPUS_MODEL` / `env.ANTHROPIC_DEFAULT_SONNET_MODEL` | string or absent | absent | Substitutes the redirect account's own model ID for the Opus/Sonnet Anthropic tiers — so a hardcoded `claude-opus-4-8`/etc. reference resolves to the foreign model instead. | `switch_account()` — written (both, identical value, mirroring that account's `redirect_model`) when switching TO a `backend: "redirect"` account tagged `inference_provider: "kimi"` **or** `inference_provider: "deepseek"` (same mirror rule for both providers) | Claude binary itself (not read by `clp`) |
| `env.ANTHROPIC_DEFAULT_HAIKU_MODEL` | string or absent | absent | Substitutes a model ID for the Haiku Anthropic tier. | `switch_account()` — for `inference_provider: "kimi"`, mirrors `redirect_model` (same value as Opus/Sonnet); for `inference_provider: "deepseek"`, fixed to the literal `"deepseek-v4-flash"` regardless of `redirect_model` (Feature 078) | Claude binary itself (not read by `clp`) |
| `env.ANTHROPIC_DEFAULT_FABLE_MODEL` | string or absent | absent | Substitutes a model ID for the Fable Anthropic tier. | `switch_account()` — written only for `inference_provider: "kimi"`, mirroring `redirect_model`; DeepSeek's own integration guide documents no Fable-tier mapping, so `inference_provider: "deepseek"` never writes this key (Feature 078) | Claude binary itself (not read by `clp`) |
| `env.CLAUDE_CODE_SUBAGENT_MODEL` | string or absent | absent | Model identifier used for subagent dispatch. | `switch_account()` — for `inference_provider: "kimi"`, same condition and mirrored value as the 4 tier vars above; for `inference_provider: "deepseek"`, fixed to `"deepseek-v4-flash"` (same rule as `ANTHROPIC_DEFAULT_HAIKU_MODEL`, Feature 078) | Claude binary itself (not read by `clp`) |
| `env.CLAUDE_CODE_EFFORT_LEVEL` | string or absent | absent | Reasoning-effort level requested from the redirect target; always the fixed string `"max"` for both a kimi and a deepseek redirect account — distinct from `effortLevel` (this schema's own top-level field), which tracks Anthropic-backend session effort and is never touched by this write. | `switch_account()` — written (fixed `"max"`) under the same condition as the tier vars above, for either provider | Claude binary itself (not read by `clp`) |
| `env.CLAUDE_CODE_AUTO_COMPACT_WINDOW` | string or absent | absent | Context-window token count at which Claude Code auto-compacts the conversation. | `switch_account()` — written under the same condition as the tier vars above; for `inference_provider: "kimi"`: `"1048576"` when `redirect_model` starts with `kimi-k3`, else `"262144"`; for `inference_provider: "deepseek"`: always the fixed `"786432"`, no model-name branching (Feature 078) | Claude binary itself (not read by `clp`) |

### Write Rules

- `clp` reads the entire `settings.json` into memory, modifies only `model`, `effortLevel`, or (redirect-backend switch only) the `env` object, and writes it back via `json_serialize_flat_object` — all other fields are preserved.
- Never `serde_json::to_string` — the hand-rolled formatter already produces pretty output.
- `env` is the one nested-object field among `clp`'s managed keys (all others are top-level strings), but no formatter change was needed to support it: `set_env_var()`/`remove_env_var()` (`claude_core::settings_io`, pre-existing — built for the `DISABLE_AUTOUPDATER`/`DISABLE_UPDATES` auto-updater toggles) already perform the nested read-modify-write, and `switch_account()` reuses them directly (see [feature/071](../feature/071_redirect_backend_accounts.md)).

### Redirect Backend Environment Override (Feature 071)

`switch_account()` maintains `env.ANTHROPIC_BASE_URL` / `env.ANTHROPIC_AUTH_TOKEN` / `env.ANTHROPIC_MODEL` as a unit, keyed off the target account's `backend`:

- **Switching TO a `backend: "redirect"` account:** write all three sub-keys from that account's `base_url` / `accessToken` / `redirect_model`. If an `env` object already exists in `settings.json` with other sub-keys (unrelated to `clp`), those other sub-keys are preserved — only the three named keys are set.
- **Switching TO a `backend: "anthropic"` account:** remove exactly the three named sub-keys from `env` (if present) so the Claude binary reverts to its own OAuth-based Anthropic routing; if `env` becomes empty as a result, remove the `env` key entirely; if `env` has other unrelated sub-keys remaining, keep `env` with those keys intact.
- `apply_model_override()` and `set_session_model()`/`get_session_model()` never touch `env` — they operate exclusively on the top-level `model`/`effortLevel` keys (see [algorithm/002](../algorithm/002_session_model_override.md)'s redirect bypass).

### Kimi-Tier Model Environment Overrides (Feature 073)

`switch_account()` extends the Feature 071 write/clear unit above with 7 more `env.*` sub-keys, gated on a narrower condition than `backend` alone: the target account must be `backend: "redirect"` **and** carry `inference_provider: "kimi"` (Feature 072's field, exact match — any other value or an absent field gets the base 3 only, never these 7).

- **Switching TO a `backend: "redirect"` account with `inference_provider: "kimi"`:** write all 7 Kimi-tier sub-keys alongside the base 3: `ANTHROPIC_DEFAULT_OPUS_MODEL`, `ANTHROPIC_DEFAULT_SONNET_MODEL`, `ANTHROPIC_DEFAULT_HAIKU_MODEL`, `ANTHROPIC_DEFAULT_FABLE_MODEL`, and `CLAUDE_CODE_SUBAGENT_MODEL` each set to that account's `redirect_model` value (identical to `ANTHROPIC_MODEL`); `CLAUDE_CODE_EFFORT_LEVEL` set to the fixed string `"max"`; `CLAUDE_CODE_AUTO_COMPACT_WINDOW` set to `"1048576"` when `redirect_model` starts with `kimi-k3`, else `"262144"`.
- **Switching TO a `backend: "redirect"` account whose `inference_provider` is not `"kimi"`:** write only the base 3 sub-keys (Feature 071 behavior, unchanged); the 7 Kimi-tier sub-keys are actively removed if present from a prior switch (see next bullet) — this account gets no stale Kimi-tier state left behind.
- **Switching TO a `backend: "anthropic"` account, or to a non-kimi `backend: "redirect"` account:** remove all 7 Kimi-tier sub-keys in addition to whatever the base-3 rule above already removes or leaves in place; `env` is pruned entirely only once every sub-key (base and Kimi-tier) is gone. This includes switching directly to a `backend: "redirect"` account tagged `inference_provider: "deepseek"` — the 7 Kimi-tier sub-keys are removed in the same call that writes the 6 DeepSeek-tier sub-keys below (Feature 078).
- **Compact-window sizing rationale:** a too-large window risks a genuine context-overflow failure on a smaller-context model; a too-small window only costs a minor, safe degradation (more frequent compaction). The narrower `"262144"` default therefore applies to anything that isn't recognizably `kimi-k3*`, and the wider `"1048576"` value is opt-in via the model name itself. See [feature/073](../feature/073_kimi_provider_preset.md) for the full design rationale.

### DeepSeek-Tier Model Environment Overrides (Feature 078)

`switch_account()` extends the Feature 071 write/clear unit with a second, independently-gated bundle of 6 more `env.*` sub-keys — 4 of which share their key names with the Kimi-tier bundle above (`ANTHROPIC_DEFAULT_OPUS_MODEL`/`_SONNET_MODEL`/`_HAIKU_MODEL`, `CLAUDE_CODE_SUBAGENT_MODEL`, `CLAUDE_CODE_EFFORT_LEVEL`, `CLAUDE_CODE_AUTO_COMPACT_WINDOW` — no `_FABLE_MODEL`), gated on the target account being `backend: "redirect"` **and** carrying `inference_provider: "deepseek"` (exact match; any other value, including `"kimi"`, gets the base 3 only or the Kimi-tier 7, never this bundle).

- **Switching TO a `backend: "redirect"` account with `inference_provider: "deepseek"`:** write all 6 DeepSeek-tier sub-keys alongside the base 3: `ANTHROPIC_DEFAULT_OPUS_MODEL` and `ANTHROPIC_DEFAULT_SONNET_MODEL` set to that account's `redirect_model` value (mirrors Kimi's rule exactly); `ANTHROPIC_DEFAULT_HAIKU_MODEL` and `CLAUDE_CODE_SUBAGENT_MODEL` set to the fixed literal `"deepseek-v4-flash"` **regardless of `redirect_model`** (a Pro/Flash split — not a uniform mirror, see [feature/078](../feature/078_deepseek_provider_preset.md)'s Design section); `CLAUDE_CODE_EFFORT_LEVEL` set to the fixed string `"max"` (same value Kimi uses); `CLAUDE_CODE_AUTO_COMPACT_WINDOW` set to the fixed string `"786432"`, with no model-name branching (DeepSeek V4-Pro and V4-Flash publicly share the same 1M-token context window, so no smaller-context tier exists to branch around).
- **Switching TO a `backend: "redirect"` account whose `inference_provider` is not `"deepseek"`:** write only the base 3 sub-keys, or the 7 Kimi-tier sub-keys if `inference_provider == "kimi"` (unchanged behavior); the 6 DeepSeek-tier sub-keys are actively removed if present from a prior switch — this account gets no stale DeepSeek-tier state left behind.
- **Switching TO a `backend: "anthropic"` account, or to a non-deepseek `backend: "redirect"` account:** remove all 6 DeepSeek-tier sub-keys in addition to whatever the base-3 and Kimi-tier rules above already remove or leave in place; `env` is pruned entirely only once every sub-key (base, Kimi-tier, and DeepSeek-tier) is gone. This includes switching directly to a `backend: "redirect"` account tagged `inference_provider: "kimi"` — the 6 DeepSeek-tier sub-keys are removed in the same call that writes the 7 Kimi-tier sub-keys.
- **Cross-provider clearing:** because the two bundles are independently gated on the exact same target-account switch, a switch directly from a Kimi-tagged redirect account to a DeepSeek-tagged one (or vice versa) always clears the outgoing provider's bundle in the same call that writes the incoming provider's bundle — never leaving both sets of tier vars coexisting in `env`. See [feature/078](../feature/078_deepseek_provider_preset.md) AC-11.
- **No Fable-tier substitution:** DeepSeek's own "Integrate with Claude Code" guide documents no `ANTHROPIC_DEFAULT_FABLE_MODEL` mapping, so `switch_account()` never writes it for a `inference_provider: "deepseek"` account (nor does it need clearing on switch-away, since it was never written in the first place unless a prior Kimi switch left it — which the Kimi-tier clear above already handles).

### Effort Tracking Behavior (Fix BUG-312, Fix BUG-322, TSK-335)

`apply_model_override()` writes `effortLevel` unconditionally on every call regardless of whether the model changed: `"max"` for Opus branch, `"high"` for Sonnet and absent-tier branches (TSK-335). The BUG-312 fallback guard (`get_session_effort().is_none()` → `"high"`) is retained as unreachable safety net. The rotation carry-forward `set_session_effort()` was removed — `apply_model_override()` owns all effort writes.

### Features

| File | Relationship |
|------|-------------|
| [feature/034_explicit_session_model_override.md](../feature/034_explicit_session_model_override.md) | `set_session_model()` and `get_session_model()` |
| [feature/035_model_command.md](../feature/035_model_command.md) | `.model` command; `map_model_shorthand()` |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `set_session_effort()`, footer effort display |
| [feature/071_redirect_backend_accounts.md](../feature/071_redirect_backend_accounts.md) | Base 3-variable `env.*` write/clear — `ANTHROPIC_BASE_URL`/`ANTHROPIC_AUTH_TOKEN`/`ANTHROPIC_MODEL` |
| [feature/073_kimi_provider_preset.md](../feature/073_kimi_provider_preset.md) | 7 additional Kimi-tier `env.*` variables, gated on `inference_provider == "kimi"` |
| [feature/078_deepseek_provider_preset.md](../feature/078_deepseek_provider_preset.md) | 6 additional DeepSeek-tier `env.*` variables, gated on `inference_provider == "deepseek"`; cross-provider clearing between the Kimi-tier and DeepSeek-tier bundles |

### Schema

| File | Relationship |
|------|-------------|
| [003_file_topology.md](003_file_topology.md) | `settings_file()` path method |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/007](../invariant/007_json_storage_format.md) | Exception: `json_serialize_flat_object` is exempt |
