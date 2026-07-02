# Feature Test: Params Command

### Scope

- **Purpose**: FT- test cases for the `.params` command — show-all, single-param, kind filter, JSON format, and error paths.
- **Responsibility**: Acceptance criteria verifying param catalog coverage, observable value resolution, CLI-only annotation, kind filtering, and exit codes.
- **In Scope**: All `.params` modes, kind:: filter, format::json, env var reads, config reads, CLI-only marking, exit codes.
- **Out of Scope**: Config write operations (→ `06_config_command.md`), resolution algorithm unit tests (→ `../../algorithm/02_config_resolution.md`).

Feature test surface for `.params`. See [feature/007_params_command.md](../../../docs/feature/007_params_command.md) for specification.

## Behavioral Divergence Pair

Two valid `.params` invocations that produce structurally different output:

- **Input A:** `clv .params` (no params) → table of all catalog params with current observable values
- **Input B:** `clv .params key::model` → deep-dive block for one param, showing all forms and effective value

## Test Case Index

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-1 | AC-1 | `.params` show-all exits 0 with ≥35 entries, each annotated | ✅ `ft1_007_params_show_all_min_entries` |
| FT-2 | AC-2 | `.params key::model` shows all three forms, current values, and default | ✅ `ft2_007_params_single_model_full_detail` |
| FT-3 | AC-3 | `.params kind::config` shows only config-key params; env-only params absent | ✅ `ft3_007_params_kind_config_filters` |
| FT-4 | AC-4 | `.params kind::env` shows only env-var params; config-only params absent | ✅ `ft4_007_params_kind_env_filters` |
| FT-5 | AC-5 | `.params key::model` with CLAUDE_MODEL set shows env value with (env) annotation | ✅ `ft5_007_params_env_override_visible` |
| FT-6 | AC-6 | `.params key::bash_timeout` shows CLAUDE_CODE_BASH_TIMEOUT unset + default 120000 | ✅ `ft6_007_params_env_only_param` |
| FT-7 | AC-7 | `.params format::json` exits 0; output is valid JSON array with required fields | ✅ `ft7_007_params_json_output_structure` |
| FT-8 | AC-8 | `.params key::print` shows CLI-only annotation | ✅ `ft8_007_params_cli_only_annotation` |
| FT-9 | AC-9 | `.params key::NONEXISTENT` exits 2 | ✅ `ft9_007_params_unknown_key_exits_2` |
| FT-10 | AC-10 | `.params kind::badvalue` exits 1 | ✅ `ft10_007_params_invalid_kind_exits_1` |
| FT-11 | AC-11 | `.params key::model` with no env and no config → shows default with (default) annotation | ✅ `ft11_007_params_default_source_annotation` |
| FT-12 | AC-12 | `.params` show-all output is sorted alphabetically | ✅ `ft12_007_params_show_all_alphabetical` |

## Test Coverage Summary

- Show-all mode: 2 tests (FT-1, FT-12)
- Single-param mode: 3 tests (FT-2, FT-5, FT-11)
- Kind filter: 2 tests (FT-3, FT-4)
- Env-var params: 2 tests (FT-5, FT-6)
- CLI-only annotation: 1 test (FT-8)
- JSON format: 1 test (FT-7)
- Error paths: 2 tests (FT-9, FT-10)

**Total:** 12 tests

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| `ft1_007_params_show_all_min_entries` | `tests/integration/feature_surface_test.rs` | FT-1 |
| `ft2_007_params_single_model_full_detail` | `tests/integration/feature_surface_test.rs` | FT-2 |
| `ft3_007_params_kind_config_filters` | `tests/integration/feature_surface_test.rs` | FT-3 |
| `ft4_007_params_kind_env_filters` | `tests/integration/feature_surface_test.rs` | FT-4 |
| `ft5_007_params_env_override_visible` | `tests/integration/feature_surface_test.rs` | FT-5 |
| `ft6_007_params_env_only_param` | `tests/integration/feature_surface_test.rs` | FT-6 |
| `ft7_007_params_json_output_structure` | `tests/integration/feature_surface_test.rs` | FT-7 |
| `ft8_007_params_cli_only_annotation` | `tests/integration/feature_surface_test.rs` | FT-8 |
| `ft9_007_params_unknown_key_exits_2` | `tests/integration/feature_surface_test.rs` | FT-9 |
| `ft10_007_params_invalid_kind_exits_1` | `tests/integration/feature_surface_test.rs` | FT-10 |
| `ft11_007_params_default_source_annotation` | `tests/integration/feature_surface_test.rs` | FT-11 |
| `ft12_007_params_show_all_alphabetical` | `tests/integration/feature_surface_test.rs` | FT-12 |

---

### FT-1: show-all exits 0 with ≥35 entries

- **Given:** clean environment; HOME set; no CLAUDE_MODEL env var
- **When:** `clv .params`
- **Then:** exit 0; stdout contains at least 35 parameter entries; each entry includes a source annotation (default/env/user/project) or CLI-only marker
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-1](../../../docs/feature/007_params_command.md)

---

### FT-2: single param shows all three forms, current values, and default

- **Given:** clean environment; HOME set; `settings.json` has `model: "claude-sonnet-5"`; `CLAUDE_MODEL` unset
- **When:** `clv .params key::model`
- **Then:** exit 0; output shows CLI form (`--model`), env form (`CLAUDE_MODEL`), config form (`model`); shows current config value, default `claude-sonnet-5`, and effective value with source
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-2](../../../docs/feature/007_params_command.md)

---

### FT-3: kind::config filters to config-key params only

- **Given:** clean environment; HOME set
- **When:** `clv .params kind::config`
- **Then:** exit 0; output contains only params that have a config key form (e.g. `model`, `theme`); env-only params (`bash_timeout`, `api_key`) absent from output
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-3](../../../docs/feature/007_params_command.md)

---

### FT-4: kind::env filters to env-var params only

- **Given:** clean environment; HOME set
- **When:** `clv .params kind::env`
- **Then:** exit 0; output contains only params with an env var form; config-only params (e.g. `theme`, `voiceEnabled`) absent from output
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-4](../../../docs/feature/007_params_command.md)

---

### FT-5: env override visible with (env) annotation

- **Given:** clean environment; `CLAUDE_MODEL=claude-opus-4-8` set in env; HOME set
- **When:** `clv .params key::model`
- **Then:** exit 0; output shows env value `claude-opus-4-8` annotated with `(env)` source; env layer wins over config and default
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-5](../../../docs/feature/007_params_command.md)

---

### FT-6: env-only param shows env var status and default

- **Given:** clean environment; `CLAUDE_CODE_BASH_TIMEOUT` unset; HOME set
- **When:** `clv .params key::bash_timeout`
- **Then:** exit 0; output shows `CLAUDE_CODE_BASH_TIMEOUT` with `unset` status and default value `120000`
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-6](../../../docs/feature/007_params_command.md)

---

### FT-7: format::json produces valid JSON array with required fields

- **Given:** clean environment; HOME set
- **When:** `clv .params format::json`
- **Then:** exit 0; stdout is valid JSON array; each entry contains `name`, `cli_flag`, `env_var`, `config_key`, `env_value`, `config_value`, `default`, `effective`, `source` fields
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-7](../../../docs/feature/007_params_command.md)

---

### FT-8: CLI-only param shows unobservable annotation

- **Given:** clean environment; HOME set
- **When:** `clv .params key::print`
- **Then:** exit 0; output shows `--print` CLI flag form and `(CLI-only, unobservable)` annotation; no env or config value reported
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-8](../../../docs/feature/007_params_command.md)

---

### FT-9: unknown key exits 2

- **Given:** clean environment; HOME set
- **When:** `clv .params key::NONEXISTENT`
- **Then:** exit 2; stderr or stdout contains error indicating key not found in params catalog
- **Exit:** 2
- **Source:** [feature/007_params_command.md — AC-9](../../../docs/feature/007_params_command.md)

---

### FT-10: invalid kind value exits 1

- **Given:** clean environment; HOME set
- **When:** `clv .params kind::badvalue`
- **Then:** exit 1; stderr or stdout contains error referencing invalid kind value or listing valid options (`config`, `env`)
- **Exit:** 1
- **Source:** [feature/007_params_command.md — AC-10](../../../docs/feature/007_params_command.md)

---

### FT-11: default source annotation when no env or config

- **Given:** clean environment; empty `settings.json`; `CLAUDE_MODEL` unset; HOME set
- **When:** `clv .params key::model`
- **Then:** exit 0; output shows catalog default `claude-sonnet-5` annotated with `(default)` source
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-11](../../../docs/feature/007_params_command.md)

---

### FT-12: show-all output is sorted alphabetically

- **Given:** clean environment; HOME set
- **When:** `clv .params`
- **Then:** exit 0; parameter entries appear in alphabetical order by name; first entry name is lexicographically ≤ last entry name
- **Exit:** 0
- **Source:** [feature/007_params_command.md — AC-12](../../../docs/feature/007_params_command.md)
