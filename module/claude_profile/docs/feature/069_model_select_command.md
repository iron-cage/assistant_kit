# Feature: Subprocess Model Select Command

### Scope

- **Purpose**: Provide a `clp .model.select` command to get or pin the subprocess model used by `clr run`, `clr ask`, `clr isolated`, and `clr refresh` via a `model` key in `~/.clr/config.toml`.
- **Responsibility**: Documents the `.model.select` command, its three operating modes (get/set/reset), the `id::`, `reset::`, and `format::` parameters, the `~/.clr/config.toml` `model` key it manages (format: [claude_core/docs/api/002_toml_io.md](../../../claude_core/docs/api/002_toml_io.md)), and the clr integration that reads this same key from `claude_runner`'s CLI dispatch and from `claude_runner_core::resolve_isolated_default_model()`.
- **In Scope**: `.model.select` command; get mode (no `id::`, no `reset::`) reading `model` from `~/.clr/config.toml`'s user tier; set mode (`id::VALUE`) writing `model` to `~/.clr/config.toml`'s user tier; reset mode (`reset::1`) removing the `model` key; full model ID values only (no shorthand mapping — use `.models` output); `format::text` and `format::json` in get mode (JSON output key stays `subprocess_model` — this command's own CLI-visible contract, independent of the backing store's key name); `~/.clr/config.toml` file and `.clr` directory creation when absent on first write; clr reading the `model` key via two independent consumers (`claude_runner`'s `--model` CLI resolution and `claude_runner_core::resolve_isolated_default_model()`) and using it in place of `ISOLATED_DEFAULT_MODEL`/hardcoded default when set.
- **Out of Scope**: Interactive session model in `settings.json` (→ Feature 035); touch/refresh subprocess model control (→ Feature 026 `imodel::` — intentionally separate, quota-adaptive); model discovery (→ Feature 068); subprocess effort level (→ algorithm/008); project-tier `.clr.toml` reads/writes by `.model.select` itself (it manages only the user tier — the project tier is merged in separately by the two consumers described under "clr integration").

### Design

`.model.select` manages the `model` key in `~/.clr/config.toml`'s user tier for clr task-execution subprocesses (`clr run`, `clr ask`, `clr isolated`, `clr refresh`). This preference adds a user-settable override layer above `ISOLATED_DEFAULT_MODEL` without affecting the existing `imodel::` mechanism for touch/refresh subprocesses.

**Preference storage:**

`~/.clr/config.toml` — a flat TOML key=value file (parser/serializer and format documented in [claude_core/docs/api/002_toml_io.md](../../../claude_core/docs/api/002_toml_io.md)):
```toml
model = "claude-opus-4-8"
```

`.model.select` reads and writes only this file's user tier — it never touches the project-level `.clr.toml` file itself (that file participates only in the two read-side consumers described below). The file and its parent `.clr` directory are created on first set. When `model` is absent or `~/.clr/config.toml` does not exist, `clr` falls back to `ISOLATED_DEFAULT_MODEL` or the hardcoded CLI default, depending on the consumer.

**Get mode** (no `id::`, no `reset::1`):

Reads `~/.clr/config.toml`'s user tier. Extracts the `model` key. Prints `model.select: claude-opus-4-8` or `model.select: (unset)` in text format; `{"subprocess_model":"claude-opus-4-8"}` or `{"subprocess_model":null}` in JSON format — the JSON key is always `subprocess_model`, unchanged by the task 410 storage migration. Exits 0.

**Set mode** (`id::VALUE`):

Validates that `VALUE` is non-empty. Writes `model` to `~/.clr/config.toml`'s user tier (creates the file and `.clr` directory if absent; preserves any other keys already in the file). Prints `model.select: VALUE (pinned)`. Exits 0. No live API validation of the model ID — the user is expected to run `.models` first to obtain the correct full ID.

**Reset mode** (`reset::1`):

Removes the `model` key from `~/.clr/config.toml`'s user tier. Preserves other keys. Prints `model.select: (reset to default)`. Exits 0. If `~/.clr/config.toml` does not exist, prints the same message and exits 0 (idempotent).

**Mutual exclusion:** `id::` and `reset::1` together → exits 1 with stderr `model.select: id:: and reset::1 are mutually exclusive`.

**clr integration:**

The `model` key `.model.select` writes is read independently by two consumers, each with its own precedence chain — both resolve the same `~/.clr/config.toml` user-tier value when no higher tier overrides it:

1. **`claude_runner`'s own `--model` CLI resolution** — `dispatch_run()` / `dispatch_ask()` (`claude_runner/src/cli/mod.rs`) call `config::load_config()` + `config::apply_config_defaults()` (`claude_runner/src/cli/config.rs`) as the 4th of 5 precedence tiers: explicit `--model` flag → `--args-file`/stdin JSON → `CLR_MODEL` env var → config-file tier → hardcoded default. The config-file tier merges project `.clr.toml` (higher precedence) over user `~/.clr/config.toml` (lower precedence) on the `model` key before filling in whichever `CliArgs` fields are still unset.
2. **`claude_runner_core::resolve_isolated_default_model()`** (`claude_runner_core/src/isolated.rs`) — consulted by `run_isolated_ext()`'s `IsolatedModel::Default` match arm, which `dispatch_isolated()` (`clr isolated`) always passes (there is no `--model` CLI flag on `isolated`). A simpler, independent 2-tier lookup over the same `model` key: project `.clr.toml` → user `~/.clr/config.toml` → `None` if neither is set, in which case `IsolatedModel::model_id()` supplies `ISOLATED_DEFAULT_MODEL` (`"opus"`).

The preference applies to `run`, `ask`, and `isolated`. `refresh` always uses `REFRESH_DEFAULT_MODEL`; see below.

**Why not affect touch/refresh subprocess model:**

Touch (`imodel::`) is quota-adaptive by design. The auto model selection (Haiku vs Sonnet based on quota) is a feature, not a configuration. Pinning it would defeat the purpose. `imodel::` per-invocation override remains the right mechanism for explicit touch model control.

Refresh (`clr refresh`) passes `IsolatedModel::Specific(REFRESH_DEFAULT_MODEL)` regardless of the preference — credential stability requires a known working model. The user pin is intentionally excluded from the refresh path.

### Acceptance Criteria

- **AC-01**: `clp .model.select` with no `~/.clr/config.toml` → prints `model.select: (unset)`. Exits 0.
- **AC-02**: `clp .model.select` with `~/.clr/config.toml` containing `model = "claude-opus-4-8"` → prints `model.select: claude-opus-4-8`. Exits 0.
- **AC-03**: `clp .model.select id::claude-opus-4-8` → `~/.clr/config.toml` contains `model = "claude-opus-4-8"`. Stdout: `model.select: claude-opus-4-8 (pinned)`. Exits 0.
- **AC-04**: `clp .model.select id::claude-sonnet-5` → `~/.clr/config.toml` contains `model = "claude-sonnet-5"`. Exits 0.
- **AC-05**: `clp .model.select reset::1` with preference set → `model` key removed from `~/.clr/config.toml`; other keys preserved. Prints `model.select: (reset to default)`. Exits 0.
- **AC-06**: `clp .model.select reset::1` with no `~/.clr/config.toml` → exits 0 idempotently; prints `model.select: (reset to default)`.
- **AC-07**: `clp .model.select id::VALUE` creates `~/.clr/config.toml` (and its parent `.clr` directory) when absent. Exits 0.
- **AC-08**: `clp .model.select id::VALUE` on an existing `config.toml` with other keys → other keys preserved. Exits 0.
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
| [claude_core/docs/api/002_toml_io.md](../../../claude_core/docs/api/002_toml_io.md) | `~/.clr/config.toml` flat-TOML format and `claude_core::toml_io` parser/serializer — `model` key |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/model_select.rs` | `.model.select` command handler |
| `src/registry.rs` | Registration of `.model.select` command and parameters |
| `module/claude_runner_core/src/isolated.rs` | `resolve_isolated_default_model()` — reads the `model` key; falls back to `ISOLATED_DEFAULT_MODEL` |
| `module/claude_runner/src/cli/config.rs` | `load_config()` / `apply_config_defaults()` — reads the `model` key as the CLI's config-file tier |

### Tests

| File | Relationship |
|------|--------------|
| `tests/docs/feature/069_model_select_command.md` | FT-01 through FT-12 |
| `tests/docs/cli/command/20_model_select.md` | IT-01 through IT-12 |
