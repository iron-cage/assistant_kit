# Test: Type 02 — OutputFormat

Boundary and validation test cases for the `OutputFormat` type. See
[docs/cli/type/002_output_format.md](../../../../docs/cli/type/002_output_format.md) for the
type specification.

`OutputFormat` is an enum selecting between `text`, `json`, and `table` rendering modes. `table`
is accepted only by `.accounts`; other commands reject it. Unknown values are always rejected.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `"text"` parsed to TEXT variant | Valid (default) |
| TC-2 | `"json"` parsed to JSON variant | Valid (structured) |
| TC-3 | `"table"` parsed to TABLE variant | Valid (compact) |
| TC-4 | Case-insensitive input `"JSON"` parsed correctly | Valid (case fold) |
| TC-5 | Unknown value `"csv"` rejected | Invalid |

**Total:** 5 TC cases

---

### TC-1: `"text"` parsed to TEXT variant

- **Given:** The string `"text"`
- **When:** `OutputFormat::new("text")` is called
- **Then:** Returns `Ok(OutputFormat::Text)`; `is_text()` returns `true`
- **Source:** [docs/cli/type/002_output_format.md](../../../../docs/cli/type/002_output_format.md)

---

### TC-2: `"json"` parsed to JSON variant

- **Given:** The string `"json"`
- **When:** `OutputFormat::new("json")` is called
- **Then:** Returns `Ok(OutputFormat::Json)`; `is_json()` returns `true`
- **Source:** [docs/cli/type/002_output_format.md](../../../../docs/cli/type/002_output_format.md)

---

### TC-3: `"table"` parsed to TABLE variant

- **Given:** The string `"table"`
- **When:** `OutputFormat::new("table")` is called
- **Then:** Returns `Ok(OutputFormat::Table)`; `is_table()` returns `true`
- **Source:** [docs/cli/type/002_output_format.md](../../../../docs/cli/type/002_output_format.md)

---

### TC-4: Case-insensitive input parsed correctly

- **Given:** The string `"JSON"` (uppercase)
- **When:** `OutputFormat::new("JSON")` is called
- **Then:** Returns `Ok(OutputFormat::Json)` — parsing is case-insensitive per the type
  constraint "case-insensitive"
- **Source:** [docs/cli/type/002_output_format.md](../../../../docs/cli/type/002_output_format.md)

---

### TC-5: Unknown value rejected

- **Given:** The string `"csv"` (not a defined variant)
- **When:** `OutputFormat::new("csv")` is called
- **Then:** Returns `Err(…)` — unknown output format values are rejected
- **Source:** [docs/cli/type/002_output_format.md](../../../../docs/cli/type/002_output_format.md)
