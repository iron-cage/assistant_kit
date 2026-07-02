# FT — Feature 068: Models List Command

### Scope

- **Purpose**: Test cases for the `.models` model discovery command.
- **Source**: `docs/feature/068_models_list_command.md`
- **Covers**: AC-01 through AC-10

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `offline::1` — stdout contains `claude-opus-4-8` | `ft01_offline_contains_opus` |
| FT-02 | AC-02 | `offline::1` — stdout contains `claude-sonnet-5` | `ft02_offline_contains_sonnet` |
| FT-03 | AC-03 | `offline::1` — stdout contains `claude-haiku-4-5-20251001` | `ft03_offline_contains_haiku` |
| FT-04 | AC-04 | `offline::1 format::table` — output has `ID` header | `ft04_offline_table_has_header` |
| FT-05 | AC-05 | `offline::1 format::text` — one ID per line, no table markers | `ft05_offline_text_one_per_line` |
| FT-06 | AC-06 | `offline::1 format::json` — output is valid JSON array with `id` fields | `ft06_offline_json_valid_array` |
| FT-07 | AC-07 | `offline::1 name::opus` — only models with `opus` in ID returned | `ft07_name_filter_opus_only` |
| FT-08 | AC-08 | `offline::1 name::zz_no_match` — empty output, exits 0 | `ft08_name_filter_no_match` |
| FT-09 | AC-09 | `.models` appears in `clp .help` | `ft09_models_in_help_output` |
| FT-10 | AC-10 | `offline::1 name::claude-opus` — substring match returns all matching | `ft10_name_filter_substring` |

### Notes

- All FT cases are integration tests in `tests/cli/models_test.rs`.
- All FT cases except FT-09 use `offline::1` to avoid network dependency in CI.
- FT-05 (format::text): verify stdout lines match `^claude-[a-z0-9.-]+$`; no `|` or padding characters.
- FT-06 (format::json): parse stdout as JSON; verify it is an array; each element has an `id` field.
- FT-07 (name::opus): all returned IDs must contain `"opus"`; models without `"opus"` must be absent.
- FT-08 (no match): exit code must be 0 even with zero rows returned.
- FT-09: requires `clp .help` only (no env setup needed).
- FT-10 (name::claude-opus): substring match; all results contain `"claude-opus"`; haiku and sonnet absent.

---

### FT-01: Offline mode contains `claude-opus-4-8`

- **Given:** Any environment (no network needed).
- **When:** `clp .models offline::1`
- **Then:** Stdout contains `claude-opus-4-8`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft01_offline_contains_opus`
- **Source:** [068_models_list_command.md AC-01](../../../docs/feature/068_models_list_command.md)

---

### FT-02: Offline mode contains `claude-sonnet-5`

- **Given:** Any environment.
- **When:** `clp .models offline::1`
- **Then:** Stdout contains `claude-sonnet-5`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft02_offline_contains_sonnet`
- **Source:** [068_models_list_command.md AC-02](../../../docs/feature/068_models_list_command.md)

---

### FT-03: Offline mode contains `claude-haiku-4-5-20251001`

- **Given:** Any environment.
- **When:** `clp .models offline::1`
- **Then:** Stdout contains `claude-haiku-4-5-20251001`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft03_offline_contains_haiku`
- **Source:** [068_models_list_command.md AC-03](../../../docs/feature/068_models_list_command.md)

---

### FT-04: Table format has `ID` header

- **Given:** Any environment.
- **When:** `clp .models offline::1 format::table`
- **Then:** First stdout line contains `ID`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft04_offline_table_has_header`
- **Source:** [068_models_list_command.md AC-04](../../../docs/feature/068_models_list_command.md)

---

### FT-05: Text format outputs one ID per line

- **Given:** Any environment.
- **When:** `clp .models offline::1 format::text`
- **Then:** Stdout is a sequence of newline-separated model IDs; no `|` table separators present. Each line matches `^claude-[a-z0-9.-]+$`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft05_offline_text_one_per_line`
- **Source:** [068_models_list_command.md AC-05](../../../docs/feature/068_models_list_command.md)

---

### FT-06: JSON format outputs valid JSON array

- **Given:** Any environment.
- **When:** `clp .models offline::1 format::json`
- **Then:** Stdout is parseable as a JSON array (`[{...}, ...]`); each element has an `"id"` field. Exits 0.
- **Exit:** 0
- **Source fn:** `ft06_offline_json_valid_array`
- **Source:** [068_models_list_command.md AC-06](../../../docs/feature/068_models_list_command.md)

---

### FT-07: `name::opus` returns only models with `opus` in ID

- **Given:** Any environment.
- **When:** `clp .models offline::1 name::opus`
- **Then:** All returned model IDs contain `"opus"`. Models without `"opus"` in the ID are absent from output. Exits 0.
- **Exit:** 0
- **Source fn:** `ft07_name_filter_opus_only`
- **Source:** [068_models_list_command.md AC-07](../../../docs/feature/068_models_list_command.md)

---

### FT-08: `name::zz_no_match` returns empty output with exit 0

- **Given:** Any environment.
- **When:** `clp .models offline::1 name::zz_no_match`
- **Then:** Stdout contains no model IDs. Exits 0.
- **Exit:** 0
- **Source fn:** `ft08_name_filter_no_match`
- **Source:** [068_models_list_command.md AC-08](../../../docs/feature/068_models_list_command.md)

---

### FT-09: `.models` appears in `clp .help` output

- **Given:** Any environment.
- **When:** `clp .help`
- **Then:** Output contains `.models`. Exits 0.
- **Exit:** 0
- **Source fn:** `ft09_models_in_help_output`
- **Source:** [068_models_list_command.md AC-09](../../../docs/feature/068_models_list_command.md)

---

### FT-10: `name::claude-opus` is a substring match

- **Given:** Any environment.
- **When:** `clp .models offline::1 name::claude-opus`
- **Then:** All returned model IDs contain `"claude-opus"` (case-insensitive substring). Haiku and Sonnet models are absent. Exits 0.
- **Exit:** 0
- **Source fn:** `ft10_name_filter_substring`
- **Source:** [068_models_list_command.md AC-10](../../../docs/feature/068_models_list_command.md)
