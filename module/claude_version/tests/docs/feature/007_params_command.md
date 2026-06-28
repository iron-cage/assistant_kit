# Feature Test: Params Command

### Scope

- **Purpose**: FT- test cases for the `.params` command — show-all, single-param, kind filter, JSON format, and error paths.
- **Responsibility**: Acceptance criteria verifying param catalog coverage, observable value resolution, CLI-only annotation, kind filtering, and exit codes.
- **In Scope**: All `.params` modes, kind:: filter, format::json, env var reads, config reads, CLI-only marking, exit codes.
- **Out of Scope**: Config write operations (→ `06_config_command.md`), resolution algorithm unit tests (→ `../../algorithm/02_config_resolution.md`).

Feature test surface for `.params`. See [feature/007_params_command.md](../../../docs/feature/007_params_command.md) for specification.

## Behavioral Divergence Pair

Two valid `.params` invocations that produce structurally different output:

- **Input A:** `clv.params` (no params) → table of all catalog params with current observable values
- **Input B:** `clv.params key::model` → deep-dive block for one param, showing all forms and effective value

## Test Case Index

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `.params` show-all exits 0 with ≥35 entries, each annotated | ⏳ `ft01_007_params_show_all_min_entries` |
| FT-02 | AC-02 | `.params key::model` shows all three forms, current values, and default | ⏳ `ft02_007_params_single_model_full_detail` |
| FT-03 | AC-03 | `.params kind::config` shows only config-key params; env-only params absent | ⏳ `ft03_007_params_kind_config_filters` |
| FT-04 | AC-04 | `.params kind::env` shows only env-var params; config-only params absent | ⏳ `ft04_007_params_kind_env_filters` |
| FT-05 | AC-05 | `.params key::model` with CLAUDE_MODEL set shows env value with (env) annotation | ⏳ `ft05_007_params_env_override_visible` |
| FT-06 | AC-06 | `.params key::bash_timeout` shows CLAUDE_CODE_BASH_TIMEOUT unset + default 120000 | ⏳ `ft06_007_params_env_only_param` |
| FT-07 | AC-07 | `.params format::json` exits 0; output is valid JSON array with required fields | ⏳ `ft07_007_params_json_output_structure` |
| FT-08 | AC-08 | `.params key::print` shows CLI-only annotation | ⏳ `ft08_007_params_cli_only_annotation` |
| FT-09 | AC-09 | `.params key::NONEXISTENT` exits 2 | ⏳ `ft09_007_params_unknown_key_exits_2` |
| FT-10 | AC-10 | `.params kind::badvalue` exits 1 | ⏳ `ft10_007_params_invalid_kind_exits_1` |
| FT-11 | AC-11 | `.params key::model` with no env and no config → shows default with (default) annotation | ⏳ `ft11_007_params_default_source_annotation` |
| FT-12 | AC-12 | `.params` show-all output is sorted alphabetically | ⏳ `ft12_007_params_show_all_alphabetical` |

## Test Coverage Summary

- Show-all mode: 2 tests (FT-01, FT-12)
- Single-param mode: 3 tests (FT-02, FT-05, FT-11)
- Kind filter: 2 tests (FT-03, FT-04)
- Env-var params: 2 tests (FT-05, FT-06)
- CLI-only annotation: 1 test (FT-08)
- JSON format: 1 test (FT-07)
- Error paths: 2 tests (FT-09, FT-10)

**Total:** 12 tests

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| *(none yet — implementation pending)* | — | — |
