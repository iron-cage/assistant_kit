# FT — Feature 069: Model Select Command

### Scope

- **Purpose**: Test cases for the `.model.select` subprocess model preference command.
- **Source**: `docs/feature/069_model_select_command.md`
- **Covers**: AC-01 through AC-12

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | No `~/.clr/prefs.json` → `model.select: (unset)` | `ft01_get_unset_no_file` |
| FT-02 | AC-02 | `prefs.json` has `subprocess_model` → prints value | `ft02_get_shows_pinned_value` |
| FT-03 | AC-03 | `id::claude-opus-4-8` → file written; stdout `(pinned)` | `ft03_set_opus_pins_model` |
| FT-04 | AC-04 | `id::claude-sonnet-5` → file written | `ft04_set_sonnet_pins_model` |
| FT-05 | AC-05 | `reset::1` with preference set → key removed; others preserved | `ft05_reset_removes_key_preserves_others` |
| FT-06 | AC-06 | `reset::1` with no `prefs.json` → exits 0 idempotently | `ft06_reset_no_file_is_idempotent` |
| FT-07 | AC-07 | `id::VALUE` creates `prefs.json` when absent | `ft07_set_creates_file_when_absent` |
| FT-08 | AC-08 | `id::VALUE` on existing `prefs.json` → other keys preserved | `ft08_set_preserves_other_keys` |
| FT-09 | AC-09 | `id::VALUE reset::1` → exits 1 with `mutually exclusive` in stderr | `ft09_id_and_reset_mutual_exclusive` |
| FT-10 | AC-10 | `format::json` with preference set → JSON output | `ft10_get_json_format` |
| FT-11 | AC-11 | `.model.select` appears in `clp .help` | `ft11_model_select_in_help` |
| FT-12 | AC-12 | `id::` (empty) → exits 1 with non-empty required in stderr | `ft12_empty_id_exits_1` |

### Notes

- All FT cases are integration tests in `tests/cli/model_select_test.rs`.
- All FT cases use a temporary isolated `~/.clr/` directory to avoid touching the real user environment.
- FT-05: seed `prefs.json` with `{"subprocess_model":"claude-opus-4-8","other_key":"val"}` before calling `reset::1`; verify `other_key` is preserved and `subprocess_model` is absent.
- FT-08: seed `prefs.json` with `{"other_key":"val"}`; call `id::claude-opus-4-8`; verify both keys present.
- FT-09: does not require file existence — parameter validation fires first.
- FT-11: requires `clp .help` only (no env setup needed).

---

### FT-01: Get with no `prefs.json` returns `(unset)`

- **Given:** `~/.clr/prefs.json` does not exist.
- **When:** `clp .model.select`
- **Then:** Stdout is `model.select: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft01_get_unset_no_file`
- **Source:** [069_model_select_command.md AC-01](../../../docs/feature/069_model_select_command.md)

---

### FT-02: Get returns pinned model value

- **Given:** `~/.clr/prefs.json` contains `{"subprocess_model":"claude-opus-4-8"}`.
- **When:** `clp .model.select`
- **Then:** Stdout is `model.select: claude-opus-4-8\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft02_get_shows_pinned_value`
- **Source:** [069_model_select_command.md AC-02](../../../docs/feature/069_model_select_command.md)

---

### FT-03: `id::claude-opus-4-8` writes to `prefs.json`

- **Given:** Any state.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/prefs.json` contains `"subprocess_model":"claude-opus-4-8"`. Stdout contains `(pinned)`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft03_set_opus_pins_model`
- **Source:** [069_model_select_command.md AC-03](../../../docs/feature/069_model_select_command.md)

---

### FT-04: `id::claude-sonnet-5` writes to `prefs.json`

- **Given:** Any state.
- **When:** `clp .model.select id::claude-sonnet-5`
- **Then:** `~/.clr/prefs.json` contains `"subprocess_model":"claude-sonnet-5"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft04_set_sonnet_pins_model`
- **Source:** [069_model_select_command.md AC-04](../../../docs/feature/069_model_select_command.md)

---

### FT-05: `reset::1` removes key and preserves others

- **Given:** `~/.clr/prefs.json` contains `{"subprocess_model":"claude-opus-4-8","other_key":"val"}`.
- **When:** `clp .model.select reset::1`
- **Then:** `~/.clr/prefs.json` no longer contains `"subprocess_model"` key. `"other_key":"val"` is preserved. Stdout is `model.select: (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft05_reset_removes_key_preserves_others`
- **Source:** [069_model_select_command.md AC-05](../../../docs/feature/069_model_select_command.md)

---

### FT-06: `reset::1` with no `prefs.json` is idempotent

- **Given:** `~/.clr/prefs.json` does not exist.
- **When:** `clp .model.select reset::1`
- **Then:** Stdout is `model.select: (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft06_reset_no_file_is_idempotent`
- **Source:** [069_model_select_command.md AC-06](../../../docs/feature/069_model_select_command.md)

---

### FT-07: `id::VALUE` creates `prefs.json` when absent

- **Given:** `~/.clr/prefs.json` does not exist.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/prefs.json` is created; contains `"subprocess_model":"claude-opus-4-8"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft07_set_creates_file_when_absent`
- **Source:** [069_model_select_command.md AC-07](../../../docs/feature/069_model_select_command.md)

---

### FT-08: `id::VALUE` preserves pre-existing keys

- **Given:** `~/.clr/prefs.json` contains `{"other_key":"val"}`.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/prefs.json` contains both `"subprocess_model":"claude-opus-4-8"` and `"other_key":"val"`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft08_set_preserves_other_keys`
- **Source:** [069_model_select_command.md AC-08](../../../docs/feature/069_model_select_command.md)

---

### FT-09: `id::VALUE reset::1` exits 1 with mutual exclusion message

- **Given:** Any environment.
- **When:** `clp .model.select id::claude-opus-4-8 reset::1`
- **Then:** Exits 1. Stderr contains `mutually exclusive`.
- **Exit:** 1
- **Source fn:** `ft09_id_and_reset_mutual_exclusive`
- **Source:** [069_model_select_command.md AC-09](../../../docs/feature/069_model_select_command.md)

---

### FT-10: `format::json` returns JSON output

- **Given:** `~/.clr/prefs.json` contains `{"subprocess_model":"claude-opus-4-8"}`.
- **When:** `clp .model.select format::json`
- **Then:** Stdout is `{"subprocess_model":"claude-opus-4-8"}` (or valid JSON equivalent). Exits 0.
- **Variant:** When preference absent → `{"subprocess_model":null}`.
- **Exit:** 0
- **Source fn:** `ft10_get_json_format`
- **Source:** [069_model_select_command.md AC-10](../../../docs/feature/069_model_select_command.md)

---

### FT-11: `.model.select` appears in `clp .help`

- **Given:** Any environment.
- **When:** `clp .help`
- **Then:** Output contains `.model.select`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft11_model_select_in_help`
- **Source:** [069_model_select_command.md AC-11](../../../docs/feature/069_model_select_command.md)

---

### FT-12: Empty `id::` exits 1

- **Given:** Any environment.
- **When:** `clp .model.select id::`
- **Then:** Exits 1. Stderr indicates `id::` must be non-empty.
- **Exit:** 1
- **Source fn:** `ft12_empty_id_exits_1`
- **Source:** [069_model_select_command.md AC-12](../../../docs/feature/069_model_select_command.md)
