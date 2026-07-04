# Feature: Dedicated Model Get/Set Command

### Scope

- **Purpose**: Provide a standalone `clp .model` command to read and write the Claude Code session model in `~/.claude/settings.json`, without requiring an account switch or usage fetch.
- **Responsibility**: Documents the `.model` command, its two operating modes (get and set), the `set::` parameter, the shared model-shorthand validation logic, and the no-duplication contract with Feature 034.
- **In Scope**: `.model` command; get mode (no params) printing the current model shorthand or `(unset)`; set mode (`set::VALUE`) writing via `set_session_model()`; `format::json` get output; `get_session_model()` helper in `claude_profile_core`; refactoring of `validate_set_model()` into a shared `map_model_shorthand()` inner function to eliminate duplication of the opus/sonnet/haiku/default mapping table.
- **Out of Scope**: Subprocess model selection (→ 026_subprocess_model_effort.md `imodel::` parameter); automatic Sonnet→Opus threshold override (→ 027_account_use_post_switch_touch.md); `set_model::` side-parameter on `.account.use`/`.usage` (→ 034_explicit_session_model_override.md); 4-layer config resolution with env-var and project overrides (→ `clv .config key::model` in `claude_version`).

### Design

`.model` operates in two modes selected by the presence of `set::`.

**Get mode** (`clp .model`):

Calls `get_session_model(paths)` — a new helper in `claude_profile_core/src/account.rs` that reads `~/.claude/settings.json` and extracts the `"model"` field via `parse_string_field()`. Returns `None` when the file is absent, unparseable, or the `"model"` key is missing.

Text output:
```
model: opus
```
or, when absent:
```
model: (unset)
```

JSON output (`format::json`):
```json
{"model": "opus"}
```
or, when absent:
```json
{"model": null}
```

**Set mode** (`clp .model set::VALUE`):

1. Calls `map_model_shorthand(VALUE)` — the shared inner function (see No-Duplication Contract). On unknown value: exits 1 with stderr `"set:: must be one of: opus, sonnet, haiku, default; got {VALUE:?}"`.
2. Calls `set_session_model(paths, model_id)` — the same write helper used by Feature 034's `set_model::` path.
3. Prints `model set: VALUE` to stdout. Exits 0.

**No-Duplication Contract:**

Feature 034 introduced `validate_set_model()` in `src/usage/types.rs` which embeds the `opus → "claude-opus-4-8"` mapping. This feature MUST NOT duplicate that table. The implementation extracts a shared inner function:

```rust
fn map_model_shorthand( s : &str ) -> Option< Option< &'static str > >
{
  match s
  {
    "opus"    => Some( Some( "claude-opus-4-8" ) ),
    "sonnet"  => Some( Some( "claude-sonnet-5" ) ),
    "haiku"   => Some( Some( "claude-haiku-4-5-20251001" ) ),
    "default" => Some( None ),
    _         => None,
  }
}
```

`validate_set_model()` and the new `.model` handler both call `map_model_shorthand()` and format their own parameter-name-specific error messages. The model ID table exists in exactly one place.

Similarly, `get_session_model()` MUST be introduced in `claude_profile_core/src/account.rs` to avoid duplicating the inline `parse_string_field(read_to_string(settings_file), "model")` read pattern that is already used in multiple places in that file.

### Acceptance Criteria

- **AC-01**: `clp .model` prints `model: opus` when `~/.claude/settings.json` contains `{"model": "opus"}`.
- **AC-02**: `clp .model` prints `model: sonnet` when `~/.claude/settings.json` contains `{"model": "sonnet"}`.
- **AC-03**: `clp .model` prints `model: (unset)` when the `"model"` key is absent from `~/.claude/settings.json`.
- **AC-04**: `clp .model` prints `model: (unset)` when `~/.claude/settings.json` does not exist.
- **AC-05**: `clp .model set::opus` writes `"model": "claude-opus-4-8"` to `~/.claude/settings.json`. Exits 0.
- **AC-06**: `clp .model set::sonnet` writes `"model": "claude-sonnet-5"` to `~/.claude/settings.json`. Exits 0.
- **AC-07**: `clp .model set::haiku` writes `"model": "claude-haiku-4-5-20251001"` to `~/.claude/settings.json`. Exits 0.
- **AC-08**: `clp .model set::default` removes the `"model"` key from `~/.claude/settings.json`; all other keys are preserved. Exits 0.
- **AC-09**: `clp .model set::bad` exits 1 with stderr containing all four valid values: `opus`, `sonnet`, `haiku`, `default`.
- **AC-10**: `clp .model set::opus` creates `~/.claude/settings.json` when absent; file contains `{"model":"claude-opus-4-8"}`. Exits 0.
- **AC-11**: `clp .model set::opus` on an existing `settings.json` with `{"theme":"dark"}` — file contains both `"theme":"dark"` and `"model":"claude-opus-4-8"`.
- **AC-12**: `clp .model format::json` prints `{"model":"opus"}` when model is set; `{"model":null}` when absent.
- **AC-13**: `clp .model` is listed in `clp .help` output.
- **AC-14**: Implementation calls `set_session_model()` from `claude_profile_core` (no inline JSON write) and `map_model_shorthand()` from `src/usage/types.rs` (no inline model table).

### Features

| File | Relationship |
|------|--------------|
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | `set_model::` side-parameter — shares `set_session_model()` and `map_model_shorthand()` with this feature |
| [062_unified_session_config.md](062_unified_session_config.md) | `set_session_effort()` added as counterpart to `set_session_model()` for effort persistence |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/055_set.md](../cli/param/055_set.md) | `set::` parameter specification — mode selector on `.model` |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/model.rs` | `.model` command handler: get mode via `get_session_model()`; set mode via `map_model_shorthand()` + `set_session_model()` |
| `src/registry.rs` | `.model` command and `set::` parameter registration |
| `claude_profile_core/src/account.rs` | `get_session_model()` — new helper returning `Option<String>`; `set_session_model()` — existing write helper (reused) |
| `src/usage/types.rs` | `map_model_shorthand()` — new inner function extracted from `validate_set_model()`; `validate_set_model()` refactored to call it |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/feature/035_model_command.md` | Feature test spec (FT-01 through FT-12) |
| `tests/docs/cli/command/17_model.md` | Command-level integration test spec (IT-01 through IT-13) |

### Schema

| File | Relationship |
|------|-------------|
| [schema/006_settings_json.md](../schema/006_settings_json.md) | `model` field in `~/.claude/settings.json` read by `get_session_model()` and written by `set_session_model()` |
