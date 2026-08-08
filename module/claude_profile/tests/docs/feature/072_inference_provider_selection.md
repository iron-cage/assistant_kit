# FT — Feature 072: Inference Provider Selection

### Scope

- **Purpose**: Test cases for the `inference_provider` account field, the `.provider.select` global config command, and Gate 10's rotation-eligibility constraint.
- **Source**: `docs/feature/072_inference_provider_selection.md`
- **Covers**: AC-01 through AC-16

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `.account.save inference_provider::kimi` → field written | `t01_inference_provider_kimi_writes_key` |
| FT-02 | AC-02 | `.account.save` (no `inference_provider::`) → field absent, not written as `"anthropic"` | `t02_inference_provider_omitted_writes_no_key` |
| FT-03 | AC-03 | `.account.save inference_provider::` (empty) → exits 1, no file written | `t03_inference_provider_empty_value_exits_1_no_write` |
| FT-04 | AC-04 | Pre-existing account (no `inference_provider` key) reads as `"anthropic"` | `t05_accounts_default_shows_provider_column_and_anthropic_fallback` (secondary assertion — not a dedicated test) |
| FT-05 | AC-05 | `.accounts` (no `cols::`) shows `Provider` column, default `anthropic` | `t05_accounts_default_shows_provider_column_and_anthropic_fallback` |
| FT-06 | AC-06 | `.accounts cols::-inference_provider` omits the column | `t06_accounts_cols_minus_inference_provider_omits_column` |
| FT-07 | AC-07 | `.provider.select` with no prior selection → `provider.select: anthropic` | `t07_provider_select_get_default_anthropic` |
| FT-08 | AC-08 | `.provider.select id::kimi` → file written; stdout `(selected)` | `t08_provider_select_set_kimi_persists_and_confirms` |
| FT-09 | AC-09 | `.provider.select id::` (empty) → exits 1 | `t10_provider_select_empty_id_exits_1` |
| FT-10 | AC-10 | `.provider.select id::kimi reset::1` → exits 1, mutually exclusive | `t11_provider_select_id_and_reset_mutually_exclusive` |
| FT-11 | AC-11 | `.provider.select reset::1` with selection set → key removed, others preserved | `t09_provider_select_reset_preserves_model_key` |
| FT-12 | AC-12 | `.provider.select reset::1` with no `config.toml` → idempotent exit 0 | *(none — coverage gap; no test calls `reset::1` against a never-created `config.toml`)* |
| FT-13 | AC-13 | `.provider.select format::json` → `{"provider":"anthropic"}` | `t12_provider_select_json_format` |
| FT-14 | AC-14 | `provider = "kimi"` selected; mixed-provider account list; rotation never selects `anthropic`-tagged account, `force::1` included | `test_cc_gate10_provider_mismatch_skips_account`, `test_cc_gate10_mismatch_not_bypassed_by_force_equivalent` |
| FT-15 | AC-15 | No selection made (default `anthropic`); account tagged `inference_provider: "kimi"` never selected by rotation | `test_cc_gate10_provider_mismatch_skips_account`, `test_cc_gate10_empty_and_explicit_anthropic_are_equivalent` |
| FT-16 | AC-16 | `.provider.select` get-mode value unaffected by which account is currently active | *(none — coverage gap; no test switches the active account and re-reads `.provider.select` get-mode)* |

### Notes

- **Source fn staleness note:** FT-01 through FT-16 originally cited `ft01`–`ft16`-prefixed function names in files `tests/cli/inference_provider_test.rs`, `tests/cli/provider_select_test.rs`, and `tests/usage/sort_next_test.rs` — none of these files or names were ever created (confirmed via `git log -S`: the commit that introduced this doc touched no matching `.rs` files). Real coverage exists under different names in `tests/cli/account_provider_test.rs` (FT-01–FT-13) and `tests/usage/sort_next_tests_b.rs` (FT-14–FT-15); citations below have been corrected to the real function names. FT-12 and FT-16 are genuine coverage gaps — no backing test exists for either.
- All FT cases are integration tests in `tests/cli/account_provider_test.rs` (FT-01–FT-13, shared with `tests/docs/cli/command/21_provider_select.md`'s IT-01–IT-12 for FT-07–FT-13), and `tests/usage/sort_next_tests_b.rs` (FT-14–FT-15; FT-16 is a coverage gap).
- All FT cases use a temporary isolated `~/.clr/` directory and/or temporary account store to avoid touching the real user environment.
- FT-01–FT-06 exercise the `{name}.json` field and `.accounts` rendering side of this feature — see `docs/schema/002_account_json.md` SC-7 for the on-disk field contract and `docs/cli/param/073_inference_provider.md` for the parameter contract.
- FT-07–FT-13 exercise `.provider.select` itself — near-identical in content to `tests/docs/cli/command/21_provider_select.md`'s IT-01–IT-12 (same underlying test functions), indexed here under the feature entity for full-feature AC traceability rather than duplicated as distinct tests.
- FT-14–FT-16 exercise Gate 10 — see `tests/docs/algorithm/004_eligibility_gates.md` AC-08 for the eligibility-gate-level test case; FT-14/FT-15 here assert the same behavior from the feature/AC perspective (mixed account lists, with and without an explicit `.provider.select`).
- FT-05: default column set assertion — `Provider` header present with no `cols::` param at all, distinguishing this from opt-in columns like `host`/`role` which require `cols::+host`/`cols::+role`.
- FT-11: seed `config.toml` with `provider = "kimi"` and `other_key = "val"` before calling `reset::1`; verify `other_key` is preserved and `provider` is absent (subsequent get shows `anthropic`).
- FT-14: seed at least one `anthropic`-tagged and one `kimi`-tagged account, both otherwise fully eligible (no other gate firing); assert `find_next_for_strategy()` never returns the `anthropic`-tagged account's index when `provider` is selected as `kimi`, including under `gate_ownership=true` and `force::1`-equivalent conditions.
- FT-15: same setup as FT-14 but with no `.provider.select` ever called — asserts Gate 10 uses the default `anthropic` comparison value, not a no-op.

---

### FT-01: `.account.save inference_provider::kimi` writes the field

- **Given:** Any state.
- **When:** `clp .account.save name::kimi inference_provider::kimi`
- **Then:** `kimi.json` contains `"inference_provider": "kimi"`. Exits 0.
- **Exit:** 0
- **Source fn:** `t01_inference_provider_kimi_writes_key`
- **Source:** [072_inference_provider_selection.md AC-01](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-02: `.account.save` without `inference_provider::` omits the field

- **Given:** Any state.
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** `alice@acme.com.json` has no `inference_provider` key at all. Exits 0.
- **Exit:** 0
- **Source fn:** `t02_inference_provider_omitted_writes_no_key`
- **Source:** [072_inference_provider_selection.md AC-02](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-03: Empty `inference_provider::` exits 1

- **Given:** Any state.
- **When:** `clp .account.save name::kimi inference_provider::`
- **Then:** Exits 1. Stderr names `inference_provider::` as requiring a non-empty value. No file written.
- **Exit:** 1
- **Source fn:** `t03_inference_provider_empty_value_exits_1_no_write`
- **Source:** [072_inference_provider_selection.md AC-03](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-04: Pre-existing account defaults to `anthropic`

> Covered as a secondary assertion within `t05_accounts_default_shows_provider_column_and_anthropic_fallback` (`tests/cli/account_provider_test.rs`) — that test's `alice@test.com` account has no `inference_provider` key and asserts its `Provider` line falls back to `anthropic`.

- **Given:** `legacy.json` exists with no `inference_provider` key (saved before this feature).
- **When:** `clp .accounts name::legacy` (or any read path)
- **Then:** Account is treated as `inference_provider: "anthropic"` — no error, no misclassification.
- **Exit:** 0
- **Source fn:** `t05_accounts_default_shows_provider_column_and_anthropic_fallback` (secondary assertion)
- **Source:** [072_inference_provider_selection.md AC-04](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-05: `.accounts` shows `Provider` column by default

- **Given:** At least one saved account.
- **When:** `clp .accounts`
- **Then:** Output includes a `Provider` column/line showing `anthropic` for accounts with no `inference_provider` key.
- **Exit:** 0
- **Source fn:** `t05_accounts_default_shows_provider_column_and_anthropic_fallback`
- **Source:** [072_inference_provider_selection.md AC-05](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-06: `cols::-inference_provider` hides the column

- **Given:** At least one saved account.
- **When:** `clp .accounts cols::-inference_provider`
- **Then:** Output has no `Provider` column/line.
- **Exit:** 0
- **Source fn:** `t06_accounts_cols_minus_inference_provider_omits_column`
- **Source:** [072_inference_provider_selection.md AC-06](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-07: Get with no prior selection returns `anthropic`

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .provider.select`
- **Then:** Stdout is `provider.select: anthropic\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `t07_provider_select_get_default_anthropic`
- **Source:** [072_inference_provider_selection.md AC-07](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-08: `id::kimi` selects the provider

- **Given:** Any state.
- **When:** `clp .provider.select id::kimi`
- **Then:** `~/.clr/config.toml` contains `provider = "kimi"`. Stdout contains `(selected)`. Exits 0.
- **Exit:** 0
- **Source fn:** `t08_provider_select_set_kimi_persists_and_confirms`
- **Source:** [072_inference_provider_selection.md AC-08](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-09: Empty `id::` exits 1

- **Given:** Any environment.
- **When:** `clp .provider.select id::`
- **Then:** Exits 1. Stderr: `id:: must be a non-empty provider name`.
- **Exit:** 1
- **Source fn:** `t10_provider_select_empty_id_exits_1`
- **Source:** [072_inference_provider_selection.md AC-09](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-10: `id::VALUE reset::1` together exits 1

- **Given:** Any environment.
- **When:** `clp .provider.select id::kimi reset::1`
- **Then:** Exits 1. Stderr contains `mutually exclusive`.
- **Exit:** 1
- **Source fn:** `t11_provider_select_id_and_reset_mutually_exclusive`
- **Source:** [072_inference_provider_selection.md AC-10](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-11: `reset::1` removes key and preserves others

- **Given:** `~/.clr/config.toml` contains `provider = "kimi"` and `other_key = "val"`.
- **When:** `clp .provider.select reset::1`
- **Then:** `provider` key removed; `other_key = "val"` preserved. Stdout is `provider.select: anthropic (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `t09_provider_select_reset_preserves_model_key`
- **Source:** [072_inference_provider_selection.md AC-11](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-12: `reset::1` with no `config.toml` is idempotent

> **Coverage gap** — no test exercises this scenario. `t09_provider_select_reset_preserves_model_key` is the only `reset::1` test and always seeds `config.toml` first; no test calls `reset::1` against a directory where `~/.clr/config.toml` was never created. Same gap as `tests/docs/cli/command/21_provider_select.md`'s IT-06.

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .provider.select reset::1`
- **Then:** Stdout is `provider.select: anthropic (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** *(none — coverage gap)*
- **Source:** [072_inference_provider_selection.md AC-12](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-13: `format::json` returns `provider` key

- **Given:** `~/.clr/config.toml` contains `provider = "kimi"` (or absent).
- **When:** `clp .provider.select format::json`
- **Then:** Stdout is `{"provider":"kimi"}` (or `{"provider":"anthropic"}` when absent). Exits 0.
- **Exit:** 0
- **Source fn:** `t12_provider_select_json_format`
- **Source:** [072_inference_provider_selection.md AC-13](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-14: Gate 10 excludes mismatched provider under explicit selection

- **Given:** `provider = "kimi"` selected; account list contains an `inference_provider: "anthropic"` account and an `inference_provider: "kimi"` account, both otherwise fully eligible.
- **When:** `clp .usage rotate::1` (or auto-switch evaluation) runs, including with `force::1`.
- **Then:** The `anthropic`-tagged account is never selected as next/current target under any `force::1` combination.
- **Exit:** 0
- **Source fn:** `test_cc_gate10_provider_mismatch_skips_account`, `test_cc_gate10_mismatch_not_bypassed_by_force_equivalent` (`tests/usage/sort_next_tests_b.rs`; real tests fix `selected_provider="anthropic"` and tag the excluded account `"kimi"` — the inverse of this case's illustrative `kimi`-selected/`anthropic`-tagged example, but the same Gate 10 exclusion logic)
- **Source:** [072_inference_provider_selection.md AC-14](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-15: Gate 10 excludes mismatched provider under default selection

- **Given:** No `.provider.select` ever called (effective selection `anthropic`); account list contains an `inference_provider: "kimi"` account, otherwise fully eligible, and no `anthropic`-tagged account eligible.
- **When:** `clp .usage rotate::1` (or auto-switch evaluation) runs.
- **Then:** The `kimi`-tagged account is never selected — Gate 10 fires using the default `anthropic` comparison value exactly as it would for an explicit selection.
- **Exit:** 0
- **Source fn:** `test_cc_gate10_provider_mismatch_skips_account`, `test_cc_gate10_empty_and_explicit_anthropic_are_equivalent` (`tests/usage/sort_next_tests_b.rs`)
- **Source:** [072_inference_provider_selection.md AC-15](../../../docs/feature/072_inference_provider_selection.md)

---

### FT-16: `.provider.select` never derives from the current account

> **Coverage gap** — no test exercises this scenario. No existing test switches the active account and then re-reads `.provider.select` get-mode to confirm independence from the account's own `inference_provider` tag.

- **Given:** Current active account has `inference_provider: "kimi"`; `~/.clr/config.toml` has no `provider` key.
- **When:** `clp .provider.select`
- **Then:** Stdout is `provider.select: anthropic\n` — unaffected by the current account's own `inference_provider` tag.
- **Exit:** 0
- **Source fn:** *(none — coverage gap)*
- **Source:** [072_inference_provider_selection.md AC-16](../../../docs/feature/072_inference_provider_selection.md)
