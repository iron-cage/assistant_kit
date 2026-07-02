# Command Tests :: `.model.select`

### Scope

- **Purpose**: Integration test cases for the `.model.select` subprocess model preference command.
- **Source**: `docs/cli/command/007_model.md`, `docs/feature/069_model_select_command.md`
- **Covers**: AC-01 through AC-12

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-01 | No `~/.clr/prefs.json` → `model.select: (unset)` | `it01_get_unset_no_file` |
| IT-02 | AC-02 | `prefs.json` has `subprocess_model` → prints value | `it02_get_shows_pinned_value` |
| IT-03 | AC-03 | `id::claude-opus-4-8` → file written; stdout `(pinned)` | `it03_set_opus_pins_model` |
| IT-04 | AC-04 | `id::claude-sonnet-5` → file written | `it04_set_sonnet_pins_model` |
| IT-05 | AC-05 | `reset::1` with preference set → key removed; others preserved | `it05_reset_removes_key_preserves_others` |
| IT-06 | AC-06 | `reset::1` with no `prefs.json` → exits 0 idempotently | `it06_reset_no_file_is_idempotent` |
| IT-07 | AC-07 | `id::VALUE` creates `prefs.json` when absent | `it07_set_creates_file_when_absent` |
| IT-08 | AC-08 | `id::VALUE` on existing `prefs.json` → other keys preserved | `it08_set_preserves_other_keys` |
| IT-09 | AC-09 | `id::VALUE reset::1` → exits 1 with `mutually exclusive` in stderr | `it09_id_and_reset_mutual_exclusive` |
| IT-10 | AC-10 | `format::json` with preference set → JSON output | `it10_get_json_format` |
| IT-11 | AC-11 | `.model.select` appears in `clp .help` | `it11_model_select_in_help` |
| IT-12 | AC-12 | `id::` (empty) → exits 1 with non-empty required in stderr | `it12_empty_id_exits_1` |

### Notes

- All IT cases use a temporary isolated `~/.clr/` directory to avoid touching the real user environment.
- IT-05: seed `prefs.json` with `{"subprocess_model":"claude-opus-4-8","other_key":"val"}` before calling `reset::1`; verify `other_key` is preserved and `subprocess_model` is absent.
- IT-08: seed `prefs.json` with `{"other_key":"val"}`; call `id::claude-opus-4-8`; verify both keys present.
- IT-09: does not require file existence — parameter validation fires first.
- IT-11: requires `clp .help` only (no env setup needed).

---

### IT-01: Get with no `prefs.json` returns `(unset)`

- **Given:** `~/.clr/prefs.json` does not exist.
- **When:** `clp .model.select`
- **Then:** Stdout is `model.select: (unset)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `it01_get_unset_no_file`
- **Source:** [069_model_select_command.md AC-01](../../../../docs/feature/069_model_select_command.md)

---

### IT-02: Get returns pinned model value

- **Given:** `~/.clr/prefs.json` contains `{"subprocess_model":"claude-opus-4-8"}`.
- **When:** `clp .model.select`
- **Then:** Stdout is `model.select: claude-opus-4-8\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `it02_get_shows_pinned_value`
- **Source:** [069_model_select_command.md AC-02](../../../../docs/feature/069_model_select_command.md)

---

### IT-03: `id::claude-opus-4-8` writes to `prefs.json`

- **Given:** Any state.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/prefs.json` contains `"subprocess_model":"claude-opus-4-8"`. Stdout contains `(pinned)`. Exits 0.
- **Exit:** 0
- **Source fn:** `it03_set_opus_pins_model`
- **Source:** [069_model_select_command.md AC-03](../../../../docs/feature/069_model_select_command.md)

---

### IT-04: `id::claude-sonnet-5` writes to `prefs.json`

- **Given:** Any state.
- **When:** `clp .model.select id::claude-sonnet-5`
- **Then:** `~/.clr/prefs.json` contains `"subprocess_model":"claude-sonnet-5"`. Exits 0.
- **Exit:** 0
- **Source fn:** `it04_set_sonnet_pins_model`
- **Source:** [069_model_select_command.md AC-04](../../../../docs/feature/069_model_select_command.md)

---

### IT-05: `reset::1` removes key and preserves others

- **Given:** `~/.clr/prefs.json` contains `{"subprocess_model":"claude-opus-4-8","other_key":"val"}`.
- **When:** `clp .model.select reset::1`
- **Then:** `~/.clr/prefs.json` no longer contains `"subprocess_model"` key. `"other_key":"val"` is preserved. Stdout is `model.select: (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `it05_reset_removes_key_preserves_others`
- **Source:** [069_model_select_command.md AC-05](../../../../docs/feature/069_model_select_command.md)

---

### IT-06: `reset::1` with no `prefs.json` is idempotent

- **Given:** `~/.clr/prefs.json` does not exist.
- **When:** `clp .model.select reset::1`
- **Then:** Stdout is `model.select: (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `it06_reset_no_file_is_idempotent`
- **Source:** [069_model_select_command.md AC-06](../../../../docs/feature/069_model_select_command.md)

---

### IT-07: `id::VALUE` creates `prefs.json` when absent

- **Given:** `~/.clr/prefs.json` does not exist.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/prefs.json` is created; contains `"subprocess_model":"claude-opus-4-8"`. Exits 0.
- **Exit:** 0
- **Source fn:** `it07_set_creates_file_when_absent`
- **Source:** [069_model_select_command.md AC-07](../../../../docs/feature/069_model_select_command.md)

---

### IT-08: `id::VALUE` preserves pre-existing keys

- **Given:** `~/.clr/prefs.json` contains `{"other_key":"val"}`.
- **When:** `clp .model.select id::claude-opus-4-8`
- **Then:** `~/.clr/prefs.json` contains both `"subprocess_model":"claude-opus-4-8"` and `"other_key":"val"`. Exits 0.
- **Exit:** 0
- **Source fn:** `it08_set_preserves_other_keys`
- **Source:** [069_model_select_command.md AC-08](../../../../docs/feature/069_model_select_command.md)

---

### IT-09: `id::VALUE reset::1` exits 1 with mutual exclusion message

- **Given:** Any environment.
- **When:** `clp .model.select id::claude-opus-4-8 reset::1`
- **Then:** Exits 1. Stderr contains `mutually exclusive`.
- **Exit:** 1
- **Source fn:** `it09_id_and_reset_mutual_exclusive`
- **Source:** [069_model_select_command.md AC-09](../../../../docs/feature/069_model_select_command.md)

---

### IT-10: `format::json` returns JSON output

- **Given:** `~/.clr/prefs.json` contains `{"subprocess_model":"claude-opus-4-8"}`.
- **When:** `clp .model.select format::json`
- **Then:** Stdout is `{"subprocess_model":"claude-opus-4-8"}` (or valid JSON equivalent). Exits 0.
- **Variant:** When preference absent → `{"subprocess_model":null}`.
- **Exit:** 0
- **Source fn:** `it10_get_json_format`
- **Source:** [069_model_select_command.md AC-10](../../../../docs/feature/069_model_select_command.md)

---

### IT-11: `.model.select` appears in `clp .help`

- **Given:** Any environment.
- **When:** `clp .help`
- **Then:** Output contains `.model.select`. Exits 0.
- **Exit:** 0
- **Source fn:** `it11_model_select_in_help`
- **Source:** [069_model_select_command.md AC-11](../../../../docs/feature/069_model_select_command.md)

---

### IT-12: Empty `id::` exits 1

- **Given:** Any environment.
- **When:** `clp .model.select id::`
- **Then:** Exits 1. Stderr indicates `id::` must be non-empty.
- **Exit:** 1
- **Source fn:** `it12_empty_id_exits_1`
- **Source:** [069_model_select_command.md AC-12](../../../../docs/feature/069_model_select_command.md)
