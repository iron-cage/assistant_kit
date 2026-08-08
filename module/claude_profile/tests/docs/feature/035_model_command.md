# FT — Feature 035: Unified Model & Effort Command

### Scope

- **Purpose**: Test cases for the unified `scope::`-routed `.model` get/set/reset command.
- **Source**: `docs/feature/035_model_command.md`
- **Covers**: AC-01 through AC-27 (AC-28 — `claude_profile_core::account::remove_session_effort()` itself — is task 464's own deliverable, covered by that crate's own test suite; reused here only as a dependency, not re-verified)

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | Get, `scope::` omitted (defaults `session`) → `scope: session (<path>)`, both fields shown | ✅ `t01_get_default_scope_is_session` |
| FT-02 | AC-02 | Get, `scope::subprocess` → `scope: subprocess (<path>)`, both fields shown | ✅ `t02_get_subprocess_scope` |
| FT-03 | AC-03 | `scope::bad` → exit 1, stderr names `session`/`subprocess` | ✅ `t03_get_invalid_scope_exits_1` |
| FT-04 | AC-04 | `model::opus` (session) → writes `claude-opus-4-8` | ✅ `t04_set_model_session_each_shorthand` |
| FT-05 | AC-05 | `model::sonnet` (session) → writes `claude-sonnet-5` | ✅ `t04_set_model_session_each_shorthand` |
| FT-06 | AC-06 | `model::haiku` (session) → writes `claude-haiku-4-5-20251001` | ✅ `t04_set_model_session_each_shorthand` |
| FT-07 | AC-07 | `model::default` (session) → removes `model` key, other keys preserved | ✅ `t05_set_model_session_default_removes_key` |
| FT-08 | AC-08 | `model::bad` (session) → exit 1, stderr lists opus/sonnet/haiku/default | ✅ `t06_set_model_session_invalid_exits_1` |
| FT-09 | AC-09 | `scope::subprocess model::claude-opus-4-8` → writes to config.toml user tier | ✅ `t07_set_model_subprocess_writes_config_toml` |
| FT-10 | AC-10 | `scope::subprocess model::` (empty) → exit 1, non-empty required | ✅ `t08_set_model_subprocess_empty_exits_1` |
| FT-11 | AC-11 | `effort_level::high` (session) → writes `effortLevel` | ✅ `t09_set_effort_session_writes_effort_level` |
| FT-12 | AC-12 | `effort_level::bad` (session) → exit 1, stderr lists low/normal/high/max | ✅ `t10_set_effort_session_invalid_exits_1` |
| FT-13 | AC-13 | `scope::subprocess effort_level::medium` → writes `effort` to config.toml | ✅ `t11_set_effort_subprocess_writes_config_toml` |
| FT-14 | AC-14 | `scope::subprocess effort_level::normal` → exit 1 (session-only value), stderr lists low/medium/high/max | ✅ `t12_set_effort_subprocess_session_only_value_exits_1` |
| FT-15 | AC-15 | `reset_model::1` (session) → removes `model` key | ✅ `t13_reset_model_session_removes_key` |
| FT-16 | AC-16 | `reset_effort_level::1` (session) → removes `effortLevel` key | ✅ `t14_reset_effort_session_removes_key` |
| FT-17 | AC-17 | `scope::subprocess reset_model::1` → idempotent, exit 0 whether or not key/file existed | ✅ `t15_reset_model_subprocess_idempotent` |
| FT-18 | AC-18 | `scope::subprocess reset_effort_level::1` → idempotent, exit 0 whether or not key/file existed | ✅ `t16_reset_effort_subprocess_idempotent` |
| FT-19 | AC-19 | `model::opus reset_model::1` (same scope) → exit 1, named conflict | ✅ `t17_mutual_exclusion_model_exits_1` |
| FT-20 | AC-20 | `effort_level::high reset_effort_level::1` (same scope) → exit 1, named conflict | ✅ `t18_mutual_exclusion_effort_exits_1` |
| FT-21 | AC-21 | `model::opus reset_effort_level::1` (mixed concepts, session) → both actions applied, exit 0 | ✅ `t19_combine_across_concepts` |
| FT-22 | AC-22 | `scope::subprocess model::claude-opus-4-8 effort_level::max` → both written, other keys preserved | ✅ `t20_combine_within_subprocess_scope_preserves_keys` |
| FT-23 | AC-23 | `format::json` (get, session) → `{"scope":...,"path":...,"model":...,"effort_level":...}` | ✅ `t21_json_format_shape` |
| FT-24 | AC-24 | `scope::subprocess model::VALUE`, `.clr/` absent → creates directory + `config.toml` | ✅ `t22_subprocess_creates_missing_dir_and_file` |
| FT-25 | AC-25 | Absolute path disclosure — every mode names the fully resolved path | ✅ cross-cutting, see Notes |
| FT-26 | AC-26 | `.model` listed once in `clp .help`; `.model.select` no longer a distinct entry | ✅ `dot04_all_visible_commands_present` / `dot05_exactly_fourteen_command_rows` / `dot13_model_select_hidden_from_listing` (`tests/cli/dot_test.rs`) |
| FT-27 | AC-27 | No inline file I/O — reuses `claude_profile_core::account::*`/`claude_core::toml_io::*` | ✅ architectural constraint, verified by code review |

### Notes

- All FT cases with a dedicated source fn are integration tests in `tests/cli/model_test.rs`, invoking the compiled `clp` binary against a temporary `HOME`.
- FT-04/05/06 share one source fn (`t04_set_model_session_each_shorthand`) — it loops all three shorthand values in one test, mirroring how AC-04/05/06 are three instances of the same mechanism.
- FT-25 (AC-25, absolute path disclosure) has no single dedicated test — it is a property every get-mode and write-mode assertion in `model_test.rs` already checks by asserting the exact resolved path string in each expected output. Listed here for AC traceability, not as a gap.
- FT-26 (AC-26, help-listing) is verified in `tests/cli/dot_test.rs`, not `model_test.rs` — the listing mechanism (`hidden_from_list`) is shared CLI-wide infrastructure, not `.model`-specific behavior.
- FT-27 (AC-27, no-duplication) is an architectural constraint verified by code review (`src/commands/model.rs` calls only `claude_profile_core::account::*` and `claude_core::toml_io::*` primitives, no inline `std::fs` read/write of settings.json or config.toml content) — no dedicated runtime test, matching this spec's prior treatment of the equivalent no-duplication criterion under the old design.
- `.model.select`'s own retirement-stub behavior (formerly AC-01 through AC-12 of superseded Feature 069) is covered separately in `tests/docs/cli/command/20_model_select.md` / `tests/cli/model_select_test.rs` — not duplicated here.
- FT-07/FT-15 and FT-11/FT-16 both target `settings.json`'s `model`/`effortLevel` keys via different action parameters (`model::default` vs `reset_model::1`; there is no `effort_level::default` shorthand) — each is exercised by its own distinct fixture/assertion pair, seeding a pre-existing unrelated key to confirm preservation.

---

### FT-01: Get, default scope is session

- **Given:** Fresh `HOME` — `~/.claude/settings.json` does not exist.
- **When:** `clp .model`
- **Then:** Stdout is `scope: session (<absolute settings.json path>)\nmodel: (unset)\neffort_level: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t01_get_default_scope_is_session`
- **Source:** [035_model_command.md AC-01](../../../docs/feature/035_model_command.md)

---

### FT-02: Get, `scope::subprocess`

- **Given:** Fresh `HOME` — `~/.clr/config.toml` does not exist.
- **When:** `clp .model scope::subprocess`
- **Then:** Stdout is `scope: subprocess (<absolute config.toml path>)\nmodel: (unset)\neffort_level: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t02_get_subprocess_scope`
- **Source:** [035_model_command.md AC-02](../../../docs/feature/035_model_command.md)

---

### FT-03: Invalid `scope::` value

- **Given:** Any environment.
- **When:** `clp .model scope::bad`
- **Then:** Exits 1. Stderr contains `session` and `subprocess`.
- **Exit:** 1
- **Source fn:** ✅ `t03_get_invalid_scope_exits_1`
- **Source:** [035_model_command.md AC-03](../../../docs/feature/035_model_command.md)

---

### FT-04: `model::opus` (session) writes `claude-opus-4-8`

- **Given:** Fresh `HOME`.
- **When:** `clp .model model::opus`
- **Then:** `~/.claude/settings.json` contains `"model":"claude-opus-4-8"`. Stdout confirms `model: opus  →  <path> (session)`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t04_set_model_session_each_shorthand`
- **Source:** [035_model_command.md AC-04](../../../docs/feature/035_model_command.md)

---

### FT-05: `model::sonnet` (session) writes `claude-sonnet-5`

- **Given:** Fresh `HOME`.
- **When:** `clp .model model::sonnet`
- **Then:** `~/.claude/settings.json` contains `"model":"claude-sonnet-5"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t04_set_model_session_each_shorthand`
- **Source:** [035_model_command.md AC-05](../../../docs/feature/035_model_command.md)

---

### FT-06: `model::haiku` (session) writes `claude-haiku-4-5-20251001`

- **Given:** Fresh `HOME`.
- **When:** `clp .model model::haiku`
- **Then:** `~/.claude/settings.json` contains `"model":"claude-haiku-4-5-20251001"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t04_set_model_session_each_shorthand`
- **Source:** [035_model_command.md AC-06](../../../docs/feature/035_model_command.md)

---

### FT-07: `model::default` (session) removes the key; other keys preserved

- **Given:** `~/.claude/settings.json` contains `{"model":"claude-sonnet-5","theme":"dark"}`.
- **When:** `clp .model model::default`
- **Then:** `"model"` key absent from `settings.json`; `"theme":"dark"` preserved. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t05_set_model_session_default_removes_key`
- **Source:** [035_model_command.md AC-07](../../../docs/feature/035_model_command.md)

---

### FT-08: `model::bad` (session) exits 1 with shorthand list

- **Given:** Any environment.
- **When:** `clp .model model::bad`
- **Then:** Exits 1. Stderr contains each of `opus`, `sonnet`, `haiku`, `default`.
- **Exit:** 1
- **Source fn:** ✅ `t06_set_model_session_invalid_exits_1`
- **Source:** [035_model_command.md AC-08](../../../docs/feature/035_model_command.md)

---

### FT-09: `scope::subprocess model::claude-opus-4-8` writes to `config.toml`

- **Given:** Fresh `HOME`.
- **When:** `clp .model scope::subprocess model::claude-opus-4-8`
- **Then:** `~/.clr/config.toml`'s user tier contains `model = "claude-opus-4-8"`. Stdout confirms `model: claude-opus-4-8  →  <path> (subprocess)`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t07_set_model_subprocess_writes_config_toml`
- **Source:** [035_model_command.md AC-09](../../../docs/feature/035_model_command.md)

---

### FT-10: `scope::subprocess model::` (empty) exits 1

- **Given:** Fresh `HOME`.
- **When:** `clp .model scope::subprocess model::`
- **Then:** Exits 1. Stderr requires a non-empty model ID. No `config.toml` written.
- **Exit:** 1
- **Source fn:** ✅ `t08_set_model_subprocess_empty_exits_1`
- **Source:** [035_model_command.md AC-10](../../../docs/feature/035_model_command.md)

---

### FT-11: `effort_level::high` (session) writes `effortLevel`

- **Given:** Fresh `HOME`.
- **When:** `clp .model effort_level::high`
- **Then:** `~/.claude/settings.json` contains `"effortLevel":"high"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t09_set_effort_session_writes_effort_level`
- **Source:** [035_model_command.md AC-11](../../../docs/feature/035_model_command.md)

---

### FT-12: `effort_level::bad` (session) exits 1 with vocabulary list

- **Given:** Any environment.
- **When:** `clp .model effort_level::bad`
- **Then:** Exits 1. Stderr contains each of `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** ✅ `t10_set_effort_session_invalid_exits_1`
- **Source:** [035_model_command.md AC-12](../../../docs/feature/035_model_command.md)

---

### FT-13: `scope::subprocess effort_level::medium` writes `effort`

- **Given:** Fresh `HOME`.
- **When:** `clp .model scope::subprocess effort_level::medium`
- **Then:** `~/.clr/config.toml`'s user tier contains `effort = "medium"`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t11_set_effort_subprocess_writes_config_toml`
- **Source:** [035_model_command.md AC-13](../../../docs/feature/035_model_command.md)

---

### FT-14: `scope::subprocess effort_level::normal` exits 1 (session-only value)

- **Given:** Any environment.
- **When:** `clp .model scope::subprocess effort_level::normal`
- **Then:** Exits 1 — `normal` is not in the subprocess vocabulary. Stderr contains each of `low`, `medium`, `high`, `max`.
- **Exit:** 1
- **Source fn:** ✅ `t12_set_effort_subprocess_session_only_value_exits_1`
- **Source:** [035_model_command.md AC-14](../../../docs/feature/035_model_command.md)

---

### FT-15: `reset_model::1` (session) removes the key

- **Given:** `~/.claude/settings.json` contains `{"model":"claude-opus-4-8"}`.
- **When:** `clp .model reset_model::1`
- **Then:** `"model"` key absent. Stdout confirms `model: (reset)  →  <path> (session)`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t13_reset_model_session_removes_key`
- **Source:** [035_model_command.md AC-15](../../../docs/feature/035_model_command.md)

---

### FT-16: `reset_effort_level::1` (session) removes the key

- **Given:** `~/.claude/settings.json` contains `{"effortLevel":"high"}`.
- **When:** `clp .model reset_effort_level::1`
- **Then:** `"effortLevel"` key absent — exercises `remove_session_effort()` (task 464). Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t14_reset_effort_session_removes_key`
- **Source:** [035_model_command.md AC-16](../../../docs/feature/035_model_command.md)

---

### FT-17: `scope::subprocess reset_model::1` is idempotent

- **Given:** Fresh `HOME` — no `.clr/` directory at all.
- **When:** `clp .model scope::subprocess reset_model::1`, run twice in sequence.
- **Then:** Both invocations exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t15_reset_model_subprocess_idempotent`
- **Source:** [035_model_command.md AC-17](../../../docs/feature/035_model_command.md)

---

### FT-18: `scope::subprocess reset_effort_level::1` is idempotent

- **Given:** Fresh `HOME` — no `.clr/` directory at all.
- **When:** `clp .model scope::subprocess reset_effort_level::1`, run twice in sequence.
- **Then:** Both invocations exit 0.
- **Exit:** 0
- **Source fn:** ✅ `t16_reset_effort_subprocess_idempotent`
- **Source:** [035_model_command.md AC-18](../../../docs/feature/035_model_command.md)

---

### FT-19: `model::` + `reset_model::1` (same concept) exits 1

- **Given:** Any environment.
- **When:** `clp .model model::opus reset_model::1`
- **Then:** Exits 1. Stderr states `model:: and reset_model::1 are mutually exclusive`.
- **Exit:** 1
- **Source fn:** ✅ `t17_mutual_exclusion_model_exits_1`
- **Source:** [035_model_command.md AC-19](../../../docs/feature/035_model_command.md)

---

### FT-20: `effort_level::` + `reset_effort_level::1` (same concept) exits 1

- **Given:** Any environment.
- **When:** `clp .model effort_level::high reset_effort_level::1`
- **Then:** Exits 1. Stderr states `effort_level:: and reset_effort_level::1 are mutually exclusive`.
- **Exit:** 1
- **Source fn:** ✅ `t18_mutual_exclusion_effort_exits_1`
- **Source:** [035_model_command.md AC-20](../../../docs/feature/035_model_command.md)

---

### FT-21: Combine across concepts (`model::` + `reset_effort_level::1`)

- **Given:** `~/.claude/settings.json` contains `{"effortLevel":"max"}`.
- **When:** `clp .model model::opus reset_effort_level::1`
- **Then:** Both actions apply in one call: `"model":"claude-opus-4-8"` written, `"effortLevel"` removed. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t19_combine_across_concepts`
- **Source:** [035_model_command.md AC-21](../../../docs/feature/035_model_command.md)

---

### FT-22: Combine within one subprocess call, other keys preserved

- **Given:** `~/.clr/config.toml` already contains `provider = "kimi"` (written by `.provider.select`).
- **When:** `clp .model scope::subprocess model::claude-opus-4-8 effort_level::max`
- **Then:** Both `model` and `effort` written in the same call; `provider = "kimi"` preserved. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t20_combine_within_subprocess_scope_preserves_keys`
- **Source:** [035_model_command.md AC-22](../../../docs/feature/035_model_command.md)

---

### FT-23: `format::json` matches the documented shape

- **Given:** Fresh `HOME`.
- **When:** `clp .model format::json`
- **Then:** Stdout parses as `{"scope":"session","path":"<path>","model":null,"effort_level":null}`. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t21_json_format_shape`
- **Source:** [035_model_command.md AC-23](../../../docs/feature/035_model_command.md)

---

### FT-24: Subprocess write creates missing directory and file

- **Given:** Fresh `HOME` — `~/.clr/` does not exist.
- **When:** `clp .model scope::subprocess model::claude-haiku-4-5-20251001`
- **Then:** `~/.clr/` directory and `config.toml` are both created. Exits 0.
- **Exit:** 0
- **Source fn:** ✅ `t22_subprocess_creates_missing_dir_and_file`
- **Source:** [035_model_command.md AC-24](../../../docs/feature/035_model_command.md)

---

### FT-25: Absolute path disclosure (cross-cutting)

- **Given:** Any scope, any mode.
- **When:** Any `.model` invocation.
- **Then:** Output names the fully resolved absolute path — never `~`-relative, bare filename, or omitted. Verified as an assertion embedded in every FT-01/02/04/07/09/11/13/15 expected-output check above, not by one isolated test.
- **Exit:** n/a
- **Source fn:** ✅ cross-cutting (see Notes)
- **Source:** [035_model_command.md AC-25](../../../docs/feature/035_model_command.md)

---

### FT-26: `.model` listed once; `.model.select` no longer distinct

- **Given:** Any environment.
- **When:** `clp .help` (or `clp .`)
- **Then:** `.model` appears exactly once; `.model.select` does not appear as a listed row (still dispatchable — see `20_model_select.md`).
- **Exit:** 0
- **Source fn:** ✅ `dot04_all_visible_commands_present` / `dot05_exactly_fourteen_command_rows` / `dot13_model_select_hidden_from_listing` (`tests/cli/dot_test.rs`)
- **Source:** [035_model_command.md AC-26](../../../docs/feature/035_model_command.md)

---

### FT-27: No inline I/O duplication (architectural)

- **Given:** `src/commands/model.rs` source.
- **When:** Code review.
- **Then:** All reads/writes go through `claude_profile_core::account::*` (session) or `claude_core::toml_io::*` (subprocess) — zero inline `std::fs` parsing/serialization of settings.json or config.toml content.
- **Exit:** n/a
- **Source fn:** ✅ architectural constraint, verified by code review
- **Source:** [035_model_command.md AC-27](../../../docs/feature/035_model_command.md)
