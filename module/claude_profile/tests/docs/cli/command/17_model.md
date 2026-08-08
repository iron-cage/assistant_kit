# IT — Command 18: `.model`

### Scope

- **Purpose**: CLI-level integration test cases for the unified `scope::`-routed `.model` get/set/reset command — syntax forms, parameter dispatch, and output shape.
- **Source**: `docs/cli/command/007_model.md` (primary — Command 18), `docs/feature/035_model_command.md` (secondary — behavioral contract, AC-01 through AC-27)
- **Covers**: AC-01 through AC-27 (AC-28 — `claude_profile_core::account::remove_session_effort()` — is task 464's own deliverable; not re-verified here)

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-01 | `.model` (no params) → `scope::` defaults `session`, both fields shown | ✅ `t01_get_default_scope_is_session` |
| IT-02 | AC-02 | `.model scope::subprocess` → get mode, config.toml path shown | ✅ `t02_get_subprocess_scope` |
| IT-03 | AC-03 | `.model scope::bad` → exit 1, names `session`/`subprocess` | ✅ `t03_get_invalid_scope_exits_1` |
| IT-04 | AC-04 | `.model model::opus` → shorthand resolves to `claude-opus-4-8` | ✅ `t04_set_model_session_each_shorthand` |
| IT-05 | AC-05 | `.model model::sonnet` → shorthand resolves to `claude-sonnet-5` | ✅ `t04_set_model_session_each_shorthand` |
| IT-06 | AC-06 | `.model model::haiku` → shorthand resolves to `claude-haiku-4-5-20251001` | ✅ `t04_set_model_session_each_shorthand` |
| IT-07 | AC-07 | `.model model::default` → removes `model` key from settings.json | ✅ `t05_set_model_session_default_removes_key` |
| IT-08 | AC-08 | `.model model::bad` → exit 1, vocabulary named in stderr | ✅ `t06_set_model_session_invalid_exits_1` |
| IT-09 | AC-09 | `.model scope::subprocess model::claude-opus-4-8` → writes config.toml | ✅ `t07_set_model_subprocess_writes_config_toml` |
| IT-10 | AC-10 | `.model scope::subprocess model::` (empty) → exit 1 | ✅ `t08_set_model_subprocess_empty_exits_1` |
| IT-11 | AC-11 | `.model effort_level::high` → writes `effortLevel` to settings.json | ✅ `t09_set_effort_session_writes_effort_level` |
| IT-12 | AC-12 | `.model effort_level::bad` → exit 1, vocabulary named in stderr | ✅ `t10_set_effort_session_invalid_exits_1` |
| IT-13 | AC-13 | `.model scope::subprocess effort_level::medium` → writes `effort` to config.toml | ✅ `t11_set_effort_subprocess_writes_config_toml` |
| IT-14 | AC-14 | `.model scope::subprocess effort_level::normal` → exit 1 (session-only value rejected) | ✅ `t12_set_effort_subprocess_session_only_value_exits_1` |
| IT-15 | AC-15 | `.model reset_model::1` → removes `model` key | ✅ `t13_reset_model_session_removes_key` |
| IT-16 | AC-16 | `.model reset_effort_level::1` → removes `effortLevel` key | ✅ `t14_reset_effort_session_removes_key` |
| IT-17 | AC-17 | `.model scope::subprocess reset_model::1` → idempotent | ✅ `t15_reset_model_subprocess_idempotent` |
| IT-18 | AC-18 | `.model scope::subprocess reset_effort_level::1` → idempotent | ✅ `t16_reset_effort_subprocess_idempotent` |
| IT-19 | AC-19 | `.model model::opus reset_model::1` → exit 1, mutual exclusion named | ✅ `t17_mutual_exclusion_model_exits_1` |
| IT-20 | AC-20 | `.model effort_level::high reset_effort_level::1` → exit 1, mutual exclusion named | ✅ `t18_mutual_exclusion_effort_exits_1` |
| IT-21 | AC-21 | `.model model::opus reset_effort_level::1` → both actions apply, exit 0 | ✅ `t19_combine_across_concepts` |
| IT-22 | AC-22 | `.model scope::subprocess model::claude-opus-4-8 effort_level::max` → both written in one call | ✅ `t20_combine_within_subprocess_scope_preserves_keys` |
| IT-23 | AC-23 | `.model format::json` → parses to documented JSON shape | ✅ `t21_json_format_shape` |
| IT-24 | AC-24 | `.model scope::subprocess model::VALUE`, `.clr/` absent → dir + file created | ✅ `t22_subprocess_creates_missing_dir_and_file` |
| IT-25 | AC-25 | Absolute path disclosure across every mode | ✅ cross-cutting, see Notes |
| IT-26 | AC-26 | `.model` listed once in `clp .help`; `.model.select` no longer a distinct row | ✅ `dot04_all_visible_commands_present` / `dot05_exactly_fourteen_command_rows` / `dot13_model_select_hidden_from_listing` (`tests/cli/dot_test.rs`) |
| IT-27 | AC-27 | No inline I/O duplication — reuses shared primitives | ✅ architectural constraint, verified by code review |

### Notes

- All IT cases exercise the compiled `clp` binary as a subprocess (`tests/cli/model_test.rs`), matching this crate's CLI-integration-test convention (`cli_runner.rs`'s `run_cs_with_env`).
- IT-04/05/06 share one source fn — the three shorthand forms are one parametrized loop in `t04_set_model_session_each_shorthand`, not three separate functions.
- IT-25 (AC-25) is a cross-cutting property, not one dedicated test — every get/write-mode assertion elsewhere in this table already pins the exact resolved path string in its expected output.
- IT-26 (AC-26) lives in `dot_test.rs`, not `model_test.rs` — command-listing/hiding is CLI-wide infrastructure (`hidden_from_list`), shared across all commands, not `.model`-specific.
- IT-27 (AC-27) is verified by code review of `src/commands/model.rs`, not a runtime test — see `docs/cli/command/007_model.md`'s own Notes on reused primitives.
- Full syntax reference (11 example invocations), the Parameters table (`scope::`, `model::`, `effort_level::`, `reset_model::`, `reset_effort_level::`, `format::`), and the get/write Algorithm steps live in `docs/cli/command/007_model.md` — not restated here; each IT case below cites the specific syntax form it exercises.
- `.model.select`'s own retirement-stub behavior is covered separately in `20_model_select.md` / `tests/cli/model_select_test.rs` — not duplicated here.

---

### IT-01: `.model` — bare invocation, get mode, default scope

- **Given:** Fresh `HOME`; no `settings.json`.
- **When:** `clp .model`
- **Then:** `scope::` parameter omitted → defaults to `session` per the Parameters table. Stdout: `scope: session (<path>)\nmodel: (unset)\neffort_level: (unset)\n`. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t01_get_default_scope_is_session`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-02: `.model scope::subprocess` — get mode

- **Given:** Fresh `HOME`; no `.clr/config.toml`.
- **When:** `clp .model scope::subprocess`
- **Then:** Stdout: `scope: subprocess (<path>)\nmodel: (unset)\neffort_level: (unset)\n`. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t02_get_subprocess_scope`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-03: `.model scope::bad` — invalid scope rejected

- **Given:** Any environment.
- **When:** `clp .model scope::bad`
- **Then:** Exit 1. Stderr names `session` and `subprocess` as the only valid values.
- **Exit:** 1
- **Source fn:** ✅ `t03_get_invalid_scope_exits_1`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-04: `.model model::opus` — shorthand dispatch

- **Given:** Fresh `HOME`.
- **When:** `clp .model model::opus`
- **Then:** Resolves via the shorthand table to `claude-opus-4-8`, written to `settings.json`. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t04_set_model_session_each_shorthand`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-05: `.model model::sonnet` — shorthand dispatch

- **Given:** Fresh `HOME`.
- **When:** `clp .model model::sonnet`
- **Then:** Resolves to `claude-sonnet-5`, written to `settings.json`. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t04_set_model_session_each_shorthand`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-06: `.model model::haiku` — shorthand dispatch

- **Given:** Fresh `HOME`.
- **When:** `clp .model model::haiku`
- **Then:** Resolves to `claude-haiku-4-5-20251001`, written to `settings.json`. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t04_set_model_session_each_shorthand`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-07: `.model model::default` — removal shorthand

- **Given:** `settings.json` contains `{"model":"claude-sonnet-5","theme":"dark"}`.
- **When:** `clp .model model::default`
- **Then:** `"model"` key removed; `"theme":"dark"` untouched. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t05_set_model_session_default_removes_key`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-08: `.model model::bad` — invalid session model rejected

- **Given:** Any environment.
- **When:** `clp .model model::bad`
- **Then:** Exit 1. Stderr lists `opus`, `sonnet`, `haiku`, `default`.
- **Exit:** 1
- **Source fn:** ✅ `t06_set_model_session_invalid_exits_1`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-09: `.model scope::subprocess model::claude-opus-4-8` — subprocess write

- **Given:** Fresh `HOME`.
- **When:** `clp .model scope::subprocess model::claude-opus-4-8`
- **Then:** `config.toml`'s user tier gets `model = "claude-opus-4-8"` verbatim (no shorthand table for subprocess scope — full IDs only). Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t07_set_model_subprocess_writes_config_toml`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-10: `.model scope::subprocess model::` — empty value rejected

- **Given:** Fresh `HOME`.
- **When:** `clp .model scope::subprocess model::`
- **Then:** Exit 1. Stderr requires a non-empty model ID. No `config.toml` written.
- **Exit:** 1
- **Source fn:** ✅ `t08_set_model_subprocess_empty_exits_1`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-11: `.model effort_level::high` — session effort write

- **Given:** Fresh `HOME`.
- **When:** `clp .model effort_level::high`
- **Then:** `settings.json` gets `"effortLevel":"high"`. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t09_set_effort_session_writes_effort_level`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-12: `.model effort_level::bad` — invalid session effort rejected

- **Given:** Any environment.
- **When:** `clp .model effort_level::bad`
- **Then:** Exit 1. Stderr lists `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** ✅ `t10_set_effort_session_invalid_exits_1`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-13: `.model scope::subprocess effort_level::medium` — subprocess effort write

- **Given:** Fresh `HOME`.
- **When:** `clp .model scope::subprocess effort_level::medium`
- **Then:** `config.toml`'s user tier gets `effort = "medium"`. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t11_set_effort_subprocess_writes_config_toml`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-14: `.model scope::subprocess effort_level::normal` — session-only value rejected

- **Given:** Any environment.
- **When:** `clp .model scope::subprocess effort_level::normal`
- **Then:** Exit 1 — `normal`/`high` (session vocabulary) differ from `low`/`medium`/`high`/`max` (subprocess vocabulary); `normal` is not valid for `scope::subprocess`. Stderr lists `low`, `medium`, `high`, `max`.
- **Exit:** 1
- **Source fn:** ✅ `t12_set_effort_subprocess_session_only_value_exits_1`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-15: `.model reset_model::1` — session reset

- **Given:** `settings.json` contains `{"model":"claude-opus-4-8"}`.
- **When:** `clp .model reset_model::1`
- **Then:** `"model"` key removed. Stdout confirms `model: (reset)  →  <path> (session)`. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t13_reset_model_session_removes_key`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-16: `.model reset_effort_level::1` — session reset

- **Given:** `settings.json` contains `{"effortLevel":"high"}`.
- **When:** `clp .model reset_effort_level::1`
- **Then:** `"effortLevel"` key removed via `remove_session_effort()` (task 464). Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t14_reset_effort_session_removes_key`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-17: `.model scope::subprocess reset_model::1` — idempotent subprocess reset

- **Given:** Fresh `HOME` — no `.clr/` at all.
- **When:** `clp .model scope::subprocess reset_model::1`, twice in sequence.
- **Then:** Both calls exit 0 — resetting an already-absent key is not an error.
- **Exit:** 0
- **Source fn:** ✅ `t15_reset_model_subprocess_idempotent`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-18: `.model scope::subprocess reset_effort_level::1` — idempotent subprocess reset

- **Given:** Fresh `HOME` — no `.clr/` at all.
- **When:** `clp .model scope::subprocess reset_effort_level::1`, twice in sequence.
- **Then:** Both calls exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t16_reset_effort_subprocess_idempotent`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-19: `.model model::opus reset_model::1` — same-concept conflict

- **Given:** Any environment.
- **When:** `clp .model model::opus reset_model::1`
- **Then:** Exit 1. Stderr states `model::` and `reset_model::1` are mutually exclusive.
- **Exit:** 1
- **Source fn:** ✅ `t17_mutual_exclusion_model_exits_1`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-20: `.model effort_level::high reset_effort_level::1` — same-concept conflict

- **Given:** Any environment.
- **When:** `clp .model effort_level::high reset_effort_level::1`
- **Then:** Exit 1. Stderr states `effort_level::` and `reset_effort_level::1` are mutually exclusive.
- **Exit:** 1
- **Source fn:** ✅ `t18_mutual_exclusion_effort_exits_1`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-21: `.model model::opus reset_effort_level::1` — cross-concept combination

- **Given:** `settings.json` contains `{"effortLevel":"max"}`.
- **When:** `clp .model model::opus reset_effort_level::1`
- **Then:** `model::` and `reset_effort_level::` target different concepts — both apply in one call. `"model":"claude-opus-4-8"` written, `"effortLevel"` removed. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t19_combine_across_concepts`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-22: `.model scope::subprocess model::... effort_level::...` — single-call combination

- **Given:** `config.toml` already has `provider = "kimi"`.
- **When:** `clp .model scope::subprocess model::claude-opus-4-8 effort_level::max`
- **Then:** Both `model` and `effort` written in the same call; `provider = "kimi"` preserved. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t20_combine_within_subprocess_scope_preserves_keys`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-23: `.model format::json` — JSON output shape

- **Given:** Fresh `HOME`.
- **When:** `clp .model format::json`
- **Then:** Stdout parses as `{"scope":"session","path":"<path>","model":null,"effort_level":null}` — matches the Formats table in `docs/cli/command/007_model.md`.
- **Exit:** 0
- **Source fn:** ✅ `t21_json_format_shape`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-24: Subprocess write creates missing directory and file

- **Given:** Fresh `HOME` — `~/.clr/` absent.
- **When:** `clp .model scope::subprocess model::claude-haiku-4-5-20251001`
- **Then:** `~/.clr/` and `config.toml` both created. Exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t22_subprocess_creates_missing_dir_and_file`
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-25: Absolute path disclosure (cross-cutting)

- **Given:** Any scope, any mode.
- **When:** Any `.model` invocation.
- **Then:** Output names the fully resolved absolute path — never `~`-relative or omitted (Notes bullet in `docs/cli/command/007_model.md`).
- **Exit:** n/a
- **Source fn:** ✅ cross-cutting (see Notes)
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-26: `.model` listed once; `.model.select` no longer distinct

- **Given:** Any environment.
- **When:** `clp .help` (or `clp .`)
- **Then:** `.model` appears exactly once; `.model.select` does not appear as a listed row (still dispatchable — see `20_model_select.md`).
- **Exit:** 0
- **Source fn:** ✅ `dot04_all_visible_commands_present` / `dot05_exactly_fourteen_command_rows` / `dot13_model_select_hidden_from_listing` (`tests/cli/dot_test.rs`)
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)

---

### IT-27: No inline I/O duplication (architectural)

- **Given:** `src/commands/model.rs` source.
- **When:** Code review.
- **Then:** All reads/writes go through `claude_profile_core::account::*` (session) or `claude_core::toml_io::*` (subprocess) — no inline re-implementation.
- **Exit:** n/a
- **Source fn:** ✅ architectural constraint, verified by code review
- **Source:** [007_model.md — Command 18](../../../../docs/cli/command/007_model.md)
