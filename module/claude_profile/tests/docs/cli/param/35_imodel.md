# Test: `imodel::` Parameter

Edge case coverage for the `imodel::` parameter on `.usage`. For `.account.use` `imodel::` validation, see [command/005_account_use.md](../command/05_account_use.md) (IT-21). See [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `imodel::auto` accepted with empty credential store | Valid Value |
| EC-2 | `imodel::sonnet` accepted with empty credential store | Valid Value |
| EC-3 | `imodel::opus` accepted with empty credential store | Valid Value |
| EC-4 | `imodel::keep` accepted with empty credential store | Valid Value |
| EC-5 | `imodel::bad` exits 1, stderr names all five valid values | Invalid Value |
| EC-6 | `imodel::sonnet` — args contain `--model claude-sonnet-4-6` | Arg Construction |
| EC-11 | `imodel::haiku` accepted with empty credential store | Valid Value |

---

### EC-1: `imodel::auto` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::auto`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it122_imodel_auto_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)

---

### EC-2: `imodel::sonnet` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::sonnet`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned (no accounts to touch).
- **Exit:** 0
- **Source fn:** `it127_imodel_sonnet_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)

---

### EC-3: `imodel::opus` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::opus`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it128_imodel_opus_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)

---

### EC-4: `imodel::keep` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::keep`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it129_imodel_keep_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)

---

### EC-5: `imodel::bad` exits 1 (invalid value)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage imodel::bad`
- **Then:** Exits 1. Stderr contains each of the five valid values: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source fn:** `it123_imodel_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-10](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-6: `imodel::sonnet` — subprocess arg slice contains `--model claude-sonnet-4-6`

- **Given:** One account with valid quota data and idle 5h window (`five_hour.resets_at = None`); `imodel::sonnet touch::1`.
- **When:** `clp .usage imodel::sonnet touch::1 trace::1`
- **Then:** Exits 0. `resolve_model()` returns `IsolatedModel::Specific("claude-sonnet-4-6")`. Args passed to `run_isolated()` include `--model claude-sonnet-4-6`. Verified via unit test on `resolve_model()` with `imodel="sonnet"`.
- **Exit:** 0
- **Source fn:** `it_imodel_sonnet_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-02](../../../../docs/feature/026_subprocess_model_effort.md)

---

> **Note:** EC-7, EC-8, EC-9, EC-10, EC-12 removed — subprocess arg construction not directly observable via clp binary output — behavior only verifiable at unit-test level. Unit tests live in `tests/cli/usage_test.rs` under the `it_imodel_` prefix.

---

### EC-11: `imodel::haiku` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::haiku`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned (no accounts to touch).
- **Exit:** 0
- **Source fn:** `it142_imodel_haiku_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)
