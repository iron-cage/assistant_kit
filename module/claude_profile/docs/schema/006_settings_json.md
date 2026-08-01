# Schema: Session Settings — `~/.claude/settings.json`

### Scope

- **Purpose**: Define which fields in `~/.claude/settings.json` are read or written by `clp`, their semantics, and the write callers.
- **Responsibility**: Documents the `settings.json` fields that `clp` reads or writes and their write rules.
- **In Scope**: All `settings.json` fields that `clp` touches — `model`, `effortLevel`, and (redirect-backend accounts only) the `env.ANTHROPIC_BASE_URL` / `env.ANTHROPIC_AUTH_TOKEN` / `env.ANTHROPIC_MODEL` sub-keys. All other fields, including any other `env` sub-key, are owned by the Claude binary and must never be modified.
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
| `model` | string or absent | absent | Session model shorthand (`"sonnet"`, `"opus"`, `"haiku"`, or full model ID). Controls which Claude model is used for interactive sessions. | `set_session_model()`, `switch_account()` (restores from `{name}.json`), `.model set::`, `.account.use set_model::`, `apply_model_override()` (Fix BUG-311: bidirectional), `set_session_effort()` init path | `get_session_model()`, `.usage`/`.accounts` `model::1`, `recommended_model()` in `format.rs` |
| `effortLevel` | string or absent | absent → initialized by `apply_model_override()` on first use | Effort level for interactive sessions (`"low"`, `"normal"`, `"high"`, `"max"`). Controls extended thinking depth. | `apply_model_override()` writes unconditionally: `"max"` for Opus, `"high"` for Sonnet/absent-tier (TSK-335); BUG-312 fallback `"high"` retained as unreachable safety net | `get_session_effort()` — footer `Current` line in `.usage`; `Next` line uses model-derived effort from `recommended_model()` |
| `env.ANTHROPIC_BASE_URL` | string or absent | absent | Redirect target base URL, read natively by the Claude binary at process startup to route all API traffic to a foreign endpoint instead of `api.anthropic.com`. | `switch_account()` — written when switching TO a `backend: "redirect"` account, from that account's `base_url` (see [schema/002](../schema/002_account_json.md)) | Claude binary itself (not read by `clp`) |
| `env.ANTHROPIC_AUTH_TOKEN` | string or absent | absent | Static API key sent as the auth credential for the redirect target. | `switch_account()` — written when switching TO a `backend: "redirect"` account, from that account's `accessToken` (see [schema/001](../schema/001_credentials_json.md)) | Claude binary itself (not read by `clp`) |
| `env.ANTHROPIC_MODEL` | string or absent | absent | Model identifier string sent to the redirect target (e.g. a Moonshot Kimi model ID) — the foreign backend's own model catalog, unrelated to `model`'s Anthropic shorthand. | `switch_account()` — written when switching TO a `backend: "redirect"` account, from that account's `redirect_model` (see [schema/002](../schema/002_account_json.md)) | Claude binary itself (not read by `clp`) |

### Write Rules

- `clp` reads the entire `settings.json` into memory, modifies only `model`, `effortLevel`, or (redirect-backend switch only) the `env` object, and writes it back via `json_serialize_flat_object` — all other fields are preserved.
- Never `serde_json::to_string` — the hand-rolled formatter already produces pretty output.
- `env` is the one nested-object field among `clp`'s managed keys (all others are top-level strings), but no formatter change was needed to support it: `set_env_var()`/`remove_env_var()` (`claude_core::settings_io`, pre-existing — built for the `DISABLE_AUTOUPDATER`/`DISABLE_UPDATES` auto-updater toggles) already perform the nested read-modify-write, and `switch_account()` reuses them directly (see [feature/071](../feature/071_redirect_backend_accounts.md)).

### Redirect Backend Environment Override (Feature 071)

`switch_account()` maintains `env.ANTHROPIC_BASE_URL` / `env.ANTHROPIC_AUTH_TOKEN` / `env.ANTHROPIC_MODEL` as a unit, keyed off the target account's `backend`:

- **Switching TO a `backend: "redirect"` account:** write all three sub-keys from that account's `base_url` / `accessToken` / `redirect_model`. If an `env` object already exists in `settings.json` with other sub-keys (unrelated to `clp`), those other sub-keys are preserved — only the three named keys are set.
- **Switching TO a `backend: "anthropic"` account:** remove exactly the three named sub-keys from `env` (if present) so the Claude binary reverts to its own OAuth-based Anthropic routing; if `env` becomes empty as a result, remove the `env` key entirely; if `env` has other unrelated sub-keys remaining, keep `env` with those keys intact.
- `apply_model_override()` and `set_session_model()`/`get_session_model()` never touch `env` — they operate exclusively on the top-level `model`/`effortLevel` keys (see [algorithm/002](../algorithm/002_session_model_override.md)'s redirect bypass).

### Effort Tracking Behavior (Fix BUG-312, Fix BUG-322, TSK-335)

`apply_model_override()` writes `effortLevel` unconditionally on every call regardless of whether the model changed: `"max"` for Opus branch, `"high"` for Sonnet and absent-tier branches (TSK-335). The BUG-312 fallback guard (`get_session_effort().is_none()` → `"high"`) is retained as unreachable safety net. The rotation carry-forward `set_session_effort()` was removed — `apply_model_override()` owns all effort writes.

### Features

| File | Relationship |
|------|-------------|
| [feature/034_explicit_session_model_override.md](../feature/034_explicit_session_model_override.md) | `set_session_model()` and `get_session_model()` |
| [feature/035_model_command.md](../feature/035_model_command.md) | `.model` command; `map_model_shorthand()` |
| [feature/062_unified_session_config.md](../feature/062_unified_session_config.md) | `set_session_effort()`, footer effort display |

### Schema

| File | Relationship |
|------|-------------|
| [003_file_topology.md](003_file_topology.md) | `settings_file()` path method |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/007](../invariant/007_json_storage_format.md) | Exception: `json_serialize_flat_object` is exempt |
