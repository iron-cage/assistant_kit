# Test Surface Spec Remediation

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** 🎯 (Verified)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** `module/claude_version/tests/docs/`
- **Validated By:** claude-opus-4-6 (MAAV)
- **Validation Date:** 2026-06-16

## Goal

The test surface audit identified five categories of violations in `tests/docs/` that misrepresent actual spec quality and make automated Coverage Gate checks unreliable. This task remediates all five: (1) renames 52 spec files from `NNN_` to `NN_` prefix per `l1_imp_surface.rulebook.md § Spec : File Naming`; (2) replaces all `cm`/`clm` binary name references with `clv` (the actual binary name per `src/bin/clv.rs`); (3) normalizes all seven type spec files to use `TC-` prefix exclusively; (4) fixes four dead cross-references pointing to non-existent flat-file doc paths; (5) fixes the malformed GWT format in `002_status.md` IT cases. No code changes. Success is verified by: `grep -r 'NNN_\|^EC-' tests/docs/cli/type/ --include='*.md'` returns zero matches, all readme parity checks pass, and `./verb/test` shows no regressions.

## In Scope

- Rename all 52 `NNN_*.md` spec files in `tests/docs/` to `NN_*.md` (all directories except `cli/format/` which is already correct)
- Update all cross-references to renamed files in: all `readme.md` files under `tests/docs/`, `tests/docs/cli/readme.md` Navigation section, and spec file `See [...]` links
- Replace binary name `cm` and `clm` with `clv` in all spec file test case invocations (`When:` fields and test case descriptions)
- Fix type spec prefix: rename all `EC-N` cases to `TC-N` in `tests/docs/cli/type/001_verbosity_level.md` through `007_config_key.md`; verify each type spec has ≥ 4 `TC-` cases
- Fix four dead cross-references: remove/update links to `docs/cli/005_params.md`, `docs/cli/006_types.md`, `docs/cli/001_commands.md`, `docs/cli/003_parameter_groups.md` (these flat files no longer exist; link to the current `docs/cli/param/`, `docs/cli/type/`, `docs/cli/command/` directories instead)
- Fix `tests/docs/cli/command/002_status.md`: clean up malformed `**Expected:**` embedded inside `- **When:**` fields; ensure each case has a proper `- **Then:**` line

## Out of Scope

- Source code changes — no test implementations are modified; only spec doc files
- New test cases — this task does not add coverage, only repairs existing spec hygiene
- `l1_imp_surface.rulebook.md` Surface Mapping and Element Types table updates — those files are in a separate repository (`$PRO/genai/dev/testing/`) and require explicit cross-repo authorization
- Command spec format migration (GWT → `**Command:** / **Expected behavior:**`) — a separate architectural decision; all 13 command specs currently use GWT consistently which satisfies Q111
- `tests/docs/cli/command/001_help.md` source function name `tc080_help_lists_12_commands` — that is in test source code and outside this task's doc-only scope

## Requirements

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   `l1_imp_surface.rulebook.md § Spec : File Naming` — `NN_element_name.md` for ≤ 99 elements
-   `l1_imp_surface.rulebook.md § Spec : Test Case Format` — `TC-` prefix for type specs
-   `l1_imp_surface.rulebook.md § Spec : Behavioral Divergence` — both inputs must be valid values
-   `l1_imp_surface.rulebook.md § Index : Readme Parity` — row count = non-readme file count
-   Privacy invariant: `iron-cage/agent_kit` is PUBLIC — use only committed-safe content

## Work Procedure

Execute in order. Do not skip or reorder steps.
*(Document-only task — no test code changes; steps 2-6 omitted; steps 3-6 are TDD-exempt per task template.)*

1. **Read rulebooks** — `kbase .rulebooks`; confirm `l1_imp_surface.rulebook.md` priority 1 and all applicable constraints.

2. **Rename spec files (NNN_ → NN_)** — For each spec directory, use `git mv` to rename files. Process directories in order: `cli/command/` (001→013 → 01→13), `cli/param/` (001→012 → 01→12), `cli/type/` (001→007 → 01→07), `cli/param_group/` (001→004 → 01→04), `cli/user_story/` (001→005 → 01→05), `feature/` (001→006 → 01→06), `algorithm/` (001→002 → 01→02), `pattern/` (001 → 01), `pitfall/` (001→002 → 01→02), `collection/` (001 → 01). `cli/format/` already uses `NN_` — skip.

3. **Update cross-references** — For each renamed file, update all references:
   - `tests/docs/cli/readme.md` Navigation links (### Commands, ### Parameters, etc.)
   - `tests/docs/cli/command/readme.md`, `cli/param/readme.md`, etc. — Overview Table filenames
   - Any `See [...]` link in spec files that points to a sibling renamed spec
   Confirm: `grep -r '001_\|002_\|003_\|004_\|005_\|006_\|007_\|008_\|009_\|010_\|011_\|012_\|013_' tests/docs/ --include='*.md'` returns zero matches (excluding `docs/` source references and `src/` paths).

4. **Fix binary name (cm/clm → clv)** — `grep -rl '\bcm\b\|clm ' tests/docs/ --include='*.md'` to find all files. For each file, replace occurrences in `- **When:**` fields and test case descriptions only. Do not modify source function names or integration test names in Source Functions tables.

5. **Fix type spec prefix (EC- → TC-)** — For each of `cli/type/01_verbosity_level.md` through `07_config_key.md`:
   - Rename all `EC-N` case headings to `TC-N` (continuing the existing TC-N sequence)
   - Update the Test Case Index table rows to match
   - Verify TC- count ≥ 4 per file; add cases if any file falls below minimum after renaming
   - Update the Behavioral Divergence Pair if it references EC- cases

6. **Fix dead cross-references** — For each spec file containing `005_params.md`, `006_types.md`, `001_commands.md`, or `003_parameter_groups.md` references: update to point to the current `docs/cli/param/`, `docs/cli/type/`, `docs/cli/command/`, or `docs/cli/param_group/` directories respectively (link to the `readme.md` or remove the dead link).

7. **Fix 002_status.md GWT format** — Remove the embedded `**Expected:**` text from inside `- **When:**` blocks in IT-1 through IT-12. Ensure each case has a properly specified `- **Then:**` line with a concrete observable assertion (not just "see spec").

8. **Verify Coverage Gate** — Run Q101–Q112 checks manually on affected directories. Specific checks:
   - Q104: `find tests/docs -name '*.md' | grep -v readme | grep -v procedure | grep '/[0-9]\{3\}_'` → 0 results
   - Q108: For each directory, count rows in Overview Table matches non-readme file count
   - Q107: Every CLI case has an exit code stated
   - Binary name: `grep -r '\bcm\b\|clm ' tests/docs/ --include='*.md' -l` → 0 files
   - Type prefix: `grep -rn '^### EC-\|^| EC-' tests/docs/cli/type/ --include='*.md'` → 0 matches

9. **Run tests** — `./verb/test` (from `module/claude_version/` directory). Confirm all previously passing tests continue to pass; zero new failures.

10. **Update task state** — Set ✅ in `task/readme.md`, recalculate advisability to 0, re-sort, move file to `task/completed/`.

## Test Matrix

*(Verification scenarios for this doc-only task — each row is verified by the step-8 commands, not by automated tests.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Spec file in `cli/command/` | Filename prefix | 2-digit: `01_help.md`, `02_status.md`, …, `13_config.md` |
| T02 | Spec file in `cli/param/` | Filename prefix | 2-digit: `01_version.md`, …, `12_unset.md` |
| T03 | Spec file in `cli/type/` | Filename prefix | 2-digit: `01_verbosity_level.md`, …, `07_config_key.md` |
| T04 | Spec file in `feature/`, `algorithm/`, `pattern/`, `pitfall/`, `collection/` | Filename prefix | 2-digit prefix throughout |
| T05 | Binary name in `- **When:**` field | Name token | `clv`, not `cm` or `clm` |
| T06 | Case heading in `cli/type/*.md` | Case ID prefix | `TC-N` exclusively; no `EC-N` |
| T07 | TC- case count per type spec | Minimum coverage | ≥ 4 per file |
| T08 | Reference to `005_params.md` | Link target | Resolves to existing path (`docs/cli/param/readme.md` or equivalent) |
| T09 | `002_status.md` IT-1 through IT-12 | GWT format | No `**Expected:**` inside `- **When:**` blocks |
| T10 | readme.md Overview Table rows vs file count | Q108 parity | Counts equal for every test surface directory |

## Acceptance Criteria

-   `find tests/docs -name '*.md' | grep -v readme | grep -v procedure | grep '/[0-9]\{3\}_'` → 0 results
-   `grep -rl '\bcm\b\|clm ' tests/docs/ --include='*.md'` (excluding source function names) → 0 files
-   `grep -rn '^### EC-\|^| EC-' tests/docs/cli/type/ --include='*.md'` → 0 matches
-   Every type spec in `cli/type/` has ≥ 4 TC- cases
-   Every readme.md Overview Table row count equals its directory's non-readme file count (Q108 passes everywhere)
-   `./verb/test` passes with zero regressions from the baseline of 420/420

## Validation

**Execution:** An independent validator (not the executor) runs this section after SUBMIT transition.

### Checklist

**Spec file naming (Q104)**
- [ ] C1 — Does `find tests/docs -name '*.md' | grep -v readme | grep -v procedure | grep '/[0-9]\{3\}_'` return zero results?
- [ ] C2 — Does `tests/docs/cli/format/` still use `NN_` prefix (01_text.md, 02_json.md unchanged)?

**Binary name consistency**
- [ ] C3 — Does `grep -rl '\bcm\b' tests/docs/ --include='*.md'` (in When/Then lines only) return zero files?
- [ ] C4 — Does `grep -rl 'clm ' tests/docs/ --include='*.md'` return zero files?

**Type spec prefix (TC- only)**
- [ ] C5 — Does `grep -rn '^### EC-' tests/docs/cli/type/ --include='*.md'` return zero matches?
- [ ] C6 — Does every file in `tests/docs/cli/type/` contain at least 4 lines matching `^### TC-`?

**Cross-references**
- [ ] C7 — Does `grep -rl '005_params\.md\|006_types\.md\|001_commands\.md\|003_parameter_groups\.md' tests/docs/ --include='*.md'` return zero files?

**Readme parity (Q108)**
- [ ] C8 — Do all Overview Table row counts in `tests/docs/*/readme.md` equal the corresponding non-readme file counts?

**No regressions**
- [ ] C9 — Does `./verb/test` pass with zero new failures?

**Out of Scope confirmation**
- [ ] C10 — Are source code files (`tests/integration/*.rs`, `tests/*.rs`) unmodified?

### Measurements

- [ ] M1 — `NNN_` files: `find tests/docs -name '*.md' | grep '/[0-9]\{3\}_' | wc -l` → 0 (was: 52)
- [ ] M2 — Type TC- count: minimum per type spec file → ≥ 4 (was: 2–3 for types 001–002)

### Invariants

- [ ] I1 — test suite: `./verb/test` → 0 new failures (baseline: 420/420)
- [ ] I2 — task system: `task/decisions.md` exists → file present

### Anti-faking checks

- [ ] AF1 — `git diff --stat HEAD` shows file renames in `tests/docs/`; zero changes in `tests/integration/` or `src/`
- [ ] AF2 — `grep -c 'clv' tests/docs/cli/param/01_version.md` → value > 0 (confirms rename + content fix applied)

## Related Documentation

- `l1_imp_surface.rulebook.md` — primary governing spec for all remediation (Coverage Gate Q101–Q112, `§ Spec : File Naming`, `§ Spec : Test Case Format`)
- `tests/docs/cli/command/001_help.md` (→ renamed `01_help.md`) — stale count fixed in this session
- `tests/docs/cli/param/012_unset.md` (→ renamed `12_unset.md`) — behavioral divergence pair fixed in this session
- `tests/docs/cli/type/` (7 files) — type spec prefix violation
- `tests/docs/cli/command/` (13 files) — NNN_→NN_ rename scope
- `tests/docs/cli/param/` (12 files) — NNN_→NN_ rename + binary name fix scope
- `tests/docs/cli/param_group/` (4 files) — NNN_→NN_ rename scope
- `tests/docs/cli/user_story/` (5 files) — NNN_→NN_ rename scope
- `tests/docs/feature/` (6 files) — NNN_→NN_ rename scope
- `tests/docs/algorithm/` (2 files) — NNN_→NN_ rename scope
- `tests/docs/pattern/` (1 file) — NNN_→NN_ rename scope
- `tests/docs/pitfall/` (2 files) — NNN_→NN_ rename scope
- `tests/docs/collection/` (1 file) — NNN_→NN_ rename scope
- `docs/entities.md` — entity instance registry (counts verified, no changes needed)

## Affected Entities

| Entity Path | Type | Why Affected |
|-------------|------|-------------|
| `tests/docs/cli/command/` | test surface | 13 files renamed NNN_→NN_ |
| `tests/docs/cli/param/` | test surface | 12 files renamed + binary name fixed |
| `tests/docs/cli/type/` | test surface | 7 files renamed + EC-→TC- prefix fixed |
| `tests/docs/cli/param_group/` | test surface | 4 files renamed |
| `tests/docs/cli/user_story/` | test surface | 5 files renamed |
| `tests/docs/feature/` | test surface | 6 files renamed |
| `tests/docs/algorithm/` | test surface | 2 files renamed |
| `tests/docs/pattern/` | test surface | 1 file renamed |
| `tests/docs/pitfall/` | test surface | 2 files renamed |
| `tests/docs/collection/` | test surface | 1 file renamed |

## History

- **2026-06-16** `CREATED` — Task filed after test surface audit identified 5 violation categories. Goal: repair all NNN_ naming, binary name, type spec prefix, dead cross-reference, and GWT format violations across tests/docs/.
- **2026-06-16** `VERIFIED` — MAAV Verification Gate passed (4/4 subagents PASS).

## Verification Record

- **Date:** 2026-06-16
- **Validator:** claude-opus-4-6 (MAAV — 4 independent subagents)
- **Result:** PASS (4/4)

| Dimension | Verdict | Key Finding |
|-----------|---------|-------------|
| Scope Coherence | PASS | 6 concrete In Scope items with exact counts; 5 meaningful Out of Scope exclusions; 3 inline verification commands with binary pass/fail outcomes |
| MOST Goal Quality | PASS | Motivated by named audit with stated downstream harm (unreliable Coverage Gate); Observable via filesystem inspection; Scoped with "No code changes" and exact counts; Testable with inline grep/find commands |
| Value / YAGNI | PASS | All 5 violation categories confirmed in codebase: 53 NNN_ files, 20 EC- headings, 4 dead cross-references, cm binary name in dozens of files, GWT malformation in 12 IT cases. No speculative items |
| Implementation Readiness | PASS | 10 executable steps with explicit commands; 10-row Test Matrix covering all 5 categories; 6 binary pass/fail Acceptance Criteria; Validation has all 4 subsections with anti-faking guards |
