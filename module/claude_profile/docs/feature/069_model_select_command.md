# Feature: Subprocess Model Select Command

### Scope

- **Purpose**: Provide a `clp .model.select` command to get or pin the subprocess model used by `clr run`, `clr ask`, `clr isolated`, and `clr refresh` via a `subprocess_model` field in `~/.clr/prefs.json`.
- **Responsibility**: Documents the `.model.select` command, its three operating modes (get/set/reset), the `id::`, `reset::`, and `format::` parameters, the `~/.clr/prefs.json` preference file schema (Schema 008), and the clr integration that reads the preference in `claude_runner_core/src/isolated.rs`.
- **In Scope**: `.model.select` command; get mode (no `id::`, no `reset::`) reading `subprocess_model` from `~/.clr/prefs.json`; set mode (`id::VALUE`) writing `subprocess_model` to `~/.clr/prefs.json`; reset mode (`reset::1`) removing `subprocess_model` key; full model ID values only (no shorthand mapping — use `.models` output); `format::text` and `format::json` in get mode; `~/.clr/prefs.json` file creation when absent on first write; clr reading the preference in `isolated.rs` and using it in place of `ISOLATED_DEFAULT_MODEL` when set.
- **Out of Scope**: Interactive session model in `settings.json` (→ Feature 035); touch/refresh subprocess model control (→ Feature 026 `imodel::` — intentionally separate, quota-adaptive); model discovery (→ Feature 068); subprocess effort level (→ algorithm/008).

### Design

`.model.select` manages the `subprocess_model` preference for clr task-execution subprocesses (`clr run`, `clr ask`, `clr isolated`, `clr refresh`). This preference adds a user-settable override layer above `ISOLATED_DEFAULT_MODEL` without affecting the existing `imodel::` mechanism for touch/refresh subprocesses.

**Preference storage:**

`~/.clr/prefs.json` — a new JSON file in the clr runtime directory (Schema 008):
```json
{ "subprocess_model": "claude-opus-4-8" }
```

The file is created on first set. When `subprocess_model` is absent or `~/.clr/prefs.json` does not exist, `clr` falls back to `ISOLATED_DEFAULT_MODEL`.

**Get mode** (no `id::`, no `reset::1`):

Reads `~/.clr/prefs.json`. Extracts `subprocess_model` field. Prints `model.select: claude-opus-4-8` or `model.select: (unset)` in text format; `{"subprocess_model":"claude-opus-4-8"}` or `{"subprocess_model":null}` in JSON format. Exits 0.

**Set mode** (`id::VALUE`):

Validates that `VALUE` is non-empty. Writes `subprocess_model` to `~/.clr/prefs.json` (creates file if absent; preserves any other keys). Prints `model.select: VALUE (pinned)`. Exits 0. No live API validation of the model ID — the user is expected to run `.models` first to obtain the correct full ID.

**Reset mode** (`reset::1`):

Removes `subprocess_model` key from `~/.clr/prefs.json`. Preserves other keys. Prints `model.select: (reset to default)`. Exits 0. If `~/.clr/prefs.json` does not exist, prints the same message and exits 0 (idempotent).

**Mutual exclusion:** `id::` and `reset::1` together → exits 1 with stderr `model.select: id:: and reset::1 are mutually exclusive`.

**clr integration:**

`dispatch_run()` / `dispatch_ask()` (`claude_runner/src/cli/mod.rs`) and `run_isolated_ext()` (`claude_runner_core/src/isolated.rs`) all read the preference. After `apply_env_vars()` resolves CLI flags and `CLR_*` env vars, if `--model` is still unset:
1. Try to read `~/.clr/prefs.json` → parse `subprocess_model` field.
2. If present and non-empty: use this model ID instead of the command's default model.
3. If absent, error, or empty: use the default model as before.

Precedence (highest to lowest): explicit `--model` flag → `CLR_MODEL` env var → `prefs.json` pin → built-in default.

The preference applies to `run`, `ask`, and `isolated`. `refresh` always uses `REFRESH_DEFAULT_MODEL`; see below.

**Why not affect touch/refresh subprocess model:**

Touch (`imodel::`) is quota-adaptive by design. The auto model selection (Haiku vs Sonnet based on quota) is a feature, not a configuration. Pinning it would defeat the purpose. `imodel::` per-invocation override remains the right mechanism for explicit touch model control.

Refresh (`clr refresh`) passes `IsolatedModel::Specific(REFRESH_DEFAULT_MODEL)` regardless of the preference — credential stability requires a known working model. The user pin is intentionally excluded from the refresh path.

### Acceptance Criteria

- **AC-01**: `clp .model.select` with no `~/.clr/prefs.json` → prints `model.select: (unset)`. Exits 0.
- **AC-02**: `clp .model.select` with `~/.clr/prefs.json` containing `{"subprocess_model":"claude-opus-4-8"}` → prints `model.select: claude-opus-4-8`. Exits 0.
- **AC-03**: `clp .model.select id::claude-opus-4-8` → `~/.clr/prefs.json` contains `"subprocess_model":"claude-opus-4-8"`. Stdout: `model.select: claude-opus-4-8 (pinned)`. Exits 0.
- **AC-04**: `clp .model.select id::claude-sonnet-5` → `~/.clr/prefs.json` contains `"subprocess_model":"claude-sonnet-5"`. Exits 0.
- **AC-05**: `clp .model.select reset::1` with preference set → `subprocess_model` key removed from `~/.clr/prefs.json`; other keys preserved. Prints `model.select: (reset to default)`. Exits 0.
- **AC-06**: `clp .model.select reset::1` with no `~/.clr/prefs.json` → exits 0 idempotently; prints `model.select: (reset to default)`.
- **AC-07**: `clp .model.select id::VALUE` creates `~/.clr/prefs.json` when absent. Exits 0.
- **AC-08**: `clp .model.select id::VALUE` on existing `prefs.json` with other keys → other keys preserved. Exits 0.
- **AC-09**: `clp .model.select id::VALUE reset::1` → exits 1 with stderr containing `mutually exclusive`.
- **AC-10**: `clp .model.select format::json` with preference set → prints `{"subprocess_model":"claude-opus-4-8"}`. Exits 0.
- **AC-11**: `clp .model.select` is listed in `clp .help` output.
- **AC-12**: `clp .model.select id::` (empty string value) → exits 1 with stderr indicating `id::` must be non-empty.

### Features

| File | Relationship |
|------|--------------|
| [068_models_list_command.md](068_models_list_command.md) | `.models` — discover full model IDs to pass to `id::` |
| [035_model_command.md](035_model_command.md) | `.model` — complementary, manages `settings.json` model (interactive session) |
| [026_subprocess_model_effort.md](026_subprocess_model_effort.md) | `imodel::` — per-invocation touch/refresh subprocess model (intentionally separate) |

### Schema

| File | Relationship |
|------|--------------|
| [schema/008_clr_prefs_json.md](../schema/008_clr_prefs_json.md) | `~/.clr/prefs.json` schema — `subprocess_model` field |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/model_select.rs` | `.model.select` command handler |
| `src/registry.rs` | Registration of `.model.select` command and parameters |
| `module/claude_runner_core/src/isolated.rs` | clr subprocess integration — reads `subprocess_model` preference; falls back to `ISOLATED_DEFAULT_MODEL` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/feature/069_model_select_command.md` | FT-01 through FT-12 |
| `tests/docs/cli/command/20_model_select.md` | IT-01 through IT-12 |
