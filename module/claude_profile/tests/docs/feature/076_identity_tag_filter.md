# FT — Feature 076: Identity Tag Filter

### Scope

- **Purpose**: Test cases for the per-Identity Tag Filter — the `_filter_{machine}_{user}` store file, `.identity.filter` get/set/clear, `.identities` listing, Gate 11 rotation exclusion, write-time guards, and loud exclusion reporting.
- **Source**: `docs/feature/076_identity_tag_filter.md`
- **Covers**: AC-01 through AC-16

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | Get with no filter file → `include=[] exclude=[] (permit-all)`, exit 0 | `identity_filter_t01_filter_get_permit_all` |
| FT-02 | AC-02 | `include::a,b` writes sorted deduplicated include, empty exclude | `identity_filter_t02_filter_include_write` |
| FT-03 | AC-03 | Both sides in one call; each given side fully replaces | `identity_filter_t03_filter_both_sides_replace` |
| FT-04 | AC-04 | `include ∩ exclude ≠ ∅` → exit 1 naming overlap, nothing written | `identity_filter_t04_filter_overlap_exits_1` |
| FT-05 | AC-05 | Invalid tag in either set → exit 1 naming it, nothing written | `identity_filter_t05_filter_invalid_tag_exits_1` |
| FT-06 | AC-06 | `clear::1` deletes; idempotent when absent; with `include::`/`exclude::` → exit 1 | `identity_filter_t06_filter_clear_idempotent_and_exclusive` |
| FT-07 | AC-07 | `identity::USER@MACHINE` targets that seat's filter for get/set/clear | `identity_filter_t07_filter_identity_targeting` |
| FT-08 | AC-08 | Include matching zero tagged accounts → stderr warning, exit 0 | `identity_filter_t08_filter_typo_guard_warns` |
| FT-09 | AC-09 | Gate 11 — automatic selection never picks a filter-failing account, any `force::1` | `test_cc_gate11_tag_mismatch_skips_account`, `test_cc_gate11_not_bypassed_by_force_equivalent` |
| FT-10 | AC-10 | `.account.use name::X` succeeds regardless of filter | `identity_filter_t10_account_use_ignores_filter` |
| FT-11 | AC-11 | No filter file → selection byte-identical to pre-feature (permit-all) | `test_cc_gate11_absent_filter_permit_all` |
| FT-12 | AC-12 | Untagged account fails non-empty include, passes any exclude | `test_cc_gate11_untagged_fails_include_passes_exclude` |
| FT-13 | AC-13 | Gate 11 excluded ≥1 → `.usage` prints `N excluded by tag filter …`; none → no line | `identity_filter_t13_usage_reports_excluded_count` |
| FT-14 | AC-14 | `.identities` unions markers + filters + owners; `(no identities)` exit 0 | `identity_filter_t14_identities_lists_union` |
| FT-15 | AC-15 | `.identities`/`.identity.filter` `format::json`; other formats exit 1 | `identity_filter_t15_identity_commands_json` |
| FT-16 | AC-16 | Filename `_filter_{machine}_{user}`, active-marker sanitization, not matched by `_active_*` ignore | `identity_filter_t16_filter_filename_derivation` |

### Notes

- ✅ Implemented — CLI cases live in `tests/cli/identity_filter_test.rs` (fn names carry the `identity_filter_` file prefix); Gate 11 cases in `tests/usage/sort_next_tests_b.rs` (`test_cc_gate11_*`, following the existing `test_cc_gate10_*` pattern).
- All FT cases must use a temporary isolated credential store and controlled `$USER`/`$HOSTNAME` (or injected Identity) — the filter filename depends on the current Identity.
- FT-09/FT-11/FT-12 are `find_first_eligible()`-level tests: seed accounts differing only in tags, otherwise fully eligible (no other gate firing), and assert selection outcomes across include-only, exclude-only, and combined filters.
- FT-09 must assert all three automatic paths are bound: `rotate::1` winner, auto-switch evaluation, and the footer `Next` recommendation.
- FT-13's positive case needs ≥1 account excluded by Gate 11 specifically (not by Gates 1–10) so the reported count is attributable; the negative case asserts the line's complete absence.
- FT-16 asserts sibling-convention parity with `docs/schema/005_active_marker.md`'s derivation (charset keep-set, `_` replacement) and that the filter file is not matched by the `_active_*` gitignore convention (store-sync intent, `docs/schema/009_identity_filter_json.md`).
- Command-level IT specs sharing these scenarios: `tests/docs/cli/command/24_identity_filter.md` (FT-01–FT-08), `tests/docs/cli/command/23_identities.md` (FT-14–FT-15).

---

### FT-01: Get with no filter file is permit-all

- **Given:** No `_filter_*` file exists for the current Identity.
- **When:** `clp .identity.filter`
- **Then:** Stdout is `include=[] exclude=[] (permit-all)`. Exits 0. No file created.
- **Exit:** 0
- **Source fn:** `identity_filter_t01_filter_get_permit_all`
- **Source:** [076_identity_tag_filter.md AC-01](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-02: `include::` writes sorted deduplicated set

- **Given:** No filter file for the current Identity.
- **When:** `clp .identity.filter include::kimi_pool,ci,kimi_pool`
- **Then:** `_filter_{machine}_{user}` contains `{"include": ["ci", "kimi_pool"], "exclude": []}`. Exits 0.
- **Exit:** 0
- **Source fn:** `identity_filter_t02_filter_include_write`
- **Source:** [076_identity_tag_filter.md AC-02](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-03: Each given side fully replaces

- **Given:** Filter file has `{"include": ["ci"], "exclude": ["personal"]}`.
- **When:** `clp .identity.filter include::kimi_pool` then `clp .identity.filter include::a exclude::b`
- **Then:** After first call: include `["kimi_pool"]`, exclude untouched `["personal"]`. After second: include `["a"]`, exclude `["b"]` (both replaced in one invocation).
- **Exit:** 0
- **Source fn:** `identity_filter_t03_filter_both_sides_replace`
- **Source:** [076_identity_tag_filter.md AC-03](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-04: Include/exclude overlap exits 1

- **Given:** Any state (including a pre-existing exclude the new include would collide with).
- **When:** `clp .identity.filter include::a exclude::a`; also `clp .identity.filter include::x` against an existing `{"exclude": ["x"]}`
- **Then:** Exits 1; stderr names the overlapping tag(s); the filter file is unchanged (or still absent).
- **Exit:** 1
- **Source fn:** `identity_filter_t04_filter_overlap_exits_1`
- **Source:** [076_identity_tag_filter.md AC-04](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-05: Invalid tag in either set exits 1

- **Given:** Any state.
- **When:** `clp .identity.filter include::Bad!Tag`; `clp .identity.filter exclude::Bad!Tag`
- **Then:** Exits 1; stderr names the offending tag (post-lowercasing form); nothing written.
- **Exit:** 1
- **Source fn:** `identity_filter_t05_filter_invalid_tag_exits_1`
- **Source:** [076_identity_tag_filter.md AC-05](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-06: `clear::1` deletes, is idempotent, and excludes set params

- **Given:** Filter file exists.
- **When:** `clp .identity.filter clear::1` (twice); then `clp .identity.filter clear::1 include::ci`
- **Then:** First clear deletes the file (exit 0); second clear with no file is still exit 0; `clear::1` combined with `include::`/`exclude::` exits 1.
- **Exit:** 0 / 0 / 1
- **Source fn:** `identity_filter_t06_filter_clear_idempotent_and_exclusive`
- **Source:** [076_identity_tag_filter.md AC-06](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-07: `identity::` targets another seat

- **Given:** Current Identity is `alice@desk`.
- **When:** `clp .identity.filter identity::bob@laptop include::ci`, then `clp .identity.filter identity::bob@laptop`, then `clp .identity.filter identity::bob@laptop clear::1`; also `identity::bob` (malformed)
- **Then:** Set/get/clear all operate on `_filter_laptop_bob`, never `alice@desk`'s file; malformed `identity::` (not exactly one `@` with both halves non-empty) exits 1.
- **Exit:** 0 (get/set/clear) / 1 (malformed)
- **Source fn:** `identity_filter_t07_filter_identity_targeting`
- **Source:** [076_identity_tag_filter.md AC-07](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-08: Typo guard warns on zero-match include

- **Given:** No account carries `typo_tag`; some accounts carry other tags.
- **When:** `clp .identity.filter include::typo_tag`
- **Then:** Write succeeds (file written, exit 0); stderr contains a warning naming `typo_tag` as carried by no account.
- **Exit:** 0
- **Source fn:** `identity_filter_t08_filter_typo_guard_warns`
- **Source:** [076_identity_tag_filter.md AC-08](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-09: Gate 11 excludes filter-failing accounts unconditionally

- **Given:** Current Identity's filter: `{"include": ["kimi_pool"], "exclude": ["personal"]}`. Accounts (all otherwise fully eligible): `a1` tags `[kimi_pool]`, `a2` tags `[ci]` (fails include), `a3` tags `[kimi_pool, personal]` (fails exclude).
- **When:** Automatic selection runs — `rotate::1`, auto-switch evaluation, footer `Next` recommendation — including with `force::1`.
- **Then:** Only `a1` is ever selected/recommended; `a2` and `a3` are never chosen under any `force::1` combination.
- **Exit:** 0
- **Source fn:** `test_cc_gate11_tag_mismatch_skips_account`, `test_cc_gate11_not_bypassed_by_force_equivalent` *(`tests/usage/sort_next_tests_b.rs`)*
- **Source:** [076_identity_tag_filter.md AC-09](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-10: `.account.use name::X` is never filtered

- **Given:** Current Identity's filter excludes everything `X` carries (e.g. include `[kimi_pool]`, `X` untagged).
- **When:** `clp .account.use name::X`
- **Then:** Switch succeeds exactly as without any filter — explicit selection bypasses Gate 11 entirely.
- **Exit:** 0
- **Source fn:** `identity_filter_t10_account_use_ignores_filter`
- **Source:** [076_identity_tag_filter.md AC-10](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-11: Absent filter file is exact pre-feature behavior

- **Given:** No `_filter_*` file for the current Identity; mixed tagged/untagged accounts.
- **When:** Automatic selection runs.
- **Then:** Selection outcome and output are byte-identical to pre-feature behavior — permit-all, no exclusion line, zero migration.
- **Exit:** 0
- **Source fn:** `test_cc_gate11_absent_filter_permit_all` *(`tests/usage/sort_next_tests_b.rs`)*
- **Source:** [076_identity_tag_filter.md AC-11](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-12: Untagged account semantics

- **Given:** Account `u` has no `tags` key. Filter variants: `{"include": ["ci"]}` and `{"exclude": ["ci"]}`.
- **When:** Automatic selection evaluates `u` under each variant.
- **Then:** Under non-empty include: `u` is excluded (carries nothing ⊉ include). Under exclude-only: `u` passes trivially (empty ∩ exclude = ∅).
- **Exit:** 0
- **Source fn:** `test_cc_gate11_untagged_fails_include_passes_exclude` *(`tests/usage/sort_next_tests_b.rs`)*
- **Source:** [076_identity_tag_filter.md AC-12](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-13: Loud exclusion reporting on `.usage`

- **Given:** Filter excludes ≥1 otherwise-eligible account (Gate 11 specifically, not Gates 1–10).
- **When:** `clp .usage` selection pass runs (recommendation or `rotate::1`).
- **Then:** Output includes `N excluded by tag filter include=[…] exclude=[…]` with the correct count and sets; with a filter excluding nothing, no such line appears.
- **Exit:** 0
- **Source fn:** `identity_filter_t13_usage_reports_excluded_count`
- **Source:** [076_identity_tag_filter.md AC-13](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-14: `.identities` unions all three sources

- **Given:** `_active_*` marker for `alice@desk`; `_filter_*` file for `bob@laptop` (no marker, owns nothing); account owned by `carol@ws1` (no marker, no filter).
- **When:** `clp .identities`
- **Then:** Three sorted rows — each Identity present with Active (account or `—`), Owned count, Include/Exclude (or `—`). Empty store prints `(no identities)`, exits 0.
- **Exit:** 0
- **Source fn:** `identity_filter_t14_identities_lists_union`
- **Source:** [076_identity_tag_filter.md AC-14](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-15: `format::json` on both commands; other formats exit 1

- **Given:** Fixture as FT-14; a filter file present.
- **When:** `clp .identities format::json`; `clp .identity.filter format::json`; `clp .identities format::table`
- **Then:** JSON outputs carry the equivalent structured data (`identity`/`active`/`owned`/`include`/`exclude`; `identity`/`include`/`exclude`); `format::table` (or any non-text/json) exits 1.
- **Exit:** 0 / 0 / 1
- **Source fn:** `identity_filter_t15_identity_commands_json`
- **Source:** [076_identity_tag_filter.md AC-15](../../../docs/feature/076_identity_tag_filter.md)

---

### FT-16: Filename derivation and sync intent

- **Given:** Current Identity with characters requiring sanitization (e.g. user `john doe`).
- **When:** `clp .identity.filter include::ci`
- **Then:** File is named `_filter_{machine}_{user}` with the same keep-charset/`_`-replacement as `active_marker_filename()`; the name does not match the `_active_*` ignore pattern (store-sync intent).
- **Exit:** 0
- **Source fn:** `identity_filter_t16_filter_filename_derivation`
- **Source:** [076_identity_tag_filter.md AC-16](../../../docs/feature/076_identity_tag_filter.md)
