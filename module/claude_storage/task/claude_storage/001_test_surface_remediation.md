# Test Surface Spec Remediation: Fix All Violations and Create Missing Surfaces

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Validated By:** null
- **Validation Date:** null

## Goal

The `tests/docs/` test surface audit (2026-05-24) identified 38 violations across 6 categories
spanning all 38 spec files. These violations make the surface structurally non-compliant with
`test_surface.rulebook.md` and `cli_doc.rulebook.md` and undermine the reliability of test
planning for the `claude_storage` CLI. Fix all violations: rename all 38 spec files from
3-digit to 2-digit naming; reformat all 11 command specs from GWT to `**Command:**/
**Expected behavior:**`; add missing `- **Commands:**` field to all 22 parameter spec files;
fix the case ID prefix in all 5 parameter group specs from EC- to CC-; correct Behavioral
Divergence labels across all 22 parameter specs; fix 3 spec files where divergence pair
members are invalid inputs; fix 8 individual spec content bugs (wrong exit codes, non-
deterministic outcomes, broken prerequisites, duplicate IDs, stale anchors, wrong titles);
fix the readme Status column format in all 5 subdirectory readmes; and create the two missing
test surface spec files in `tests/docs/feature/` and `tests/docs/operation/`.

Task is complete when ALL of the following hold:
1. `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l` = 0
2. `ls tests/docs/feature/*.md | grep -v readme | wc -l` = 1
3. `ls tests/docs/operation/*.md | grep -v readme | wc -l` = 1
4. `grep -rL "Commands:" tests/docs/cli/param/*.md | wc -l` = 0
5. All `tests/docs/cli/param_group/*.md` use CC-N case IDs (no EC- prefix in group specs)
6. All `tests/docs/cli/command/*.md` use `**Command:**` + `**Expected behavior:**` format
7. All subdirectory `readme.md` files use `| Name | Purpose | Status |` column format

## In Scope

**A-1 — Rename all 38 spec files from NNN to NN prefix:**
- `tests/docs/cli/command/001_*.md` through `011_*.md` → `01_*.md` through `11_*.md` (11 files)
- `tests/docs/cli/param/001_*.md` through `022_*.md` → `01_*.md` through `22_*.md` (22 files)
- `tests/docs/cli/param_group/001_*.md` through `005_*.md` → `01_*.md` through `05_*.md` (5 files)
- Update all internal cross-references and readme Responsibility Tables in the same session

**A-2 — Reformat all 11 command specs from GWT to Command/Expected behavior format:**
- `tests/docs/cli/command/` (all 11 files): replace `- **Given:**/When:/Then:` blocks with
  `**Command:**` code block + `**Expected behavior:**` bullets per `test_surface.rulebook.md
  § Spec : Test Case Format`

**A-3 — Add `- **Commands:**` field to all 22 parameter spec test cases:**
- All 22 files in `tests/docs/cli/param/`: insert `- **Commands:** <list>` in every EC-N case
  body, populated from `docs/cli/004_params.md` Commands column for each parameter

**A-4 — Fix EC- prefix to CC- in all 5 parameter group spec files:**
- All 5 files in `tests/docs/cli/param_group/`: rename every `EC-N` case heading and reference
  to `CC-N` per `test_surface.rulebook.md § Inventory : Element Types`

**A-5 — Fix Behavioral Divergence labels in all 22 parameter specs:**
- All 22 files: correct the Behavioral Divergence Pair declaration from
  `EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)` to accurately describe
  the pair content (both members are valid inputs producing different outputs)

**A-6 — Fix readme Status column format in all 5 subdirectory readmes:**
- `tests/docs/cli/command/readme.md`, `param/readme.md`, `param_group/readme.md`,
  `tests/docs/feature/readme.md`, `tests/docs/operation/readme.md`:
  convert Responsibility Table to `| Name | Purpose | Status |` with ⏳/✅ per
  `test_surface.rulebook.md § Index : Readme Parity`

**B-1 — Fix `08_output.md` divergence pair (EC-1 is invalid input):**
- EC-1 ("Required — missing output:: exits with 1") is a missing-parameter error, not a
  valid input. Replace EC-1 with a valid input case (e.g., `output::` with a valid path)
  and move the missing-output test to a different case number. Correct the pair labels.

**B-2 — Fix `11_query.md` divergence pair (both members are invalid):**
- EC-1 ("missing query::") and EC-2 ("empty value") are both error cases. The pair must
  use two valid inputs producing different outputs. Add a valid single-word query as EC-1
  and a valid multi-word phrase as EC-2; renumber error cases accordingly (appending at end
  to avoid ID renumbering of existing stable cases is acceptable if IDs already exist).

**B-3 — Fix `17_topic.md` divergence pair (EC-2 is invalid input):**
- EC-2 ("empty value rejected") is invalid. Replace pair to use two valid inputs: e.g.,
  EC-1 (simple name, produces one path suffix) and EC-2 (name with hyphen, produces
  different path suffix with hyphen preserved).

**C-1 — Fix `08_output.md` EC-6 exit code contradiction:**
- Case title says "exits with 2"; Exit field says `Exit: 0`. Resolve to the correct exit
  code per source behavior (nonexistent parent should exit non-zero). Update title or Exit
  field to be consistent.

**C-2 — Fix `21_count.md` invalid type value `conversation`:**
- EC-1, EC-2, EC-5, EC-6 use `type::conversation` which is not a valid `type::` enum value
  (`uuid|path|all` are the valid values). Replace `type::conversation` with a valid value
  or remove the `type::` parameter if not applicable to the `count::` test cases.

**C-3 — Fix `17_topic.md` EC-4 non-deterministic outcome:**
- Then field says "Either accepted as-is or rejected". Replace with a single deterministic
  expected outcome. Consult source behavior and document the actual result.

**C-4 — Fix `03_session_identification.md` EC-2 missing required `output::`:**
- EC-2 tests `clg .export session_id::test-session-uuid` without `output::` and expects
  exit 0. `.export` requires `output::` (per `08_output.md` EC-1). Add `output::/tmp/...`
  to the When field.

**C-5 — Fix `07_projects.md` duplicate case IDs:**
- IT-41, IT-42, IT-43 each appear twice with different content. Assign unique IDs to the
  duplicate cases by appending at the end (IT-51, IT-52, IT-53) to preserve existing
  stable IDs.

**C-6 — Fix `07_projects.md` IT-40 body/index mismatch:**
- Index: "Empty/malformed meta.json fallback to 'unknown'". Body: tests singular noun
  correctness. Align index entry with the actual test body content.

**C-7 — Fix `09_path.md` EC-3 ambiguous exit:**
- Then field allows "Exit 0 or 1". Set a single deterministic exit code and update the
  Given/When/Then to produce a predictable outcome.

**C-8 — Fix `12_scope.md` EC-7 command switch and over-specification:**
- EC-7 switches from `clg .list` (used in EC-1 through EC-6) to `clg .projects` without
  explanation, and uses a verbatim output block. Align EC-7 to use `clg .list` consistently
  or document the command switch explicitly. Replace verbatim output with observable
  property assertions.

**C-9/C-10 — Fix stale source anchors in `18_type.md` and `19_verbosity.md`:**
- `18_type.md`: Source says `parameter--17-type`; correct to `parameter--18-type`
- `19_verbosity.md`: Source says `parameter--18-verbosity`; correct to `parameter--19-verbosity`

**C-11/C-12 — Fix wrong command names in command spec titles:**
- `08_project_path.md`: Title "Command :: `.path`" → "Command :: `.project.path`"
- `09_project_exists.md`: Title "Command :: `.exists`" → "Command :: `.project.exists`"

**C-13 through C-16 — Fix stale source anchors in command specs 08–11:**
- `08_project_path.md`: `#command--10-path` → `#command--8-project-path`
- `09_project_exists.md`: `#command--11-exists` → `#command--9-project-exists`
- `10_session_dir.md`: `#command--12-sessiondir` → `#command--10-session-dir`
- `11_session_ensure.md`: `#command--13-sessionensure` → `#command--11-session-ensure`

**C-17 — Fix missing `---` separators and trailing colons in `21_count.md` and `22_limit.md`:**
- Add horizontal rule (`---`) between each test case to match format of all other 36 spec files
- Remove trailing colons from case headings (e.g., `### EC-1: ... only:` → `### EC-1: ... only`)

**D-1 — Create `tests/docs/feature/01_cli_tool.md`:**
- Create spec file with ≥ 4 FT-prefixed test cases derived from `docs/feature/001_cli_tool.md`
- Cases must cover: CLI tool is installed, command dispatch works, parameter parsing, error
  handling for unknown commands
- Use `**Command:**` + `**Expected behavior:**` format (feature specs = command-style)

**D-2 — Create `tests/docs/operation/01_migration_guide.md`:**
- Create spec file with ≥ 4 OP-prefixed test cases derived from `docs/operation/001_migration_guide.md`
- Cases must validate operational procedure steps are executable and produce documented outcomes
- Use GWT format (operation specs = procedural steps)

**E-1 — Remove `13_session.md` EC-6 duplicate of EC-1:**
- EC-6 (`clg .list session::default`) is identical to EC-1 in both command and behavior.
  Replace EC-6 with a distinct test (e.g., `session::` filter on `.count` command, or
  session filter with no matching results).

**E-2 — Expand `19_verbosity.md` to cover all 5 applicable commands:**
- Add test cases for `verbosity::` applied to `.list`, `.show`, `.count`, and `.projects`,
  not only `.status`. Each new case must show observably different output from the default.

**E-3 — Document or remove `15_sessions_bool.md` EC-2 undocumented claim:**
- EC-2 asserts `sessions::0 session::default` filters projects but hides sessions. This
  behavior is not documented in source. Verify against source code and either update the
  source doc (`docs/cli/003_parameter_groups.md`) or correct the assertion.

**E-4 — Document or remove `05_scope_configuration.md` EC-6 undocumented default:**
- EC-6 tests `path:: without scope:: defaults to under scope` — not in source doc. Verify
  and update source doc or correct the test case to match documented behavior.

## Out of Scope

- Implementing automated Rust test code (`tests/*.rs`) — that is a separate task
- Modifying source specification documents in `docs/` (except to add missing behavior
  documentation required by E-3 and E-4 fixes)
- Fixing `docs/cli/format/` surface mapping (no target mapping defined in rulebook; defer
  until rulebook is updated to define the mapping)
- Changing the test fixture data or test infrastructure
- Addressing Unicode/concurrency/CLAUDE_STORAGE_ROOT coverage gaps (E-6, E-7) — deferred
  to test implementation task

## Acceptance Criteria

- AC-001: `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l` outputs `0`
- AC-002: `ls tests/docs/feature/*.md | grep -v readme | wc -l` outputs `1`
- AC-003: `ls tests/docs/operation/*.md | grep -v readme | wc -l` outputs `1`
- AC-004: `grep -rL "Commands:" tests/docs/cli/param/*.md | wc -l` outputs `0`
- AC-005: No file in `tests/docs/cli/param_group/` contains `EC-` case headings
- AC-006: No file in `tests/docs/cli/command/` contains `- **Given:**` lines
- AC-007: All 5 subdirectory `readme.md` files contain `| Name | Purpose | Status |` header
- AC-008 (negative): `grep -rn "invalid/rejected path" tests/docs/cli/param/ | wc -l` = 0
- AC-009 (negative): `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l` = 0 (same as AC-001, double-checked)
- AC-010: `07_projects.md` (or `07_projects.md` post-rename) contains no duplicate IT-N IDs

## Work Procedure

1. Read `test_surface.rulebook.md` and `cli_doc.rulebook.md` in full to internalize all format requirements before touching files.
2. **Red phase (verify violations exist):** Run AC-001 and AC-004 to confirm current violation counts match the audit findings before making changes.
3. **Rename phase (A-1):** Rename all 38 spec files from NNN to NN prefix using `git mv` to preserve history. Update all readme Responsibility Table file references in the same session.
4. **Format phase (A-2):** Reformat all 11 command spec files from GWT to `**Command:**/Expected behavior:` format. Verify: `grep -rn "\- \*\*Given:\*\*" tests/docs/cli/command/ | wc -l` = 0.
5. **Commands: field phase (A-3):** Add `- **Commands:** <list>` to every EC-N case in all 22 param spec files. Populate from `docs/cli/004_params.md` Commands column.
6. **CC- prefix phase (A-4):** Rename EC-N → CC-N in all 5 param_group spec files.
7. **Divergence label phase (A-5):** Fix Behavioral Divergence Pair declarations in all 22 param specs.
8. **Readme format phase (A-6):** Convert Responsibility Table headers to `| Name | Purpose | Status |` in all 5 subdirectory readmes.
9. **Content fix phase (B-1 through C-17):** Apply individual spec content fixes in the order listed in In Scope. After each group (B, C), verify affected files are self-consistent.
10. **New spec phase (D-1, D-2):** Create `tests/docs/feature/01_cli_tool.md` and `tests/docs/operation/01_migration_guide.md` with minimum required case counts.
11. **Edge case phase (E-1 through E-4):** Apply edge case and coverage fixes.
12. **Green phase:** Run all acceptance criteria commands (AC-001 through AC-010). All must pass.
13. Update task state to ✅ and write Outcomes section.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| Spec file with NNN prefix | Audit check AC-001 | `find` returns 0 files |
| param spec missing Commands: | AC-004 | `grep -rL` returns 0 files |
| param_group spec with EC- prefix | Audit (A-4) | No EC- in param_group files |
| command spec with Given:/When:/Then: | AC-006 | `grep -rn "Given:"` in command/ = 0 |
| Subdirectory readme missing Status column | AC-007 | All readmes have `| Name | Purpose | Status |` |
| Divergence pair label with "invalid/rejected path" | AC-008 | `grep "invalid/rejected path"` in param/ = 0 |
| `21_count.md` using `type::conversation` | C-2 fix | No `type::conversation` in spec |
| `07_projects.md` with duplicate IT-41, IT-42, IT-43 | C-5 fix | All IT-N IDs unique in file |
| `tests/docs/feature/` directory | D-1 | Contains `01_cli_tool.md` with ≥ 4 FT- cases |
| `tests/docs/operation/` directory | D-2 | Contains `01_migration_guide.md` with ≥ 4 OP- cases |
| `13_session.md` EC-1 and EC-6 identical | E-1 fix | EC-6 tests distinct behavior from EC-1 |

## Validation

### Checklist

- [ ] AC-001: `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l` = 0?
- [ ] AC-002: `ls tests/docs/feature/*.md | grep -v readme | wc -l` = 1?
- [ ] AC-003: `ls tests/docs/operation/*.md | grep -v readme | wc -l` = 1?
- [ ] AC-004: `grep -rL "Commands:" tests/docs/cli/param/*.md | wc -l` = 0?
- [ ] AC-005: No EC- case headings in `tests/docs/cli/param_group/`?
- [ ] AC-006: No `- **Given:**` lines in `tests/docs/cli/command/`?
- [ ] AC-007: All 5 subdirectory readmes use `| Name | Purpose | Status |` format?
- [ ] AC-008: `grep -rn "invalid/rejected path" tests/docs/cli/param/ | wc -l` = 0?
- [ ] AC-010: No duplicate IT-N IDs in the projects spec file?
- [ ] `tests/docs/feature/01_cli_tool.md` exists with ≥ 4 FT- cases?
- [ ] `tests/docs/operation/01_migration_guide.md` exists with ≥ 4 OP- cases?
- [ ] `08_project_path.md` title says "Command :: `.project.path`"?
- [ ] `09_project_exists.md` title says "Command :: `.project.exists`"?
- [ ] `08_output.md` EC-6 title and Exit field agree on the same exit code?
- [ ] `21_count.md` uses no `type::conversation` value?
- [ ] `17_topic.md` EC-4 has a single deterministic expected outcome?

### Measurements

- Total 3-digit-prefixed spec files after fix: 0 (was 38)
- Total param spec files missing Commands: field: 0 (was 22)
- Total param_group spec files using EC- prefix: 0 (was 5)
- Total command spec files using GWT format: 0 (was 11)
- Total spec files with `| Name | Purpose | Status |` readme: 5 (was 0)

### Invariants

- Existing stable case IDs (IT-N, EC-N) in spec files must not be renumbered — only new cases appended at end
- Source specification documents in `docs/cli/` are not modified (read-only for this task)
- All `git mv` renames preserve git history (no `rm` + `create`)

### Anti-faking checks

- Run `find tests/docs -name "[0-9][0-9][0-9]_*.md"` and show its output — must be empty
- Run `grep -rn "\- \*\*Given:\*\*" tests/docs/cli/command/` and show output — must be empty
- Open `tests/docs/feature/01_cli_tool.md` and count FT- case headings — must be ≥ 4
- Open `tests/docs/cli/param_group/01_output_control.md` (post-rename) and show first CC-N heading
- Run `grep -rn "invalid/rejected path" tests/docs/cli/param/` — must return no output

## Related Documentation

- `tests/docs/cli/command/*.md` — All 11 command spec files (targets of A-2)
- `tests/docs/cli/param/*.md` — All 22 parameter spec files (targets of A-3, A-5, B-1, B-2, B-3, C-7, C-9, C-10, C-17, E-1, E-2, E-3)
- `tests/docs/cli/param_group/*.md` — All 5 parameter group spec files (targets of A-4, C-4, E-4)
- `tests/docs/feature/readme.md` — Feature test surface index (created 2026-05-24)
- `tests/docs/operation/readme.md` — Operation test surface index (created 2026-05-24)
- `docs/cli/001_commands.md` — Command source (informs A-2 format)
- `docs/cli/004_params.md` — Parameter source (informs A-3 Commands: values)
- `docs/cli/003_parameter_groups.md` — Group source (informs E-3, E-4 verification)
- `docs/feature/001_cli_tool.md` — Feature source (informs D-1 spec creation)
- `docs/operation/001_migration_guide.md` — Operation source (informs D-2 spec creation)
- Related: claude_runner/task/claude_runner/001_test_surface_remediation.md (same task type, different crate, ✅ Complete)

## Affected Entities

- `tests/docs/cli/command/` — 11 spec files renamed + reformatted
- `tests/docs/cli/param/` — 22 spec files renamed + Commands: added + divergence fixed
- `tests/docs/cli/param_group/` — 5 spec files renamed + EC- → CC- prefix
- `tests/docs/feature/` — new spec file created
- `tests/docs/operation/` — new spec file created

## History

- **[2026-05-24]** `CREATED` — Fix all 38 test surface spec violations identified in the 2026-05-24 audit.

## Verification Record

- **Date:** 2026-05-24
- **Method:** 4 independent parallel Agent subagents (no prior session context)
- **Scope Coherence:** PASS — In Scope has 17 specific bounded items; Out of Scope has 5 named exclusions with rationale; outcome is machine-verifiable.
- **MOST Goal Quality:** PASS — Motivated (audit findings explicit), Observable (file-level changes named), Scoped (single deliverable), Testable (7 runnable shell commands in Goal).
- **Value / YAGNI:** PASS — Grounded in dated audit with specific violation counts; no speculative scope; deferred items explicitly excluded.
- **Implementation Readiness:** PASS — Work Procedure has 13 executable steps with phase labels; Test Matrix has 11 rows; Validation has Checklist (16 items), Measurements (5 targets), Invariants (3), Anti-faking checks (5 runnable commands).
- **Result:** ALL 4 PASS — task promoted to 🎯 Verified.
