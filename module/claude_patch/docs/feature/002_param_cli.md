# Feature: Param CLI

### Scope

- **Purpose**: Document the `.param.*` command subject of the `clt`/`claude_patch` binary — a read-only CLI surface for inspecting Claude Code parameter provenance (CLI/env/config/default forms, resolved effective value, and hidden-parameter visibility).
- **Responsibility**: Describe `.param.list`/`.param.show` behavior and the exact upstream `claude_version_core` gaps this subject depends on but does not itself fix.
- **In Scope**: `.param.list` (`kind::` filter), `.param.show` (full single-param provenance), delegation design to `claude_version_core`, the 4 identified upstream gaps.
- **Out of Scope**: `.patch.*` component lifecycle (→ `feature/001_patch_cli.md`), the 4-layer resolution algorithm itself (→ `claude_version/docs/algorithm/002_config_resolution.md`), the params catalog's own field definitions (→ `claude_version/docs/feature/007_params_command.md`).

### Design

**Status:** Design settled across a multi-turn planning conversation; no implementation exists yet (🔄 Planned — see `feature/readme.md` Overview Table).

**Purpose distinction:** `.param.*` is strictly read-only (see `invariant/001_no_param_mutation.md`) — it never writes settings.json or any other state. It answers "where does this Claude Code parameter's value actually come from, and what is it right now?"

**Why `.param.*` lives in `claude_patch` rather than only in `claude_version`:** Patch components (`.patch.*`) can change which value is effectively in force for a Claude Code parameter — e.g. a `version_lock` patch pins the binary version; a future config-override patch kind could pin a settings.json value. `.param.*` gives the same tool a way to verify what actually took effect (CLI form, env override, config value and its concrete file, default, and the resolved winner) without switching to a separate binary, and makes previously-hidden parameters discoverable for troubleshooting. This mirrors `claude_version`'s own precedent of cross-depending on a foreign Layer-1 crate (`claude_runner_core`) rather than duplicating logic — `claude_patch` cross-depends on `claude_version_core` the same way, calling its resolution functions directly (zero duplication).

**Commands:**

| Command | Purpose | Required args | Optional args |
|---------|---------|----------------|-----------------|
| `.param.list` | Survey all known parameters and their forms | — | `kind::config\|env\|hidden` |
| `.param.show` | Full provenance for one parameter | `name::` | — |

**`kind::` filter (`.param.list` only):**

| Value | Params shown |
|-------|---------------|
| absent | All params in catalog |
| `config` | Only params with a config-key form |
| `env` | Only params with an env-var form |
| `hidden` | Only params marked hidden (not normally advertised) — blocked on Gap 1 below |

**`.param.show` output fields:**

| Field | Source |
|-------|--------|
| CLI form | `ParamDef.cli_flag` |
| Env name + current value | `ParamDef.env_var` + live env read |
| Config key + current value + concrete file path | `ParamDef.config_key` + resolved value + resolved file path — blocked on Gap 4 below |
| Default | `ParamDef.default` |
| Effective value + source | Highest-priority non-absent layer among CLI/env/config/default |
| Hidden flag | `ParamDef.hidden` — blocked on Gap 1 below |

**Upstream gaps this subject depends on (not fixed by this crate):**

1. **No `hidden: bool` field on `ParamDef`.** Confirmed via direct inspection of `claude_version_core/src/params_catalog.rs` (`ParamDef` struct) — its fields are `name`, `cli_flag`, `env_var`, `config_key`, `default`; there is no `hidden` field, and no catalog entry is marked hidden/undocumented/internal in any form today. `.param.list kind::hidden` and `.param.show`'s hidden flag cannot be implemented until `claude_version_core` adds this field and populates it for the relevant catalog entries.
2. **CLI-only parameters have no resolvable value.** `ParamDef::is_cli_only()` correctly identifies parameters observable only via CLI flag. Claude Code never persists a CLI flag's value anywhere — there is no file or env var to read after the fact. `.param.show` on a CLI-only parameter MUST document this as ephemeral/form-only (matching `claude_version`'s own `.params` command's existing `(CLI-only)` annotation — see `claude_version/docs/feature/007_params_command.md`) and MUST NEVER fabricate a resolved value.
3. **Two disconnected resolution paths, neither complete for `.param.show`'s promised output.** `claude_version_core::config_resolve::{resolve, resolve_all}` performs the full Env → Project → User → Default resolution via its `Layer` enum, but only against `config_catalog::SettingDef` — a narrower catalog with no `cli_flag` field. `claude_version_core::params_catalog::{ParamDef, lookup, params_catalog}` carries the broader `cli_flag`/`env_var`/`config_key` fields `.param.show` needs, but its own env→config→default resolution today lives as a *private* function (`resolve_effective()` in `claude_version/src/commands/params.rs`) inside the `claude_version` CLI crate — not exported from `claude_version_core` — and it resolves against a single "user" config path only, with no project-tier lookup at all. Assembling `.param.show`'s full promised output therefore requires consulting both catalogs today; a cleaner long-term fix is for `claude_version_core` to export one unified resolver operating on `ParamDef` with full layer awareness.
4. **Provenance shows tier label only, never the concrete file path.** `config_resolve::resolve()` returns `Layer::Project` whenever its project-config lookup succeeds, but the underlying `find_project_config_file()` helper is called purely to extract the *value* at that path — the resolved path itself is discarded and never attached to `ResolvedValue` (which carries only `value` and `source: Layer`). For the `User` tier the path is always the deterministic `<home_dir>/.claude/settings.json`, so it is trivial to display even today — but for the `Project` tier, showing the concrete resolved file requires that already-computed result to actually be surfaced on `ResolvedValue` rather than discarded. This is a small, concrete, well-understood upstream fix, not a speculative one.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_patch_cli.md](001_patch_cli.md) | Sibling `.patch.*` subject in the same binary |
| [claude_version/docs/feature/007_params_command.md](../../../claude_version/docs/feature/007_params_command.md) | Sibling `.params` command in `claude_version` this subject parallels and depends on |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_no_param_mutation.md](../invariant/001_no_param_mutation.md) | Enforced constraint: `.param.*` never mutates state |

### Algorithms

| File | Relationship |
|------|--------------|
| [claude_version/docs/algorithm/002_config_resolution.md](../../../claude_version/docs/algorithm/002_config_resolution.md) | The 4-layer resolution algorithm `.param.show`'s config field depends on |

### Sources

| File | Relationship |
|------|--------------|
| `../../../claude_version_core/src/config_resolve.rs` | `resolve()`/`resolve_all()` — full Env/Project/User/Default resolution (Gaps 3-4) |
| `../../../claude_version_core/src/params_catalog.rs` | `ParamDef`, `lookup()`, `params_catalog()` — broader catalog with CLI/env/config forms (Gaps 1-3) |
| `src/commands/param.rs` (to create) | `.param.*` command handlers |

### Tests

| File | Relationship |
|------|--------------|
| `tests/param_cli.rs` (to create) | Command-level integration tests, including CLI-only ephemeral-value handling |
