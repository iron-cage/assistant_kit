# Command Tests :: `.models`

### Scope

- **Purpose**: Integration test cases for the `.models` model discovery command.
- **Source**: `docs/cli/command/008_models.md`, `docs/feature/068_models_list_command.md`
- **Covers**: AC-01 through AC-10

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-01 | `offline::1` — stdout contains `claude-opus-4-8` | `it01_offline_contains_opus` |
| IT-02 | AC-02 | `offline::1` — stdout contains `claude-sonnet-5` | `it02_offline_contains_sonnet` |
| IT-03 | AC-03 | `offline::1` — stdout contains `claude-haiku-4-5-20251001` | `it03_offline_contains_haiku` |
| IT-04 | AC-04 | `offline::1 format::table` — output has `ID` header | `it04_offline_table_has_header` |
| IT-05 | AC-05 | `offline::1 format::text` — one ID per line, no table markers | `it05_offline_text_one_per_line` |
| IT-06 | AC-06 | `offline::1 format::json` — output is valid JSON array | `it06_offline_json_valid_array` |
| IT-07 | AC-07 | `offline::1 name::opus` — only models with `opus` in ID | `it07_name_filter_opus_only` |
| IT-08 | AC-08 | `offline::1 name::zz_no_match` — empty output, exits 0 | `it08_name_filter_no_match` |
| IT-09 | AC-09 | `.models` appears in `clp .help` output | `it09_models_in_help_output` |
| IT-10 | AC-10 | `offline::1 name::claude-opus` — substring match, all matching returned | `it10_name_filter_substring` |

### Notes

- All IT cases use `offline::1` to avoid network dependency in CI.
- IT-05 (format::text): verify stdout lines match `/^claude-[a-z0-9-]+$/`; no `|` or padding characters.
- IT-06 (format::json): parse stdout as JSON; verify it is an array; each element has an `id` field.
- IT-07 (name::opus): all returned IDs must contain `"opus"`; models without `opus` must be absent.
- IT-08 (no match): exit code must be 0 even with zero rows returned.
- IT-09: requires `clp .help` (no env setup needed).

---

### IT-01: Offline mode contains `claude-opus-4-8`

- **Given:** Any environment (no network needed).
- **When:** `clp .models offline::1`
- **Then:** Stdout contains `claude-opus-4-8`. Exits 0.
- **Exit:** 0
- **Source fn:** `it01_offline_contains_opus`
- **Source:** [068_models_list_command.md AC-01](../../../../docs/feature/068_models_list_command.md)

---

### IT-02: Offline mode contains `claude-sonnet-5`

- **Given:** Any environment.
- **When:** `clp .models offline::1`
- **Then:** Stdout contains `claude-sonnet-5`. Exits 0.
- **Exit:** 0
- **Source fn:** `it02_offline_contains_sonnet`
- **Source:** [068_models_list_command.md AC-02](../../../../docs/feature/068_models_list_command.md)

---

### IT-03: Offline mode contains `claude-haiku-4-5-20251001`

- **Given:** Any environment.
- **When:** `clp .models offline::1`
- **Then:** Stdout contains `claude-haiku-4-5-20251001`. Exits 0.
- **Exit:** 0
- **Source fn:** `it03_offline_contains_haiku`
- **Source:** [068_models_list_command.md AC-03](../../../../docs/feature/068_models_list_command.md)

---

### IT-04: Table format has `ID` header

- **Given:** Any environment.
- **When:** `clp .models offline::1 format::table`
- **Then:** First stdout line contains `ID`. Exits 0.
- **Exit:** 0
- **Source fn:** `it04_offline_table_has_header`
- **Source:** [068_models_list_command.md AC-04](../../../../docs/feature/068_models_list_command.md)

---

### IT-05: Text format outputs one ID per line

- **Given:** Any environment.
- **When:** `clp .models offline::1 format::text`
- **Then:** Stdout is a sequence of newline-separated model IDs; no `|` table separators present. Each line matches `^claude-[a-z0-9.-]+$`. Exits 0.
- **Exit:** 0
- **Source fn:** `it05_offline_text_one_per_line`
- **Source:** [068_models_list_command.md AC-05](../../../../docs/feature/068_models_list_command.md)

---

### IT-06: JSON format outputs valid JSON array

- **Given:** Any environment.
- **When:** `clp .models offline::1 format::json`
- **Then:** Stdout is parseable as a JSON array (`[{...}, ...]`); each element has an `"id"` field. Exits 0.
- **Exit:** 0
- **Source fn:** `it06_offline_json_valid_array`
- **Source:** [068_models_list_command.md AC-06](../../../../docs/feature/068_models_list_command.md)

---

### IT-07: `name::opus` returns only models with `opus` in ID

- **Given:** Any environment.
- **When:** `clp .models offline::1 name::opus`
- **Then:** All returned model IDs contain `"opus"`. Models without `"opus"` in the ID are absent from output. Exits 0.
- **Exit:** 0
- **Source fn:** `it07_name_filter_opus_only`
- **Source:** [068_models_list_command.md AC-07](../../../../docs/feature/068_models_list_command.md)

---

### IT-08: `name::zz_no_match` returns empty output with exit 0

- **Given:** Any environment.
- **When:** `clp .models offline::1 name::zz_no_match`
- **Then:** Stdout contains no model IDs. Exits 0.
- **Exit:** 0
- **Source fn:** `it08_name_filter_no_match`
- **Source:** [068_models_list_command.md AC-08](../../../../docs/feature/068_models_list_command.md)

---

### IT-09: `.models` appears in `clp .help` output

- **Given:** Any environment.
- **When:** `clp .help`
- **Then:** Output contains `.models`. Exits 0.
- **Exit:** 0
- **Source fn:** `it09_models_in_help_output`
- **Source:** [068_models_list_command.md AC-09](../../../../docs/feature/068_models_list_command.md)

---

### IT-10: `name::claude-opus` is a substring match

- **Given:** Any environment.
- **When:** `clp .models offline::1 name::claude-opus`
- **Then:** All returned model IDs contain `"claude-opus"` (case-insensitive substring). Haiku and Sonnet models are absent. Exits 0.
- **Exit:** 0
- **Source fn:** `it10_name_filter_substring`
- **Source:** [068_models_list_command.md AC-10](../../../../docs/feature/068_models_list_command.md)
