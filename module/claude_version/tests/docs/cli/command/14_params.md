# Test: `.params`

### Scope

- **Purpose**: Integration test cases for the `.params` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for show-all, single-param, kind filter, format, and error modes.
- **In Scope**: Mode dispatch, kind filter, format, verbosity, env var reads, config reads, CLI-only annotation, exit codes.
- **Out of Scope**: Params catalog unit tests (→ coverage in `claude_version_core` crate tests), resolution algorithm tests (→ `../../algorithm/02_config_resolution.md`).

Integration test planning for `.params`. See [command/params.md](../../../../docs/cli/command/params.md) for specification.

## Test Factor Analysis

### Factor 1: Mode (derived from key:: presence)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| no key:: | show-all mode | Default |
| key::K (known) | single-param mode | Happy path |
| key::K (unknown) | not in catalog | Error: exit 2 |

### Factor 2: `kind::` (String, optional, show-all only)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | All params | Default |
| `config` | Config-key params only | Valid filter |
| `env` | Env-var params only | Valid filter |
| other | Unrecognized value | Invalid: exit 1 |

### Factor 3: `format::` (String, optional, default text)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent / `text` | Human-readable text | Default |
| `json` | Structured JSON array | Alternate valid |
| other | Unrecognized value | Invalid: exit 1 |

### Factor 4: `v::` (VerbosityLevel, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| 0 | Values only (compact) | Valid |
| 1 | Forms + values (default) | Default |
| 2 | Full with descriptions | Valid |

### Factor 5: Env var state (for params with env form)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Env var present and non-empty | Happy path |
| unset | Env var absent | Default — shows "unset" |

---

## Test Matrix

### Positive Tests

| TC | Description | Mode | Exit | Factors |
|----|-------------|------|------|---------|
| IT-1 | No params → show-all with ≥35 entries, each annotated | show-all | 0 | F1=no-key |
| IT-2 | `key::model` → block with CLI, env, config forms + default | single | 0 | F1=key-known |
| IT-3 | `kind::config` → only config-key params; env-only absent | show-all | 0 | F2=config |
| IT-4 | `kind::env` → only env-var params; config-only absent | show-all | 0 | F2=env |
| IT-5 | `key::model` with CLAUDE_MODEL set → env value shown with (env) | single | 0 | F5=set |
| IT-6 | `key::bash_timeout` → env CLAUDE_CODE_BASH_TIMEOUT → unset, default 120000 | single | 0 | F5=unset |
| IT-7 | `format::json` → valid JSON array with required fields per entry | show-all | 0 | F3=json |
| IT-8 | `key::print` → shows --print CLI form + CLI-only annotation | single | 0 | F1=key-known |
| IT-9 | `v::0` → compact one-line-per-param output | show-all | 0 | F4=0 |
| IT-10 | `key::model` no env no config → default with (default) annotation | single | 0 | F5=unset |
| IT-11 | Show-all output is alphabetically sorted | show-all | 0 | F1=no-key |

### Negative Tests

| TC | Description | Mode | Exit | Factors |
|----|-------------|------|------|---------|
| IT-12 | `key::NONEXISTENT_KEY` → exit 2 | — | 2 | F1=key-unknown |
| IT-13 | `kind::badvalue` → exit 1 | — | 1 | F2=invalid |
| IT-14 | `format::xml` → exit 1 | — | 1 | F3=invalid |

### Summary

- **Total:** 14 tests (11 positive, 3 negative)
- **Negative ratio:** 21.4% (error paths are structurally limited for a read-only command)
- **TC range:** IT-1 to IT-14

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-1 through IT-11 |
| 1 | Invalid arguments | IT-13, IT-14 |
| 2 | Key not in catalog | IT-12 |

### Mode Coverage

| Mode | Tests |
|------|-------|
| show-all | IT-1, IT-3, IT-4, IT-7, IT-9, IT-11 |
| single | IT-2, IT-5, IT-6, IT-8, IT-10 |

---

## Test Case Details

---

### IT-1: No params → show-all ≥35 entries

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params`
- **Then:** exit 0; stdout contains at least 35 distinct parameter names; each entry has a source annotation or `(CLI-only)` marker
- **Exit:** 0
- **Source:** [command/params.md](../../../../docs/cli/command/params.md)

---

### IT-2: key::model deep-dive

- **Given:** `HOME=<tmp>` (no settings.json), `CLAUDE_MODEL` not set
- **When:** `clv.params key::model`
- **Then:** exit 0; stdout contains strings `--model`, `CLAUDE_MODEL`, `config model`, and `claude-sonnet-5` (default)
- **Exit:** 0

---

### IT-3: kind::config → only config-key params

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params kind::config`
- **Then:** exit 0; stdout contains params with a config key form (e.g., `model`, `theme`, `autoUpdates`); env-only params (e.g., `bash_timeout`) are absent
- **Exit:** 0

---

### IT-4: kind::env → only env-var params

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params kind::env`
- **Then:** exit 0; stdout contains params with an env var form (e.g., `model`, `bash_timeout`); config-only params (e.g., `theme`) are absent
- **Exit:** 0

---

### IT-5: key::model with CLAUDE_MODEL set

- **Given:** `HOME=<tmp>`, `CLAUDE_MODEL=claude-opus-4-8` in env
- **When:** `clv.params key::model`
- **Then:** exit 0; stdout contains `claude-opus-4-8` and `(env)` annotation
- **Exit:** 0

---

### IT-6: key::bash_timeout → env-only param, unset

- **Given:** `HOME=<tmp>`, `CLAUDE_CODE_BASH_TIMEOUT` not set
- **When:** `clv.params key::bash_timeout`
- **Then:** exit 0; stdout shows `CLAUDE_CODE_BASH_TIMEOUT` env form, value `unset`, default `120000`; no config key form present
- **Exit:** 0

---

### IT-7: format::json → valid JSON array

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params format::json`
- **Then:** exit 0; stdout is valid JSON parseable as an array; each element has at minimum a `name` key
- **Exit:** 0

---

### IT-8: key::print → CLI-only annotation

- **Given:** `HOME=<tmp>`
- **When:** `clv.params key::print`
- **Then:** exit 0; stdout shows `-p / --print` form and a `CLI-only` annotation; no env value or config key shown
- **Exit:** 0

---

### IT-9: v::0 → compact values-only output

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params v::0`
- **Then:** exit 0; stdout contains one entry per param with values only; no form labels or source annotations
- **Exit:** 0

---

### IT-10: key::model no env no config → default annotation

- **Given:** `HOME=<tmp>` (no settings.json), `CLAUDE_MODEL` not set
- **When:** `clv.params key::model`
- **Then:** exit 0; stdout contains `claude-sonnet-5` with `(default)` annotation; no env or config value shown
- **Exit:** 0

---

### IT-11: Show-all output is alphabetically sorted

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params`
- **Then:** exit 0; parameter names extracted from stdout appear in ascending alphabetical order
- **Exit:** 0

---

### IT-12: Unknown key exits 2

- **Given:** `HOME=<tmp>`
- **When:** `clv.params key::NONEXISTENT_KEY`
- **Then:** exit 2; stderr contains the unknown key token
- **Exit:** 2

---

### IT-13: kind::badvalue → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params kind::badvalue`
- **Then:** exit 1; stderr contains a message indicating valid values are `config` or `env`
- **Exit:** 1

---

### IT-14: format::xml → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params format::xml`
- **Then:** exit 1; stderr contains a message indicating valid values are `text` or `json`
- **Exit:** 1

---

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| `it01_params_show_all_min_entries` | `tests/cli/params_command_test.rs` | IT-1 |
| `it02_params_single_model_full_detail` | `tests/cli/params_command_test.rs` | IT-2 |
| `it03_params_kind_config_filters` | `tests/cli/params_command_test.rs` | IT-3 |
| `it04_params_kind_env_filters` | `tests/cli/params_command_test.rs` | IT-4 |
| `it05_params_env_override_visible` | `tests/cli/params_command_test.rs` | IT-5 |
| `it06_params_env_only_param_unset` | `tests/cli/params_command_test.rs` | IT-6 |
| `it07_params_json_output_structure` | `tests/cli/params_command_test.rs` | IT-7 |
| `it08_params_cli_only_annotation` | `tests/cli/params_command_test.rs` | IT-8 |
| `it09_params_compact_v0_output` | `tests/cli/params_command_test.rs` | IT-9 |
| `it10_params_default_annotation` | `tests/cli/params_command_test.rs` | IT-10 |
| `it11_params_show_all_alphabetical` | `tests/cli/params_command_test.rs` | IT-11 |
| `it12_params_unknown_key_exits_2` | `tests/cli/params_command_test.rs` | IT-12 |
| `it13_params_invalid_kind_exits_1` | `tests/cli/params_command_test.rs` | IT-13 |
| `it14_params_invalid_format_exits_1` | `tests/cli/params_command_test.rs` | IT-14 |
