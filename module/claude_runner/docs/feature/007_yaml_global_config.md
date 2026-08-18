# Feature: YAML Global Config

### Scope

- **Purpose**: Propose a YAML-format global configuration mechanism that extends the existing TOML config tier with profile support, subcommand-scoped overrides, and cross-subcommand coverage.
- **Responsibility**: Define the YAML config file format, discovery order, profile selection mechanism, subcommand scope, eligible parameters, and coexistence rules with the existing TOML config.
- **In Scope**: File discovery and format, flat and profiled YAML layouts, `--profile` parameter, subcommand-specific override sections, backward compatibility with existing TOML config, eligible parameter set expansion to all subcommands.
- **Out of Scope**: Implementation (`serde_yaml` wiring, `YamlConfig` struct, `apply_yaml_config()`) — this is a design proposal only. Remote config sources, YAML include/import, schema validation tooling.

### Status

🔄 Planned — design proposal only; no implementation exists.

### Motivation

The existing TOML config tier ([config_param.md](../cli/config_param.md)) works for `run`/`ask` (and, once dispatch wiring lands per task 521, `topic`) but has three gaps this proposal closes:

1. **Profile support absent.** Users running `clr` in automation pipelines often need named configurations (e.g., `fast`, `deep`, `ci`) switchable at runtime. TOML has no inheritance or merge-key syntax; expressing three related profiles requires three separate config files.

2. **Scope limited to `run`/`ask`.** The current TOML config is not read by `isolated`, `refresh`, `ps`, or (pending task 521's dispatch wiring) `topic`. Defaults for these subcommands can only be set via `CLR_*` env vars. A global config should be global — covering all subcommands.

3. **Multi-line values awkward.** YAML's block scalar syntax (`|`) makes multi-line `system_prompt` values natural to read and edit. TOML's `"""..."""` multiline string works but is less familiar and harder to compose in complex configs.

YAML adds all three without introducing a separate config command surface — the file IS the interface, matching the existing TOML approach.

### Design

#### File Locations and Discovery

YAML config files are discovered in the same two locations as TOML:

| Level | Path | Notes |
|-------|------|-------|
| Project | `.clr.yaml` (current working directory) | Read first; project values win over user values |
| User | `~/.clr/config.yaml` | `$XDG_CONFIG_HOME/clr/config.yaml` if `XDG_CONFIG_HOME` is set |

Both files are optional. A missing file at either location is silently treated as empty. When both are present and set the same key, the project file wins — identical precedence semantics to TOML.

**Discovery and precedence within the full 5-level chain:**

```
1. CLI flags
2. --args-file / CLR_ARGS_FILE JSON config (feature/004_json_config.md)
3. CLR_* env vars
4. YAML global config (project .clr.yaml → user ~/.clr/config.yaml)
   — falls back to TOML config (config_param.md) when no YAML file is present
5. Built-in defaults
```

When a YAML file exists at a given discovery level, it fully replaces the TOML file at that level — both formats are never merged. When no YAML file exists, TOML behavior is unchanged.

#### YAML Format — Flat (No Profiles)

A flat YAML file is equivalent to the existing TOML config, just in YAML syntax. Users migrating from TOML can translate their file mechanically:

```yaml
# ~/.clr/config.yaml
model: claude-opus-4-8
effort: high
max_sessions: 4
timeout: 600
quiet: true
retry_on_transient: 3
transient_delay: 5
journal: meta
```

TOML key names (`snake_case`) carry over to YAML unchanged. Unknown keys are silently ignored; the file need not be exhaustive.

#### YAML Format — Profiled

A profiled YAML file declares named sections. The special section `default:` is always applied as the base; a selected profile (see [Profile Selection](#profile-selection)) is then merged on top, with profile values winning over `default:` values.

```yaml
# ~/.clr/config.yaml

default:
  model: claude-opus-4-8
  effort: high
  max_sessions: 4
  timeout: 600
  journal: meta

fast:
  model: claude-haiku-4-5-20251001
  effort: low
  timeout: 60

deep:
  effort: max
  max_tokens: 200000
  retry_on_transient: 5
  transient_delay: 10
  timeout: 3600
```

YAML anchors and merge keys enable DRY composition — unreachable in TOML:

```yaml
_retry: &retry_defaults
  retry_on_transient: 3
  transient_delay: 5
  retry_on_service: 2
  service_delay: 15

default:
  <<: *retry_defaults
  model: claude-opus-4-8
  effort: high

ci:
  <<: *retry_defaults
  effort: low
  timeout: 300
  quiet: true
```

A file that declares named profiles but no `default:` section has no implicit base — only the selected profile (or nothing if no profile is active) applies.

A flat file (no profile sections) is treated as an implicit `default:` — all its keys apply unless a `--profile` is active, in which case the entire flat file acts as the base and the named profile (from the project file, if any) merges on top.

#### Profile Selection

One new CLI parameter is added:

| Parameter | CLI Flag | Env Var | Type | Default |
|-----------|----------|---------|------|---------|
| `profile` | `--profile <name>` | `CLR_PROFILE=<name>` | string | _(absent)_ |

Resolution:
- `--profile` absent, `CLR_PROFILE` absent → only `default:` section applies (or the entire flat file)
- `--profile <name>` or `CLR_PROFILE=<name>` → `default:` applied first, then named profile merged on top
- Profile name not found in the config file → `clr` exits 1; stderr names the missing profile and lists available ones

`--profile` follows the standard 5-level chain: CLI flag wins over `CLR_PROFILE`. `--profile` is not eligible for the config file itself (no self-referential profile selection).

#### Subcommand-Scoped Sections

YAML files may include subcommand-specific override sections. These are applied after the global flat/profiled keys for the relevant subcommand only:

```yaml
default:
  model: claude-opus-4-8
  journal: meta

# isolated-specific overrides (applied only when `clr isolated` runs)
isolated:
  timeout: 120
  strip_fences: true

# ps-specific overrides (applied only when `clr ps` runs)
ps:
  ps_mode: print
  ps_ancient_secs: 14400

# refresh-specific overrides (applied only when `clr refresh` runs)
refresh:
  timeout: 90
```

Subcommand sections coexist with profile sections:

```yaml
default:
  model: claude-opus-4-8

fast:
  model: claude-haiku-4-5-20251001
  effort: low

# Applied on top of resolved profile for `clr isolated` invocations
isolated:
  timeout: 60
```

Resolution order for a profiled invocation of `clr isolated --profile fast`:
1. `default:` section
2. `fast:` profile merged on top
3. `isolated:` section merged on top of the result

This makes subcommand sections the highest-priority layer within the config tier — always narrower and more specific than the global defaults or the selected profile.

#### Eligible Parameters

All 41 parameters currently eligible for the TOML config tier (see [config_param.md](../cli/config_param.md)) remain eligible. The YAML global config additionally covers subcommand-specific parameters not reachable via the existing TOML tier:

**New in YAML — `ps` subcommand parameters:**

| YAML Key | Env Var | Type | Notes |
|----------|---------|------|-------|
| `ps_mode` | `CLR_PS_MODE` | string | `all`/`interactive`/`print`; invalid values exit 1 |
| `ps_columns` | `CLR_PS_COLUMNS` | string | Comma-separated column keys |
| `ps_ancient_secs` | `CLR_PS_ANCIENT_SECS` | u64 | Elapsed-seconds threshold for 🕰 flag; default 28800 |
| `ps_high_ram_mb` | `CLR_PS_HIGH_RAM_MB` | u64 | RSS threshold for 🐘 flag; default 400 |

**New in YAML — `isolated` subcommand parameters:**

| YAML Key | Env Var | Type | Notes |
|----------|---------|------|-------|
| `strip_fences` | `CLR_STRIP_FENCES` | bool | |
| `output_style` | `CLR_OUTPUT_STYLE` | string | `summary`/`raw` |
| `summary_fields` | `CLR_SUMMARY_FIELDS` | string | Preset or comma-separated whitelist |

`timeout` and `journal`/`journal_dir` already appear in the TOML eligible set for `run`/`ask` (and, once dispatch wiring lands per task 521, `topic`); the YAML global config extends their applicability to `isolated` and `refresh` via the `isolated:` and `refresh:` subcommand sections.

#### Multi-Line Values

The primary readability advantage of YAML for config authoring:

```yaml
# Block scalar — preserves newlines, readable for long prompts
default:
  system_prompt: |
    You are a senior Rust developer.
    Follow the project's coding standards.
    Prefer explicit error handling over panics.
    Use the error_tools crate for all errors.

  model: claude-opus-4-8
```

The same value in TOML requires triple-quoted strings and is harder to read inline with other settings.

Note: `system_prompt` is currently in the "Not Configurable" list for the TOML tier (see [config_param.md](../cli/config_param.md) § Not Configurable) because it is "call-specific." For the YAML global config, `system_prompt` and `append_system_prompt` are proposed as eligible — a persistent default system prompt per-project (via `.clr.yaml`) is a concrete, stable use case YAML makes practical.

#### Error Handling

| Scenario | Behavior |
|----------|----------|
| File absent (either location) | Silently treated as empty — no error |
| Malformed YAML | `clr` exits 1; stderr names the offending file path |
| Unknown top-level key (flat file) | Silently ignored |
| Unknown key within a profile section | Silently ignored |
| Unknown subcommand section name | Silently ignored |
| `--profile <name>` with no matching section | `clr` exits 1; stderr names the profile and lists available sections |
| Both YAML and TOML present at same level | YAML takes precedence; TOML ignored at that level |

#### Backward Compatibility

No breaking changes. The YAML config is introduced as a higher-priority alternative to TOML at each discovery level:

- Users with an existing `~/.clr/config.toml` and no `~/.clr/config.yaml`: behavior unchanged.
- Users who create `~/.clr/config.yaml`: TOML is silently superseded at the user level only. The project `.clr.toml` (if any) is still honored unless `.clr.yaml` also exists in the project root.
- Migration: rename or convert the TOML file to YAML syntax. No `clr` command is needed.

#### `--profile` and the Existing `--args-file`

`--args-file` (tier 2) already provides per-invocation config injection and wins over all tier-4 config sources. Users wanting a per-invocation profile override can combine both:

```sh
clr --profile fast --args-file ./ci-overrides.json "Run the tests"
# JSON source wins over YAML profile for any key both specify
```

`--profile` operates only at tier 4 — it selects which named section of the YAML file applies; it cannot name or override `--args-file` files.

### New CLI Parameter

| # | Flag | Env Var | JSON Key | Type | Notes |
|---|------|---------|----------|------|-------|
| 87 | `--profile <name>` | `CLR_PROFILE` | `"profile"` | string | Selects named YAML profile; absent = `default:` section only; not applicable when no YAML config is present |

> **ID note.** This feature originally reserved ID 86, which has since shipped as
> [`086_no_stdin.md`](../cli/param/086_no_stdin.md) (`--no-stdin`, BUG-492). The next free ID at time
> of writing is 87 — reconfirm against `cli/param/` before implementing, since this feature is still
> unimplemented and further parameters may land first.

### Implementation Dependencies

- **`serde_yaml`** crate — YAML deserialization into typed structs.
- New `YamlConfig` struct with profile-aware deserialization (flat vs. profiled detection).
- New `load_yaml_config(profile: Option<&str>, paths: &[PathBuf])` function, replacing `load_config()` when YAML files are detected.
- `--profile` wired into `CliArgs` and applied before tier-4 YAML loading.
- `CLR_YAML_CONFIG_DIR` (analogous to `CLR_CONFIG_DIR`) for test injection of the YAML user config directory.

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/config.rs` | Existing TOML implementation — parallel YAML implementation extends this |
| `../cli/config_param.md` | TOML eligible parameters — YAML inherits all 41 plus the new subcommand set |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](001_runner_tool.md) | Runner tool — YAML global config extends the runner's parameter resolution chain |
| [feature/003_retry_hierarchy.md](003_retry_hierarchy.md) | Retry parameters are among the 41 eligible YAML keys |
| [feature/004_json_config.md](004_json_config.md) | JSON config (tier 2) — YAML config (tier 4) sits below it in the resolution chain |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/yaml_config_test.rs` (to create) | T01–T12: flat YAML, profiled YAML, project-over-user, `--profile` selection, missing profile exit 1, subcommand sections, YAML-over-TOML precedence, malformed YAML exit 1, multiline system_prompt, anchor/alias resolution, `CLR_YAML_CONFIG_DIR` injection, `CLR_PROFILE` env var |
