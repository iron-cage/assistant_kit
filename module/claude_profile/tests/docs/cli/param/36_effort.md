# Test: `effort::` Parameter

Edge case coverage for the `effort::` parameter on `.usage`. For `.account.use` `effort::` validation, see [command/005_account_use.md](../command/05_account_use.md) (IT-22). See [param/036_effort.md](../../../../docs/cli/param/036_effort.md) for specification.

**Behavioral Divergence Pair:** EC-1 ↔ EC-4 — `effort::auto` (valid value) exits 0 with "(no accounts configured)"; `effort::bad` (invalid value) exits 1 with error listing all five valid values.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `effort::auto` accepted with empty credential store | Valid Value |
| EC-2 | `effort::high` accepted with empty credential store | Valid Value |
| EC-3 | `effort::max` accepted with empty credential store | Valid Value |
| EC-4 | `effort::bad` exits 1, stderr names all five valid values | Invalid Value |
| EC-10 | `effort::low` accepted with empty credential store | Valid Value |
| EC-11 | `effort::normal` accepted with empty credential store | Valid Value |

---

### EC-1: `effort::auto` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::auto`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it124_effort_auto_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-2: `effort::high` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::high`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it130_effort_high_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-3: `effort::max` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::max`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it131_effort_max_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-4: `effort::bad` exits 1 (invalid value)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage effort::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `low`, `normal`, `high`, `max`.
- **Exit:** 1
- **Source fn:** `it125_effort_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-11](../../../../docs/feature/026_subprocess_model_effort.md)

---

> **Note:** EC-5, EC-6, EC-7, EC-8, EC-9, EC-12, EC-13, EC-14 removed — subprocess arg injection not directly observable via clp binary output — behavior only verifiable at unit-test level. Unit tests live in `tests/cli/usage_test.rs` under the `it_effort_` and `it_imodel_` prefixes.

---

### EC-10: `effort::low` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::low`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it143_effort_low_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)

---

### EC-11: `effort::normal` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage effort::normal`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it144_effort_normal_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/036_effort.md](../../../../docs/cli/param/036_effort.md)
