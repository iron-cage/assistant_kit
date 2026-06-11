# Test: Type 03 — WarningThreshold

Boundary and validation test cases for the `WarningThreshold` type. See
[docs/cli/type/003_warning_threshold.md](../../../../docs/cli/type/003_warning_threshold.md) for
the type specification.

`WarningThreshold` wraps a `u64` value in seconds. `0` disables the ExpiringSoon classification;
`3600` is the default. No upper bound — any non-negative integer is accepted.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `"0"` parsed to threshold 0 (disabled) | Valid (boundary min) |
| TC-2 | `"3600"` parsed to default threshold | Valid (default) |
| TC-3 | Large value `"86400"` accepted | Valid (boundary large) |
| TC-4 | Non-numeric string `"abc"` rejected | Invalid (type) |

**Total:** 4 TC cases

---

### TC-1: `"0"` parsed to threshold 0 (disabled)

- **Given:** The string `"0"`
- **When:** `WarningThreshold::new("0")` is called
- **Then:** Returns `Ok(WarningThreshold)` with `get()` returning `0`; `is_disabled()` returns
  `true`
- **Source:** [docs/cli/type/003_warning_threshold.md](../../../../docs/cli/type/003_warning_threshold.md)

---

### TC-2: `"3600"` parsed to default threshold

- **Given:** The string `"3600"` (the documented default value)
- **When:** `WarningThreshold::new("3600")` is called
- **Then:** Returns `Ok(WarningThreshold)` with `get()` returning `3600`; `is_disabled()` returns
  `false`
- **Source:** [docs/cli/type/003_warning_threshold.md](../../../../docs/cli/type/003_warning_threshold.md)

---

### TC-3: Large value `"86400"` accepted

- **Given:** The string `"86400"` (24 hours in seconds)
- **When:** `WarningThreshold::new("86400")` is called
- **Then:** Returns `Ok(WarningThreshold)` with `get()` returning `86400` — no upper bound
  constraint is enforced
- **Source:** [docs/cli/type/003_warning_threshold.md](../../../../docs/cli/type/003_warning_threshold.md)

---

### TC-4: Non-numeric string rejected

- **Given:** The string `"abc"` (not parseable as a non-negative integer)
- **When:** `WarningThreshold::new("abc")` is called
- **Then:** Returns `Err(…)` — non-integer input is rejected with a parse error
- **Source:** [docs/cli/type/003_warning_threshold.md](../../../../docs/cli/type/003_warning_threshold.md)
