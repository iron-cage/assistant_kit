# Test: `imodel::` Parameter

Edge case coverage for the `imodel::` parameter on `.usage`. See [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `imodel::auto` accepted with empty credential store | Valid Value |
| EC-2 | `imodel::sonnet` accepted with empty credential store | Valid Value |
| EC-3 | `imodel::opus` accepted with empty credential store | Valid Value |
| EC-4 | `imodel::keep` accepted with empty credential store | Valid Value |
| EC-5 | `imodel::bad` exits 1, stderr names all four valid values | Invalid Value |
| EC-6 | `imodel::sonnet` — args contain `--model claude-sonnet-4-6` | Arg Construction |
| EC-7 | `imodel::opus` — args contain `--model claude-opus-4-6` | Arg Construction |
| EC-8 | `imodel::keep` — args contain no `--model` flag | Arg Construction |
| EC-9 | `imodel::auto` with `7d(Son) < 30%` → subprocess uses opus | Behavioral Divergence |
| EC-10 | `imodel::auto` with `7d(Son) ≥ 30%` → subprocess uses sonnet | Behavioral Divergence |

---

### EC-1: `imodel::auto` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::auto`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it112_imodel_auto_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)

---

### EC-2: `imodel::sonnet` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::sonnet`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned (no accounts to touch).
- **Exit:** 0
- **Source fn:** `it117_imodel_sonnet_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)

---

### EC-3: `imodel::opus` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::opus`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it118_imodel_opus_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)

---

### EC-4: `imodel::keep` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage imodel::keep`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it119_imodel_keep_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/035_imodel.md](../../../../docs/cli/param/035_imodel.md)

---

### EC-5: `imodel::bad` exits 1 (invalid value)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage imodel::bad`
- **Then:** Exits 1. Stderr contains each of the four valid values: `auto`, `sonnet`, `opus`, `keep`.
- **Exit:** 1
- **Source fn:** `it113_imodel_bogus_exits_1` (in `tests/cli/usage_test.rs`)
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

### EC-7: `imodel::opus` — subprocess arg slice contains `--model claude-opus-4-6`

- **Given:** One account with valid quota data; `imodel::opus`.
- **When:** Unit test of `resolve_model(quota, "opus")`
- **Then:** Returns `IsolatedModel::Specific("claude-opus-4-6")`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_opus_explicit` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-03](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-8: `imodel::keep` — subprocess arg slice contains no `--model` flag

- **Given:** One account; `imodel::keep`.
- **When:** Unit test of `resolve_model(quota, "keep")`
- **Then:** Returns `IsolatedModel::KeepCurrent`. The arg slice contains neither `--model` nor any model string.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_keep_no_model_flag` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-04](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-9: `imodel::auto` with `7d(Son) < 30%` → subprocess uses opus (Behavioral Divergence A)

- **Given:** Account quota data where `7d(Son)` utilization is 75% (25% remaining — below threshold); `imodel::auto`.
- **When:** Unit test of `resolve_model(quota_with_son_25_pct, "auto")`
- **Then:** Returns `IsolatedModel::Specific("claude-opus-4-6")`.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_opus_when_sonnet_low` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../../docs/feature/026_subprocess_model_effort.md)

---

### EC-10: `imodel::auto` with `7d(Son) ≥ 30%` → subprocess uses sonnet (Behavioral Divergence B)

- **Given:** Account quota data where `7d(Son)` utilization is 65% (35% remaining — above threshold); same input as EC-9 except utilization differs.
- **When:** Unit test of `resolve_model(quota_with_son_35_pct, "auto")`
- **Then:** Returns `IsolatedModel::Specific("claude-sonnet-4-6")`. Divergence from EC-9: the SAME `imodel::auto` produces DIFFERENT model selection based on remaining quota, proving the threshold governs selection.
- **Exit:** n/a (unit test)
- **Source fn:** `it_imodel_auto_selects_sonnet_above_threshold` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/026_subprocess_model_effort.md AC-01](../../../../docs/feature/026_subprocess_model_effort.md)
