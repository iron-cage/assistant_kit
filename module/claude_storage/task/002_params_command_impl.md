# Implement `.params` Command

## Execution State

- **Executor Type:** dev
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Closed)
- **closes:** null
- **dir:** module/claude_version/
- **validated_by:** null
- **validation_date:** null

## Goal

Implement the `.params` command: static params catalog in `claude_version_core`, command handler in `claude_version`, command registration in `lib.rs`, YAML entry in `unilang.commands.yaml`, and all 36 pending test implementations covering IT-1..14, FT-1..12, and US-1..10 scenarios. **Why now:** `docs/cli/command/params.md` is a complete command spec with exit codes, algorithms, and sample output; `feature/007_params_command.md` is marked 🔄 (In Progress) — but no Rust implementation exists. `clv .params` is unregistered, exits with "command not found". Observable end-state: `w3 .test level::3` passes with zero failures on all 36 newly-added test functions; `clv .params` shows the full parameter catalog; `feature/007_params_command.md` status updated to ✅.

## In Scope

**claude_version_core (Layer 1 — pure domain):**
- `src/params_catalog.rs`: `ParamDef` struct (name, cli_flag, env_var, config_key, default, description, kind) + `params_catalog()` function returning a static slice covering all known Claude Code parameters across config-key, env-only, and CLI-only kinds
- `src/lib.rs`: expose `pub mod params_catalog`; update module-level doc comment

**claude_version (Layer 2 — CLI):**
- `src/commands/params.rs`: `params_routine` — show-all mode (alphabetical table of all catalog params), single-param mode (`key::K` deep-dive with all forms + env read + config 4-layer resolution), `kind::` filter (`config` / `env`); exit 2 for unknown key in catalog, exit 1 for invalid `kind::` or `format::` value; output text and JSON per `docs/cli/format/`
- `src/commands/mod.rs`: `mod params; pub use params::params_routine;`
- `src/lib.rs`: add `kind` arg (`reg_arg_opt("kind", Kind::String)`); register `.params` with `[key(), kind(), v(), fmt()]`; add `.params` entry to Settings & Config help group in `print_usage()`
- `unilang.commands.yaml`: add `.params` command definition entry (args: key, kind, verbosity, format)

**Tests (claude_version/tests/):**
- `tests/integration/params_command_test.rs`: IT-1 through IT-14 per `tests/docs/cli/command/14_params.md`
- `tests/integration/params_feature_test.rs`: FT-1 through FT-12 per `tests/docs/feature/007_params_command.md`
- `tests/integration/params_user_story_test.rs`: US-1 through US-10 per `tests/docs/cli/user_story/07_params_inspection.md`
- `tests/integration.rs`: register three new test modules (`mod params_command_test; mod params_feature_test; mod params_user_story_test;`)

**Documentation status update (after passing tests):**
- `docs/feature/007_params_command.md`: status 🔄 → ✅
- `tests/docs/cli/command/14_params.md`: ⏳ markers → function names for implemented tests
- `tests/docs/feature/007_params_command.md`: ⏳ markers → function names for implemented tests
- `tests/docs/cli/user_story/07_params_inspection.md`: ⏳ markers → function names for implemented tests

## Out of Scope

- Writing to any settings file (`.params` is strictly read-only)
- `kind::` as a formal shared param with its own `docs/cli/param/` entry (inline to `.params` only; not a shared CLI parameter)
- Changes to `.config` resolution engine (already implemented; `.params` reads via the same `config_resolve` functions)
- Changes to any command other than `.params` (status, version.*, settings.*, processes.*, config remain untouched)
- All `docs/cli/` collection instance schema fixes (completed as part of this normalization session — already consistent)
- Performance benchmarks or load tests

## Requirements

- Layer 1 (`claude_version_core`) must remain free of unilang / CLI framework dependencies
- `ParamDef` must be structurally consistent with `SettingDef` pattern in `config_catalog.rs` (static str fields, `Option` for absent forms)
- Command handler follows the same unilang 5-phase pipeline pattern as `config.rs` — no direct argument parsing outside unilang
- All test functions are real integration tests (no mocks, no stubs) per testing principles
- Text output uses `key: value` label pairs (v::1) per `docs/cli/format/001_text.md` rendering rules
- JSON output is a JSON array with snake_case keys per `docs/cli/format/002_json.md`
- Custom codestyle (2-space indent, spacing conventions) applied throughout per codestyle rulebook
- All work adheres to `cli_doc_des.rulebook.md` and `doc_des.rulebook.md`

## Delivery Requirements (Work Procedure)

1. Read `src/commands/config.rs` and `claude_version_core/src/config_catalog.rs` to internalize Layer 1 and Layer 2 patterns before writing any new code
2. Create `claude_version_core/src/params_catalog.rs`: define `ParamDef` struct (fields: `name`, `cli_flag`, `env_var`, `config_key`, `default`, `description`, `kind` as `ParamKind` enum: ConfigKey / EnvOnly / CliOnly / Hybrid); implement `params_catalog()` returning `&'static [ParamDef]` with all known Claude Code parameters; expose via `claude_version_core/src/lib.rs`
3. Create `claude_version/src/commands/params.rs`: implement `params_routine` with show-all mode (read catalog, optional `kind::` filter, alphabetical sort, render each entry), single-param mode (`key::K` lookup — exit 2 if not found, then read env via `std::env::var`, resolve config via `config_resolve`, render full detail), format dispatch (text/JSON), verbosity levels per feature spec
4. Update `claude_version/src/commands/mod.rs`: add `mod params;` and `pub use params::params_routine;`
5. Update `claude_version/src/lib.rs`: add `let knd = || reg_arg_opt("kind", Kind::String);` closure; register `.params` with `vec![key(), knd(), v(), fmt()]`; add `.params` entry to Settings & Config help group; add `knd::` / `key::` to shared options block if absent
6. Update `claude_version/unilang.commands.yaml`: append `.params` command entry with args `key` (String, optional), `kind` (String, optional), `verbosity` (Integer, optional, default "1", alias "v"), `format` (String, optional, default "text", alias "fmt"); idempotent: true; http_method_hint: "GET"
7. TDD: write failing IT-1 through IT-14 tests in `params_command_test.rs` first; run `w3 .test level::1`; implement until all 14 pass
8. Implement FT-1 through FT-12 in `params_feature_test.rs`; run and fix
9. Implement US-1 through US-10 in `params_user_story_test.rs`; run and fix
10. Register new modules in `tests/integration.rs`; run `w3 .test level::3` — confirm zero failures, zero warnings
11. Update doc status markers: `feature/007_params_command.md` 🔄 → ✅; replace ⏳ entries in test spec Source Functions Tables with actual function names

## Acceptance Criteria

- AC-1: `clv .params` exits 0; stdout contains ≥35 alphabetically-sorted parameter entries each with a source annotation (`(env)`, `(user)`, `(project)`, `(default)`, `(CLI-only)`)
- AC-2: `clv .params key::model` exits 0; shows CLI form `--model`, env form `CLAUDE_MODEL`, config form `model`, current env value, current config value with scope, default, and effective value with source annotation
- AC-3: `clv .params kind::config` exits 0; only config-key params shown; env-only params (e.g., `bash_timeout`) absent
- AC-4: `clv .params kind::env` exits 0; only env-var params shown; config-only params (e.g., `theme`) absent
- AC-5: `clv .params key::NONEXISTENT` exits 2; error message references unknown key
- AC-6: `clv .params kind::badvalue` exits 1; error message references invalid kind value or valid options
- AC-7: `clv .params format::json` exits 0; stdout is a valid JSON array; each entry contains the fields `name`, `cli_flag`, `env_var`, `config_key`, `env_value`, `config_value`, `default`, `effective`, `source`
- AC-8: `clv .params key::bash_timeout` exits 0; shows env-only annotation; `CLAUDE_CODE_BASH_TIMEOUT` status (set or unset); default 120000
- AC-9: `clv .params key::print` exits 0; shows `(CLI-only, unobservable)` annotation; no env or config value reported
- AC-10: `clv .params key::model` with `CLAUDE_MODEL` unset and no config → shows default `claude-sonnet-4-6` with `(default)` source
- AC-11: All 36 test functions (IT-1..14, FT-1..12, US-1..10) implemented and passing in `w3 .test level::3`
- AC-12: `feature/007_params_command.md` status updated to ✅; Source Functions Tables in test docs updated

## Validation

**Execution:** Independent validator per validation.rulebook.md.

### Checklist

**Implementation presence**
- [x] C1 — `claude_version_core/src/params_catalog.rs` exists; `ParamDef` struct and `params_catalog()` function present
- [x] C2 — `claude_version/src/commands/params.rs` exists; `params_routine` function present
- [x] C3 — `claude_version/src/lib.rs` registers `.params` with `key`, `kind`, `verbosity`, `format` args
- [x] C4 — `unilang.commands.yaml` contains `.params` entry

**Behavioral verification**
- [x] C5 — `clv .params` exits 0; output contains ≥35 entries
- [x] C6 — `clv .params key::NONEXISTENT` exits 2
- [x] C7 — `clv .params kind::badvalue` exits 1
- [x] C8 — `clv .params format::json` produces parseable JSON array

**Test coverage**
- [x] C9 — `params_command_test.rs` contains 14 `#[test]` functions (IT-1..14)
- [x] C10 — `params_feature_test.rs` contains 12 `#[test]` functions (FT-1..12)
- [x] C11 — `params_user_story_test.rs` contains 10 `#[test]` functions (US-1..10)

### Measurements

- [x] M1 — `w3 .test level::3` → 0 failures, 0 warnings

### Invariants

- [x] I1 — Layer 1 clean: `claude_version_core/Cargo.toml` has no `unilang` dependency after changes
- [x] I2 — decisions: `task/decisions/readme.md` present (confirmed)

### Anti-faking checks

- [x] AF1 — `grep -c '#\[test\]' tests/integration/params_command_test.rs` → 14
- [x] AF2 — `grep -c '#\[test\]' tests/integration/params_feature_test.rs` → 12
- [x] AF3 — `grep -c '#\[test\]' tests/integration/params_user_story_test.rs` → 10
- [x] AF4 — `grep -c 'NONEXISTENT' tests/integration/params_command_test.rs` → ≥1 (IT-7 exit-2 test present)

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clv .params` | Clean env, empty settings | Exit 0; ≥35 entries alphabetically sorted; each has source annotation |
| `clv .params` | Any | Entries ordered a..z; first param name ≤ last param name lexicographically |
| `clv .params key::model` | `CLAUDE_MODEL` unset, user config has `model` | Exit 0; CLI/env/config forms shown; config value with `(user)` source |
| `clv .params key::model` | `CLAUDE_MODEL=claude-opus-4-6` set | Exit 0; env value `claude-opus-4-6` shown with `(env)` source |
| `clv .params key::model` | No env, no config | Exit 0; default `claude-sonnet-4-6` shown with `(default)` source |
| `clv .params key::bash_timeout` | `CLAUDE_CODE_BASH_TIMEOUT` unset | Exit 0; env-only entry; `unset` status; default 120000 |
| `clv .params key::print` | Any | Exit 0; CLI-only annotation; no env or config value shown |
| `clv .params kind::config` | Clean env | Exit 0; only config-key params; `bash_timeout` (env-only) absent |
| `clv .params kind::env` | Clean env | Exit 0; only env-var params; `theme` (config-only) absent |
| `clv .params format::json` | Clean env | Exit 0; valid JSON array; all required fields present in every entry |
| `clv .params key::NONEXISTENT` | Any | Exit 2; error references unknown key or "not found in catalog" |
| `clv .params kind::badvalue` | Any | Exit 1; error references invalid kind value |
| `clv .params key::model format::json` | Any | Exit 0; valid JSON object; all fields present; `source` field present |

## Related Documentation

- `module/claude_version/docs/feature/007_params_command.md` — authoritative feature spec (operating modes, observable vs unobservable, algorithms, output format, exit codes)
- `module/claude_version/docs/cli/command/params.md` — command reference (parameters, syntax, examples, exit codes)
- `module/claude_version/docs/cli/user_story/007_params_inspection.md` — user story scenario (persona, acceptance criteria)
- `module/claude_version/tests/docs/cli/command/14_params.md` — IT-1..14 integration test spec
- `module/claude_version/tests/docs/feature/007_params_command.md` — FT-1..12 feature test spec
- `module/claude_version/tests/docs/cli/user_story/07_params_inspection.md` — US-1..10 acceptance test spec
- `module/claude_version/docs/algorithm/002_config_resolution.md` — 4-layer resolution algorithm used for config-layer values in single-param mode

## History

- **[2026-06-29]** `CREATED` — Task filed. Goal: implement `.params` command, params catalog, and all 36 pending test cases defined during the cli_doc_des normalization session.
- **[2026-06-29]** `VERIFY_PASS` — Verification Gate passed (4/4 dimensions PASS). Moved to 🎯 Verified.
- **[2026-06-30]** `CLOSED` — Implementation complete. 566/566 tests pass (0 failures, 0 warnings). All 36 test functions implemented (IT-1..14, FT-1..12, US-1..10). MAAV validation passed: Validator PASS; Adversarial found A1 (empty `kind::` bug, fixed) and A3 (latent test isolation risk, noted). Feature 007 status updated ✅. Moved to ✅ Closed.

## Verification Record

| Dimension | Result | Agent ID |
|-----------|--------|----------|
| Scope Coherence | PASS | aa235a74e1a5db880 |
| MOST Goal Quality | PASS | ab31a9cc41bb73083 |
| Value / YAGNI | PASS | a7e16379a8fb25403 |
| Implementation Readiness | PASS | ab0e75eb43f911b5b |
