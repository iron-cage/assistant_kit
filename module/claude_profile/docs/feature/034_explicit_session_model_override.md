# Feature: Explicit Session Model Override

### Scope

- **Purpose**: Allow users to set the `~/.claude/settings.json` model key explicitly from `.account.use` and `.usage`, bypassing the automatic `apply_model_override()` Sonnet→Opus threshold logic.
- **Responsibility**: Documents the `set_model::` parameter, `validate_set_model()` value mapping, `set_session_model()` write path, override precedence over auto-override, and `default` removal behavior.
- **In Scope**: `set_model::` parameter on `.account.use` and `.usage`; `validate_set_model()` validation function; `set_session_model()` write-to-settings implementation in `claude_profile_core`; precedence ordering ensuring explicit value wins over `apply_model_override()`; `default` value removes the `model` key; `[trace] account.use … set_model:` trace line on `.account.use` when `trace::1`.
- **Out of Scope**: Subprocess model selection (→ 026_subprocess_model_effort.md `imodel::` parameter); interactive model picker in Claude Code UI; `apply_model_override()` Sonnet→Opus threshold logic when `set_model::` is absent (→ 027_account_use_post_switch_touch.md).

### Design

`set_session_model(paths, model_id)` in `claude_profile_core` reads `~/.claude/settings.json`, inserts or removes the `"model"` key, and writes the file back. `model_id = Some(id)` inserts the key; `model_id = None` removes it (reverting Claude Code to its built-in default).

`validate_set_model(s)` maps the four accepted shorthands to their fully-qualified model IDs:

| Input | Result |
|-------|--------|
| `opus` | `Some("claude-opus-4-6")` |
| `sonnet` | `Some("claude-sonnet-4-6")` |
| `haiku` | `Some("claude-haiku-4-5-20251001")` |
| `default` | `None` (removes `model` key) |
| anything else | `Err(...)` → exit 1 |

**`.account.use` execution path:**

`set_model::` validation runs at argument parse time. The resolved string is stored as `set_model_str: Option<String>`.

Post-match placement: `set_session_model()` is called AFTER the `PreSwitchOutcome` match block, ensuring it wins over any `apply_model_override()` call that fires inside `apply_post_switch_touch()`. No guard is needed: `apply_post_switch_touch()` runs for `NeedTouch` outcomes and calls `apply_model_override()` internally; `set_session_model()` writes last — the explicit value always wins by ordering. (Note: the `AlreadyActive` match arm referenced in earlier versions of this doc was removed by Fix(BUG-285).)

```rust
// Post-match: explicit override always wins
if let Some( ref sm ) = set_model_str
{
  let model_id = validate_set_model( sm ).ok().flatten();
  set_session_model( &paths, model_id );
  if trace { eprintln!( "[trace] account.use  {name}  set_model: {sm}" ) }
}
```

**`.usage` execution path:**

`set_model::` is checked in the session-model override block (runs after touch loop, before row-filter pipeline). When `set_model` is `Some`, `set_session_model()` is called directly and `apply_model_override()` is skipped entirely. When `set_model` is `None`, the normal auto-override path executes for the `is_current` account.

**`settings.json` write mechanics:**

`set_session_model()` performs a read-merge-write: reads the existing `~/.claude/settings.json` (defaults to `{}` if absent or unparseable), inserts/removes the `"model"` key, then writes the file. Existing keys are preserved.

### Acceptance Criteria

- **AC-01**: `set_model::opus` writes `"model": "claude-opus-4-6"` to `~/.claude/settings.json` on both `.account.use` and `.usage`.
- **AC-02**: `set_model::sonnet` writes `"model": "claude-sonnet-4-6"` to `~/.claude/settings.json` on both commands.
- **AC-03**: `set_model::haiku` writes `"model": "claude-haiku-4-5-20251001"` to `~/.claude/settings.json` on both commands.
- **AC-04**: `set_model::default` removes the `"model"` key from `~/.claude/settings.json` on both commands; existing keys are unaffected.
- **AC-05**: When `set_model::` is provided, `apply_model_override()` is NOT called — the explicit value is the final write to `settings.json`.
- **AC-06**: On `.account.use` with `trace::1` and `set_model::X`: emits `[trace] account.use  {name}  set_model: X` to stderr after the credential rotation.
- **AC-07**: `set_model::bad` exits 1 with stderr containing all four valid values: `opus`, `sonnet`, `haiku`, `default`.
- **AC-08**: `set_model::` appears in `.account.use --help` and `.usage --help` output.
- **AC-09**: `set_model::` does not affect `format::json` output structure or subprocess model selection (`imodel::` governs that).
- **AC-10**: `set_session_model()` preserves all existing keys in `~/.claude/settings.json` — a write with `model_id = Some("claude-opus-4-6")` does not remove other keys such as `theme` or `autoUpdaterStatus`.
- **AC-11**: When `~/.claude/settings.json` does not exist and `set_model::opus` is invoked, the file is created with `{"model":"claude-opus-4-6"}`.

### Features

| File | Relationship |
|------|--------------|
| [004_account_use.md](004_account_use.md) | `.account.use` credential rotation — `set_model::` runs after switch completes |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | `imodel::` for subprocess model — orthogonal to `set_model::` |
| [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | Post-switch touch execution — `apply_model_override()` skipped when `set_model::` present |
| [035_model_command.md](035_model_command.md) | Standalone `.model` get/set command — shares `set_session_model()` and the `map_model_shorthand()` inner function extracted from `validate_set_model()` |
| [062_unified_session_config.md](062_unified_session_config.md) | `set_session_effort()` is the counterpart to `set_session_model()` — same read-modify-write pattern |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/054_set_model.md](../cli/param/054_set_model.md) | `set_model::` parameter specification |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/account_ops.rs` | `set_model_str` parsing, post-match `set_session_model()` call, `[trace]` emission |
| `src/usage/api.rs` | `.usage` session-model override block — `set_model` branch vs `apply_model_override` branch |
| `src/usage/types.rs` | `validate_set_model()` — calls `map_model_shorthand()` inner function and formats error with `set_model::` prefix; four-value mapping |
| `claude_profile_core/src/account.rs` | `set_session_model()` — read-merge-write on `~/.claude/settings.json` |
| `src/lib.rs` | `set_model::` parameter registration on `.account.use` and `.usage` |

### Schema

| File | Relationship |
|------|-------------|
| [schema/006_settings_json.md](../schema/006_settings_json.md) | `model` field in `~/.claude/settings.json` read and written by `set_session_model()` |
