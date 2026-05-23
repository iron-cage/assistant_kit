# Parameter :: `field::`

Edge case tests for the `field::` parameter. Tests validate String enforcement, the exhaustive list of 8 valid field names, unknown-value rejection with helpful error, default-absent behavior (shows all fields), and `format::` override (raw string output regardless of `format::`). Used by `.paths` to return a single named path value.

**Source:** [params.md#parameter--24-field](../../../../docs/cli/param/24_field.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `field::base` — valid field; returns raw path value | Valid Field |
| EC-2 | `field::credential_store` — valid field; returns path value | Valid Field |
| EC-3 | `field::unknown` — invalid field name; exit 1 with valid-names list | Unknown Field |
| EC-4 | Default is `""` (absent — shows all fields) | Default |
| EC-5 | `field::credentials format::json` — `format::` ignored when `field::` set | Format Override |
| EC-6 | All 8 valid field names accepted | Valid Values |

## Test Coverage Summary

- Valid Field: 2 tests (EC-1, EC-2)
- Unknown Field: 1 test (EC-3)
- Default: 1 test (EC-4)
- Format Override: 1 test (EC-5)
- Valid Values: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (single field mode — raw string output) ↔ EC-4 (absent by default — full path listing)

## Test Cases
---

### EC-1: `field::base` — valid field; returns raw path value

- **Given:** `.paths` environment with a resolvable `base` path.
- **When:** `clp .paths field::base`
- **Then:** stdout contains exactly the resolved `base` path value followed by a newline; no labels, no JSON wrapper; exit 0.
- **Exit:** 0
- **Source fn:** `p09_paths_field_returns_single_value`
- **Source:** [params.md#parameter--24-field](../../../../docs/cli/param/24_field.md)
---

### EC-2: `field::credential_store` — valid field; returns path value

- **Given:** `.paths` environment with a resolvable `credential_store` path.
- **When:** `clp .paths field::credential_store`
- **Then:** stdout contains exactly the resolved `credential_store` path value followed by a newline; no labels, no JSON wrapper; exit 0.
- **Exit:** 0
- **Source fn:** `p09_paths_field_returns_single_value`
- **Source:** [params.md#parameter--24-field](../../../../docs/cli/param/24_field.md)
---

### EC-3: `field::unknown` — invalid field; exit 1 with valid-names list

- **Given:** clean environment.
- **When:** `clp .paths field::unknown`
- **Then:** Exit 1; stderr contains `unknown field 'unknown'` and lists all valid field names: `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions`.
- **Exit:** 1
- **Source fn:** `p10_paths_field_unknown_exits_1`
- **Source:** [params.md#parameter--24-field](../../../../docs/cli/param/24_field.md)
---

### EC-4: Default is `""` (absent — shows all fields)

- **Given:** `.paths` environment.
- **When:** `clp .paths` (no `field::` param)
- **Then:** stdout contains labeled output for all paths (multiple lines); behavior is the full path listing, not a single-field extract; exit 0.
- **Exit:** 0
- **Source fn:** `p02_paths_text_v1_labeled`
- **Source:** [params.md#parameter--24-field](../../../../docs/cli/param/24_field.md)
---

### EC-5: `field::credentials format::json` — `format::` ignored when `field::` set

- **Given:** `.paths` environment with a resolvable `credentials` path.
- **When:** `clp .paths field::credentials format::json`
- **Then:** stdout contains the raw string path value (not a JSON object); `format::json` is overridden by `field::`; exit 0.
- **Exit:** 0
- **Source fn:** `p09_paths_field_returns_single_value`
- **Source:** [params.md#parameter--24-field](../../../../docs/cli/param/24_field.md)
---

### EC-6: All 8 valid field names accepted

- **Given:** `.paths` environment with all paths resolvable.
- **When:** `clp .paths field::F` for each F in `base`, `credentials`, `credential_store`, `projects`, `stats`, `settings`, `session_env`, `sessions`
- **Then:** Each invocation exits 0 and returns a non-empty path string on stdout; no field name from this set is rejected.
- **Exit:** 0
- **Source fn:** `p09_paths_field_returns_single_value`
- **Source:** [params.md#parameter--24-field](../../../../docs/cli/param/24_field.md)
