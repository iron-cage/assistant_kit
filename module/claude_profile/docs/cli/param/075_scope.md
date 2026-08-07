# Parameter: 75. `scope::`

Selects which of the two persisted model/effort stores a `.model` call targets. New for Feature 035 (the `.model`/`.model.select` merge).

- **Default:** `session`
- **Constraints:** `session`, `subprocess`
- **Purpose:** Route every other parameter on the same call (`model::`, `effort_level::`, `reset_model::`, `reset_effort_level::`, and get-mode output) to the correct backing file.

**Values:**

| Value | Backing store (absolute path) | Notes |
|-------|--------------------------------|-------|
| `session` (default) | `$HOME/.claude/settings.json` | Interactive Claude Code session model/effort. Read/written via `claude_profile_core::account`. |
| `subprocess` | `$HOME/.clr/config.toml` (user tier) | `clr run`/`clr ask`/`clr isolated`/`clr refresh` model/effort preference. Read/written via `claude_core::toml_io`; project-tier `.clr.toml` is read by `clr` itself but never written by `.model`. |

**Validation:** Exact match against the two-value allow-list. Any other value is rejected before any other parameter is evaluated.

**Error cases:**
- `scope::bad` → exit 1; stderr: `scope:: must be one of: session, subprocess; got "bad"`

**Examples:**

```bash
clp .model                       # scope::session (default) — get
clp .model scope::subprocess     # get subprocess store
clp .model scope::subprocess model::claude-opus-4-8
```

**Notes:**
- `scope::` does not itself select get/set/reset mode — that is determined independently by which of `model::`/`effort_level::`/`reset_model::`/`reset_effort_level::` are present (see [076_model_value.md](076_model_value.md)).
- `model::`/`effort_level::`'s valid values differ by `scope::` — session uses shorthand/JSON-native vocabularies, subprocess uses the `claude_runner_core::EffortLevel` vocabulary for effort and full model IDs for model. See [Feature 035](../../feature/035_model_command.md) Design for the full per-scope vocabulary table.
- Every mode's output names the resolved absolute path for the selected scope (see [Feature 035](../../feature/035_model_command.md) Absolute Path Disclosure).

### Referenced Type

- **Fundamental Type:** `enum`

### Referenced Parameter Groups

*(ungrouped — `.model`-specific parameter)*

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.model`](../command/007_model.md) | Backing-store router for every other parameter on the same call |
