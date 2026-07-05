# Parameter: 54. `set_model::`

Explicitly writes a Claude Code session model to `~/.claude/settings.json`. When provided, overrides the automatic `apply_model_override()` logic for that invocation.

- **Default:** *(omit)* — automatic override behavior runs as usual
- **Constraints:** `opus`, `sonnet`, `haiku`, `default`
- **Purpose:** Explicitly set the Claude Code session model from `clp` without using the interactive `/model` picker or editing `settings.json` manually.

**Values:**

| Value | Model ID written to `settings.json` | Effect |
|-------|--------------------------------------|--------|
| `opus` | `claude-opus-4-8` | Force Opus session model |
| `sonnet` | `claude-sonnet-5` | Force Sonnet session model |
| `haiku` | `claude-haiku-4-5-20251001` | Force Haiku session model |
| `default` | *(removes `model` key)* | Revert to Claude Code's built-in default |

**Precedence:** When `set_model::` is provided, `apply_model_override()` is skipped for that invocation. The explicitly requested model is the final value written to `settings.json`. Without `set_model::`, the automatic Sonnet→Opus threshold override (BUG-225 fix) runs as usual.

**Examples:**

```bash
clp .account.use name::alice@home.com set_model::opus
clp .account.use name::alice@home.com set_model::haiku
clp .usage set_model::sonnet
clp .usage set_model::default
```

**Error cases:**
- `set_model::bad` → exit 1 with stderr naming valid values: `opus`, `sonnet`, `haiku`, `default`

**Notes:**
- On `.account.use`: `set_session_model()` is called after the credential rotation and any `apply_post_switch_touch()` subprocess. The requested model is the final value.
- On `.usage`: `set_session_model()` is called for the current session (is_current account) instead of `apply_model_override()`.
- Does not affect `format::json` output structure.
- Does not affect subprocess model selection — use `imodel::` for that.

### Referenced Type

- **Fundamental Type:** `enum`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.use`](../command/001_account.md#command-5-accountuse) | Set session model after account switch |
| 2 | [`.usage`](../command/006_usage.md#command-9-usage) | Set session model for current account |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Lock session model after switching to target account |
