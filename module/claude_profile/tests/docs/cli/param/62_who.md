# Test: `who::` Parameter

Edge case coverage for the `who::` bool param on `.usage`.
See [param/061_who.md](../../../../docs/cli/param/061_who.md) for specification.

`who::` controls sessions table visibility in `.usage` output. Default behavior (auto): shown when >1 `_active_*` marker exists; hidden when ≤1.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `who::` omitted (auto) — sessions table shown when >1 marker | Behavioral Divergence |
| EC-2 | `who::` omitted (auto) — sessions table hidden when ≤1 marker | Behavioral Divergence |
| EC-3 | `who::1` — sessions table forced on even with ≤1 marker | Force On |
| EC-4 | `who::0` — sessions table suppressed even with >1 marker | Force Off |
| EC-5 | `who::1` with 0 markers — sessions table shown but empty | Edge: Empty |
| EC-6 | `who::true` rejected — non-integer value on bool param; exits 1 | Type Rejection |
| EC-7 | `who::2` rejected — integer outside {0, 1}; exits 1 | Out-of-Range |

---

### EC-1: `who::` omitted (auto) — sessions table shown when >1 marker

- **Given:** A credential store with 3 `_active_*` marker files (own + 2 others).
- **When:** `clp .usage` (no `who::` parameter)
- **Then:** Exits 0. Output includes a sessions table after the footer. Table contains rows for each `_active_*` marker; own session marked with `✓`.
- **Exit:** 0
- **Source fn:** `ft30_009_sessions_table_shown_auto_multiple_markers` (in `src/usage/render_tests.rs`)
- **Source:** [param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### EC-2: `who::` omitted (auto) — sessions table hidden when ≤1 marker

- **Given:** A credential store with only the current machine's own `_active_*` marker (no other markers).
- **When:** `clp .usage` (no `who::` parameter)
- **Then:** Exits 0. No sessions table appears in output.
- **Exit:** 0
- **Source fn:** `ft31_009_sessions_table_hidden_auto_single_marker` (in `src/usage/render_tests.rs`)
- **Source:** [param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### EC-3: `who::1` — sessions table forced on even with ≤1 marker

- **Given:** A credential store with only the current machine's own `_active_*` marker.
- **When:** `clp .usage who::1`
- **Then:** Exits 0. Sessions table appears showing only the own session row (marked `✓`).
- **Exit:** 0
- **Source fn:** `ft32_009_sessions_table_who_override` (who=Some(true) arm) (in `src/usage/render_tests.rs`)
- **Source:** [param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### EC-4: `who::0` — sessions table suppressed even with >1 marker

- **Given:** A credential store with 3 `_active_*` marker files.
- **When:** `clp .usage who::0`
- **Then:** Exits 0. No sessions table in output despite multiple markers.
- **Exit:** 0
- **Source fn:** `ft32_009_sessions_table_who_override` (who=Some(false) arm) (in `src/usage/render_tests.rs`)
- **Source:** [param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### EC-5: `who::1` with 0 markers — sessions table omitted gracefully

- **Given:** A credential store with no `_active_*` marker files.
- **When:** `clp .usage who::1`
- **Then:** Exits 0. `build_sessions_table` returns an empty string; the `if show && !sessions_text.is_empty()` guard suppresses the append even with `who=Some(true)` — no `Sessions` heading appears in output.
- **Exit:** 0
- **Source fn:** `ec5_062_who_force_on_zero_markers_omits_gracefully` (in `src/usage/render_tests.rs`)
- **Source:** [param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### EC-6: `who::true` rejected — `Kind::Integer` rejects non-integer string at framework level

- **Given:** Any environment.
- **When:** `clp .usage who::true`
- **Then:** Exits 1. `who::` is registered as `Kind::Integer`; the unilang routing layer calls `"true".parse::<i64>()` which fails before `parse_usage_params` is reached. Contrast: `Kind::String` params (e.g. `only_next::`) pass "true" as `Value::String` and `parse_int_flag` maps it to `Ok(1)`.
- **Exit:** 1
- **Source fn:** `it256_who_true_rejected_kind_integer` (in `tests/cli/usage_test.rs`)
- **Source:** [param/061_who.md](../../../../docs/cli/param/061_who.md)

---

### EC-7: `who::2` rejected — integer outside {0, 1}; exits 1

- **Given:** Any environment.
- **When:** `clp .usage who::2`
- **Then:** Exits 1. `parse_int_flag` rejects integers outside `{0, 1}`.
- **Exit:** 1
- **Source fn:** `ec7_who_rejects_integer_two` (in `src/usage/params.rs`)
- **Source:** [param/061_who.md](../../../../docs/cli/param/061_who.md)
