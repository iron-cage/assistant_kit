# Doc Structure Validation — Feature Collection

Structural compliance validation cases for the `docs/feature/` collection (46 files). Validates that all feature doc instances conform to the per-type H3 section format required by doc.rulebook.md § Feature Documentation : Rule 9a.

These are grep-executable and manual validation cases, not automated behavioral tests. They are the verification surface for the Rule 9a cross-references format migration.

### DT Case Index

| ID | Description | Category |
|----|-------------|----------|
| DT-01 | No unified `### Cross-References` heading in any feature doc | Format |
| DT-02 | No `| Type |` column header surviving in any feature doc | Format |
| DT-03 | No `| Responsibility |` column header surviving in any feature doc | Format |
| DT-04 | `### Sources` section present in all 45 non-deprecated feature docs | Structure |
| DT-05 | `### Tests` section present in all feature docs that had test entries | Structure |
| DT-06 | Section ordering valid — alphabetical body; `### Sources` and `### Tests` always last | Structure |
| DT-07 | Bidirectionality — 6 high-connectivity A↔B pairs each have both directions present | Structure |

**Total:** 7 DT cases

---

### DT-01: No unified Cross-References heading in any feature doc

- **Scope:** All 46 `docs/feature/*.md` files
- **Command:** `grep -rc "### Cross-References" docs/feature/*.md | grep -v ":0$"`
- **Expected:** Empty output — every file returns `:0`
- **Failure:** Any file path appears in output — that file still contains the unified heading

---

### DT-02: No Type column header surviving

- **Scope:** All 46 `docs/feature/*.md` files
- **Command:** `grep -rc "| Type | File | Responsibility |" docs/feature/*.md | grep -v ":0$"`
- **Expected:** Empty output
- **Failure:** Any file still has the 3-column table header from the old format

---

### DT-03: No Responsibility column header surviving

- **Scope:** All 46 `docs/feature/*.md` files
- **Command:** `grep -rc "| Responsibility |" docs/feature/*.md | grep -v ":0$"`
- **Expected:** Empty output
- **Failure:** Any file still uses `Responsibility` instead of `Relationship` as the column header
- **Note:** The new format uses `| File | Relationship |` — two columns, renamed header

---

### DT-04: Sources section present in all non-deprecated feature docs

- **Scope:** All 46 `docs/feature/*.md` files (023 is deprecated and legitimately omits `### Sources`)
- **Command:** `grep -l "### Sources" docs/feature/*.md | wc -l`
- **Expected:** `44`
- **Failure:** Count less than 44 — at least one non-deprecated feature doc is missing `### Sources`
- **Note:** Every active feature doc references source files; `### Sources` must be present. Deprecated feature 023 is excluded. Feature 039 currently also omits `### Sources` (pre-existing gap — separate remediation task).

---

### DT-05: Tests section present in expected files

- **Scope:** Feature docs that reference test files (approximately 24 of 46)
- **Command:** `grep -l "### Tests" docs/feature/*.md | wc -l`
- **Expected:** ≥ 24
- **Failure:** Count below 24 — files with test entries are missing `### Tests`
- **Note:** Files with no test entry (e.g., feature/010) legitimately omit `### Tests`; rule states "only include sections for entity types actually referenced"

---

### DT-06: Section ordering valid

- **Scope:** Manual spot-check — 3 files per migration phase batch (12 files total):
  - Phase 1 batch: `003_account_list.md`, `004_account_use.md`, `008_auto_rotate.md`
  - Phase 2 batch: `009_token_usage.md`, `013_account_limits.md`, `014_rich_account_metadata.md`
  - Phase 3 batch: `017_token_refresh.md`, `022_org_identity_snapshot.md`, `024_session_touch.md`
  - Phase 4 batch: `026_subprocess_model_effort.md`, `028_usage_row_filtering.md`, `031_account_inspect.md`
- **Check per file:**
  1. List all H3 section names in the cross-reference area (between last AC and end of file)
  2. Verify sections are in alphabetical order
  3. Verify `### Sources` appears second-to-last (or last if no Tests)
  4. Verify `### Tests` appears last (when present)
- **Expected:** All 12 files pass ordering check
- **Failure:** Any file has non-alphabetical sections, or `### Sources`/`### Tests` not in terminal position

---

### DT-07: Bidirectionality verified for 6 high-connectivity pairs

- **Scope:** Manual check — 6 pairs from the doc_graph edge set
- **Pairs:**

| Pair | File A | File B | A→B label | B→A label |
|------|--------|--------|-----------|-----------|
| 1 | `009_token_usage.md` | `013_account_limits.md` | historical token context | live rate-limit data |
| 2 | `015_name_shortcut_syntax.md` | `030_account_renewal_override.md` | shortcut applies to renewal | uses name shortcut |
| 3 | `020_usage_sort_strategies.md` | `023_next_account_strategies.md` | integrate next recommendation | relies on sort order |
| 4 | `024_session_touch.md` | `027_account_use_post_switch_touch.md` | post-switch touch delegates | uses session touch primitive |
| 5 | `025_per_machine_active_marker.md` | `032_account_assign.md` | marker write via assign | implements per-machine marker |
| 6 | `026_subprocess_model_effort.md` | `027_account_use_post_switch_touch.md` | params forwarded to post-switch | inherits model/effort params |

- **Check per pair:** Open both files; confirm A's `### Features` section references B, and B's `### Features` section references A (or appropriate section for the referenced type)
- **Expected:** All 12 individual references present (6 pairs × 2 directions)
- **Failure:** Any A→B entry exists without the corresponding B→A entry
