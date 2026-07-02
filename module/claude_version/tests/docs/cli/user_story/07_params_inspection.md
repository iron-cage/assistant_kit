# Test: Params Inspection

Acceptance tests for User Story 007. See [user_story/007_params_inspection.md](../../../../docs/cli/user_story/007_params_inspection.md) for specification.

### Scope

- **Purpose**: Verify `.params` provides full Claude Code parameter catalog inspection with observable values and form annotations.
- **Responsibility**: Acceptance criteria coverage for the params inspection workflow.
- **Commands:** `.params`
- **In Scope**: Show-all mode, single-param deep-dive, kind filter, env var reads, CLI-only annotation, JSON output.
- **Out of Scope**: Config write operations (-> `../command/13_config.md`), parameter edge cases (-> `../param/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.params` shows ≥35 entries with source annotations | Acceptance: show-all |
| US-2 | `.params key::model` shows all forms and default | Acceptance: single-param |
| US-3 | `.params kind::config` shows only config-key params | Acceptance: kind filter |
| US-4 | `.params kind::env` shows only env-var params | Acceptance: kind filter |
| US-5 | `.params key::model` with env override shows (env) annotation | Acceptance: env priority |
| US-6 | `.params key::print` shows CLI-only annotation | Acceptance: CLI-only marking |
| US-7 | `.params format::json` returns valid JSON array with required fields | Acceptance: JSON output |
| US-8 | `.params key::UNKNOWN` exits 2 | Acceptance: error handling |
| US-9 | `.params kind::bad` exits 1 | Acceptance: error handling |
| US-10 | Show-all output is alphabetically sorted | Acceptance: ordering |

## Test Coverage Summary

- Show-all: 2 tests (US-1, US-10)
- Single-param: 2 tests (US-2, US-5)
- Kind filter: 2 tests (US-3, US-4)
- CLI-only: 1 test (US-6)
- JSON output: 1 test (US-7)
- Error paths: 2 tests (US-8, US-9)

**Total:** 10 tests

## Test Case Details

---

### US-1: `.params` shows ≥35 entries with source annotations

- **Given:** `HOME=<tmp>` (no settings.json); no special env vars set
- **When:** `clv.params`
- **Then:** exit 0; stdout contains at least 35 distinct parameter names; each entry has a source annotation or `(CLI-only)` marker
- **Exit:** 0

---

### US-2: `.params key::model` shows all forms and default

- **Given:** `HOME=<tmp>` (no settings.json), `CLAUDE_MODEL` not set
- **When:** `clv.params key::model`
- **Then:** exit 0; stdout contains `--model`, `CLAUDE_MODEL`, `config model`, default `claude-sonnet-5`, and a `(default)` annotation
- **Exit:** 0

---

### US-3: `.params kind::config` shows only config-key params

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params kind::config`
- **Then:** exit 0; stdout contains params with a config key form (e.g., `model`, `theme`); env-only params (e.g., `bash_timeout`) are absent
- **Exit:** 0

---

### US-4: `.params kind::env` shows only env-var params

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params kind::env`
- **Then:** exit 0; stdout contains params with an env var form (e.g., `model`, `bash_timeout`); config-only params (e.g., `theme`) are absent
- **Exit:** 0

---

### US-5: `.params key::model` with env override shows (env) annotation

- **Given:** `HOME=<tmp>`, `CLAUDE_MODEL=claude-opus-4-8` in env
- **When:** `clv.params key::model`
- **Then:** exit 0; stdout contains `claude-opus-4-8` with `(env)` annotation indicating env layer wins
- **Exit:** 0

---

### US-6: `.params key::print` shows CLI-only annotation

- **Given:** `HOME=<tmp>`
- **When:** `clv.params key::print`
- **Then:** exit 0; stdout shows `-p / --print` form with a `CLI-only` annotation; no env value or config key shown
- **Exit:** 0

---

### US-7: `.params format::json` returns valid JSON array with required fields

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params format::json`
- **Then:** exit 0; stdout is valid JSON parseable as an array; each element has at minimum a `name` key
- **Exit:** 0

---

### US-8: `.params key::UNKNOWN` exits 2

- **Given:** `HOME=<tmp>`
- **When:** `clv.params key::UNKNOWN`
- **Then:** exit 2; stderr contains the unknown key token indicating it is not in the params catalog
- **Exit:** 2

---

### US-9: `.params kind::bad` exits 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.params kind::bad`
- **Then:** exit 1; stderr contains a message indicating valid values are `config` or `env`
- **Exit:** 1

---

### US-10: Show-all output is alphabetically sorted

- **Given:** `HOME=<tmp>` (no settings.json)
- **When:** `clv.params`
- **Then:** exit 0; parameter names extracted from stdout appear in ascending alphabetical order
- **Exit:** 0

---

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| `us01_007_params_show_all_entries` | `tests/cli/user_story_test.rs` | US-1 |
| `us02_007_params_single_model_forms` | `tests/cli/user_story_test.rs` | US-2 |
| `us03_007_params_kind_config_only` | `tests/cli/user_story_test.rs` | US-3 |
| `us04_007_params_kind_env_only` | `tests/cli/user_story_test.rs` | US-4 |
| `us05_007_params_env_override_annotated` | `tests/cli/user_story_test.rs` | US-5 |
| `us06_007_params_cli_only_print` | `tests/cli/user_story_test.rs` | US-6 |
| `us07_007_params_json_array_output` | `tests/cli/user_story_test.rs` | US-7 |
| `us08_007_params_unknown_key_exits_2` | `tests/cli/user_story_test.rs` | US-8 |
| `us09_007_params_invalid_kind_exits_1` | `tests/cli/user_story_test.rs` | US-9 |
| `us10_007_params_show_all_alphabetical` | `tests/cli/user_story_test.rs` | US-10 |
