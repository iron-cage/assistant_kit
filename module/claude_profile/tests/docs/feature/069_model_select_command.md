# FT — Feature 069: Model Select Command

> **Superseded** (Feature 035/Task 465): `.model.select` is retired — FT-01 through FT-12 below describe its original live get/set/reset design, before Feature 035 merged its command name, `id::`/`reset::` parameters, and dispatch function into `.model` via the new `scope::` parameter (see [docs/feature/035_model_command.md](../../../docs/feature/035_model_command.md)). `.model.select` now returns a migration-error stub unconditionally, regardless of any parameters given. Current coverage of the retirement-stub behavior lives in [tests/docs/cli/command/20_model_select.md](../cli/command/20_model_select.md) (IT-01 through IT-03), backed by `model_select_test.rs`'s `t23_*` functions. This file's own Source doc, [docs/feature/069_model_select_command.md](../../../docs/feature/069_model_select_command.md), carries the identical "Superseded" note. Cases below are retained for historical reference only — their cited function names no longer exist in the test suite.

### Scope

- **Purpose**: Test cases for the `.model.select` subprocess model preference command.
- **Source**: `docs/feature/069_model_select_command.md`
- **Covers**: AC-01 through AC-12

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | No `~/.clr/config.toml` → `model.select: (unset)` | `ft01_get_unset_no_file` (N/A) |
| FT-02 | AC-02 | `config.toml` has `model` → prints value | `ft02_get_shows_pinned_value` (N/A) |
| FT-03 | AC-03 | `id::claude-opus-4-8` → file written; stdout `(pinned)` | `ft03_set_opus_pins_model` (N/A) |
| FT-04 | AC-04 | `id::claude-sonnet-5` → file written | `ft04_set_sonnet_pins_model` (N/A) |
| FT-05 | AC-05 | `reset::1` with preference set → key removed; others preserved | `ft05_reset_removes_key_preserves_others` (N/A) |
| FT-06 | AC-06 | `reset::1` with no `config.toml` → exits 0 idempotently | `ft06_reset_no_file_is_idempotent` (N/A) |
| FT-07 | AC-07 | `id::VALUE` creates `config.toml` when absent | `ft07_set_creates_file_when_absent` (N/A) |
| FT-08 | AC-08 | `id::VALUE` on existing `config.toml` → other keys preserved | `ft08_set_preserves_other_keys` (N/A) |
| FT-09 | AC-09 | `id::VALUE reset::1` → exits 1 with `mutually exclusive` in stderr | `ft09_id_and_reset_mutual_exclusive` (N/A) |
| FT-10 | AC-10 | `format::json` with preference set → JSON output | `ft10_get_json_format` (N/A) |
| FT-11 | AC-11 | `.model.select` appears in `clp .help` — **FALSE under current behavior**, see FT-11 note below | `ft11_model_select_in_help` (N/A) |
| FT-12 | AC-12 | `id::` (empty) → exits 1 with non-empty required in stderr | `ft12_empty_id_exits_1` (N/A) |

### Notes

- All FT cases are integration tests in `tests/cli/model_select_test.rs`.
- All FT cases use a temporary isolated `~/.clr/` directory to avoid touching the real user environment.
- Backing store is `~/.clr/config.toml`'s `model` key (task 410 migrated this command off `claude_core::settings_io`/`~/.clr/prefs.json`). The `format::json` output shape is unchanged — still keyed `subprocess_model`, this command's own CLI-visible JSON contract, independent of the `model` backing-store key name.
- FT-05: seed `config.toml` with `model = "claude-opus-4-8"` and `other_key = "val"` before calling `reset::1`; verify `other_key` is preserved and `model` is absent.
- FT-08: seed `config.toml` with `other_key = "val"`; call `id::claude-opus-4-8`; verify both keys present.
- FT-09: does not require file existence — parameter validation fires first.
- FT-11: requires `clp .help` only (no env setup needed).

---

### FT-01: Get with no `config.toml` returns `(unset)`

> Historical only — describes `.model.select`'s retired live get behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .model.select`
- **Then:** Stdout is `model.select: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft01_get_unset_no_file` (N/A)
- **Source:** [069_model_select_command.md AC-01](../../../docs/feature/069_model_select_command.md)

---

### FT-02: Get returns pinned model value

> Historical only — describes `.model.select`'s retired live get behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`.
- **When:** `clp .model.select`
- **Then:** Stdout is `model.select: claude-opus-4-8\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft02_get_shows_pinned_value` (N/A)
- **Source:** [069_model_select_command.md AC-02](../../../docs/feature/069_model_select_command.md)

---

### FT-03: `id::claude-opus-4-8` writes to `config.toml`

> Historical only — describes `.model.select`'s retired live set behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** Any state.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`. Stdout contains `(pinned)`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft03_set_opus_pins_model` (N/A)
- **Source:** [069_model_select_command.md AC-03](../../../docs/feature/069_model_select_command.md)

---

### FT-04: `id::claude-sonnet-5` writes to `config.toml`

> Historical only — describes `.model.select`'s retired live set behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** Any state.
- **When:** `clp .model.select id::claude-sonnet-5`
- **Then:** `~/.clr/config.toml` contains `model = "claude-sonnet-5"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft04_set_sonnet_pins_model` (N/A)
- **Source:** [069_model_select_command.md AC-04](../../../docs/feature/069_model_select_command.md)

---

### FT-05: `reset::1` removes key and preserves others

> Historical only — describes `.model.select`'s retired live reset behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"` and `other_key = "val"`.
- **When:** `clp .model.select reset::1`
- **Then:** `~/.clr/config.toml` no longer contains the `model` key. `other_key = "val"` is preserved. Stdout is `model.select: (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft05_reset_removes_key_preserves_others` (N/A)
- **Source:** [069_model_select_command.md AC-05](../../../docs/feature/069_model_select_command.md)

---

### FT-06: `reset::1` with no `config.toml` is idempotent

> Historical only — describes `.model.select`'s retired live reset behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .model.select reset::1`
- **Then:** Stdout is `model.select: (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft06_reset_no_file_is_idempotent` (N/A)
- **Source:** [069_model_select_command.md AC-06](../../../docs/feature/069_model_select_command.md)

---

### FT-07: `id::VALUE` creates `config.toml` when absent

> Historical only — describes `.model.select`'s retired live set behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/config.toml` is created; contains `model = "claude-opus-4-8"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft07_set_creates_file_when_absent` (N/A)
- **Source:** [069_model_select_command.md AC-07](../../../docs/feature/069_model_select_command.md)

---

### FT-08: `id::VALUE` preserves pre-existing keys

> Historical only — describes `.model.select`'s retired live set behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** `~/.clr/config.toml` contains `other_key = "val"`.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/config.toml` contains both `model = "claude-opus-4-8"` and `other_key = "val"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft08_set_preserves_other_keys` (N/A)
- **Source:** [069_model_select_command.md AC-08](../../../docs/feature/069_model_select_command.md)

---

### FT-09: `id::VALUE reset::1` exits 1 with mutual exclusion message

> Historical only — describes `.model.select`'s retired mutual-exclusion validation. The `id::`/`reset::1` mutual-exclusion check itself is still live, just relocated — see `.model.select`'s own T23 stub coverage (`20_model_select.md`) and `.provider.select`'s equivalent (`21_provider_select.md` IT-09).

- **Given:** Any environment.
- **When:** `clp .model.select id::claude-opus-4-8 reset::1`
- **Then:** Exits 1. Stderr contains `mutually exclusive`.
- **Exit:** 1
- **Source fn:** `ft09_id_and_reset_mutual_exclusive` (N/A)
- **Source:** [069_model_select_command.md AC-09](../../../docs/feature/069_model_select_command.md)

---

### FT-10: `format::json` returns JSON output

> Historical only — describes `.model.select`'s retired live get behavior. See the Superseded note at the top of this file for current coverage.

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`.
- **When:** `clp .model.select format::json`
- **Then:** Stdout is `{"subprocess_model":"claude-opus-4-8"}` (or valid JSON equivalent). Exits 0.
- **Variant:** When preference absent → `{"subprocess_model":null}`.
- **Exit:** 0
- **Source fn:** `ft10_get_json_format` (N/A)
- **Source:** [069_model_select_command.md AC-10](../../../docs/feature/069_model_select_command.md)

---

### FT-11: `.model.select` appears in `clp .help`

> **Contradicted by current behavior**, not merely historical — Feature 035 AC-26 made `.model.select` `hidden_from_list(true)`; `clp .help` no longer lists it (covered instead by `dot_test.rs`'s `dot13`, per `model_select_test.rs`'s own module doc comment). Re-running this exact assertion today would fail.

- **Given:** Any environment.
- **When:** `clp .help`
- **Then:** Output contains `.model.select`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft11_model_select_in_help` (N/A)
- **Source:** [069_model_select_command.md AC-11](../../../docs/feature/069_model_select_command.md)

---

### FT-12: Empty `id::` exits 1

> Historical only — describes `.model.select`'s retired live set-mode validation. See the Superseded note at the top of this file for current coverage.

- **Given:** Any environment.
- **When:** `clp .model.select id::`
- **Then:** Exits 1. Stderr indicates `id::` must be non-empty.
- **Exit:** 1
- **Source fn:** `ft12_empty_id_exits_1` (N/A)
- **Source:** [069_model_select_command.md AC-12](../../../docs/feature/069_model_select_command.md)
