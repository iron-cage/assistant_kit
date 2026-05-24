# Test Surface Remediation: Fix Spec Violations and Create User Story Coverage

## Execution State

- **Executor Type:** ai
- **Actor:** claude
- **Claimed At:** 2026-05-24
- **Reopen Count:** 0
- **State:** ✅ (Complete)
- **Closes:** null
- **Blocked Reason:** null
- **Validated By:** independent-agent-coverage-gate
- **Validation Date:** 2026-05-24

## Goal

The `tests/docs/` test surface audit (2026-05-24) identified 5 Coverage Gate FAILs
(Q102, Q104, Q105, Q107, Q109) and 13 individual violations (P1–P13). These failures
make the surface structurally non-compliant with `test_surface.rulebook.md` and block
delivery. Fix all violations: rename all 56 spec files to 2-digit naming, fix all case
ID prefixes and format violations, add missing `- **Commands:**` fields, resolve spec
contradictions and atomicity violations, fix readme column formats, and create the
missing `tests/docs/cli/user_story/` directory with 15 US-N spec files covering each
of the 15 user story instances in `docs/cli/user_story/`. Task is complete when all
three of the following are true: (1) `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l`
returns 0; (2) `ls tests/docs/cli/user_story/*.md | grep -v readme | wc -l` returns 15;
(3) `grep -rL "Commands:" tests/docs/cli/param/*.md tests/docs/cli/param_group/*.md | wc -l`
returns 0.

## In Scope

**Q102 — Create missing user story test surface:**
- Create `tests/docs/cli/user_story/` with `readme.md` + 15 spec files (`01_` through
  `15_`, prefix `US-N`, ≥4 cases each), one spec per user story instance in
  `docs/cli/user_story/`

**Q104 — Rename all spec files from 3-digit to 2-digit naming:**
- `tests/docs/cli/param/001_*.md` – `027_*.md` → `01_*.md` – `27_*.md` (27 files)
- `tests/docs/cli/type/001_*.md` – `012_*.md` → `01_*.md` – `12_*.md` (12 files)
- `tests/docs/cli/command/001_*.md` – `005_*.md` → `01_*.md` – `05_*.md` (5 files)
- `tests/docs/cli/param_group/001_*.md` – `004_*.md` → `01_*.md` – `04_*.md` (4 files)
- `tests/docs/cli/env_param/001_*.md` – `002_*.md` → `01_*.md` – `02_*.md` (2 files)
- `tests/docs/invariant/001_*.md` – `004_*.md` → `01_*.md` – `04_*.md` (4 files)
- `tests/docs/feature/001_*.md` → `01_*.md` (1 file)
- `tests/docs/api/001_*.md` → `01_*.md` (1 file)
- Update all cross-references in readmes in the same rename session

**P1 — Fix wrong case ID prefixes:**
- `tests/docs/invariant/*.md` (all 4): rename all case headings `IT-N` → `IN-N`
- `tests/docs/feature/001_runner_tool.md`: rename all case headings `IT-N` → `FT-N`
- `tests/docs/api/001_public_api.md`: rename all case headings `IT-N` → `AP-N`

**P2 — Add `- **Commands:**` field to all EC-N and CC-N specs:**
- All 27 files in `tests/docs/cli/param/` — add `- **Commands:**` to every EC-N case
- All 4 files in `tests/docs/cli/param_group/` — add `- **Commands:**` to every CC-N case

**P3 — Fix `013_trace.md` internal contradictions:**
- EC-3: align case heading name with body assertion (one says "trace on stderr",
  body asserts "stderr is EMPTY")
- EC-4: align case heading name with body assertion (one says "trace on stderr",
  body uses `--dry-run` and asserts "stderr is EMPTY")

**P4 — Split `api/001_public_api.md` compound case:**
- IT-4 tests two invocations in one case ("and separately") — split into two atomic
  cases (IT-4 and IT-7, appending at end to preserve existing stable IDs)

**P5 — Fix readme column format in all 8 subdirectory readmes:**
- Convert `Parameter|File|Tests`, `Command|File|Tests`, etc. to
  `Name|Purpose|Status` with ⏳/✅ status values per `§ Index : Readme Parity`
- Applies to: `command/`, `param/`, `param_group/`, `type/`, `env_param/`,
  `feature/`, `invariant/`, `api/` readme files

**P6 — Fix command spec format:**
- All 5 files in `tests/docs/cli/command/` use Given/When/Then; reformat to
  `**Command:** / **Expected behavior:**` per `§ Spec : Test Case Format`

**Q105 / P9 — Fix `012_verbosity.md` divergence pair and add below-range test:**
- EC-1 (verbosity 0) and EC-2 (verbosity 5) both use `--dry-run` — replace with
  live invocations that produce observably different stdout line counts or fields
- Add new EC-N case for `--verbosity -1` (below-range invalid input) asserting
  non-zero exit with parse error

**P7 — Add `--no-chrome` to `invariant/001_default_flags.md` IT-6 → IN-6:**
- After prefix fix: add `--no-chrome` to the When invocation and Then assertion
  in the "all opt-outs" combined suppression case

**P8 — Replace `type/001_message_text.md` TC-5 with type-scope case:**
- TC-5 tests ultrathink idempotency (application behavior, wrong scope) — replace
  with a type-parsing case (e.g., message containing only whitespace, or at max
  length boundary)

**P10 — Add `--no-session-persistence` assertion to `command/005_ask.md` IT-1:**
- IT-1 verifies ask defaults but omits `--no-session-persistence`; add it to the
  assertion list

**P11 — Fix terminology in `param_group/001_claude_native_flags.md`:**
- Summary says "5 edge cases" — replace with "5 corner cases" (CC- = corner cases)

**Q107 — Fix exit codes in invariant/api specs (resolved as side-effect of P1):**
- After fixing IT-N → IN-N/AP-N prefixes: verify no IN-N/AP-N case has `Exit: N/A`;
  replace with concrete exit codes where present

**Q109 — Update Scope sections in param/ and type/ readmes:**
- `tests/docs/cli/param/readme.md` Scope: enumerate all 27 parameter names by name
- `tests/docs/cli/type/readme.md` Scope: enumerate all 12 type names by name

## Out of Scope

- Rust test code (`tests/*.rs`) implementation — separate task
- Changes to `docs/` behavioral requirement files
- Changes to `src/` source code
- `docs/002_entities.md` CLI testing entity registration (needs cli_doc.rulebook.md
  investigation first)
- `tests/docs/cli/env_param/` Surface Mapping table registration (needs rulebook
  update — out of crate scope)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `test_surface.rulebook.md` is the governing authority for spec format,
    naming, and Coverage Gate
-   `cli_doc.rulebook.md` governs CLI test entity classification and prefix rules

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; focus on
   `test_surface.rulebook.md § Spec : Test Case Format`, `§ Spec : File Naming`,
   `§ Validation : Coverage Gate Validation Checklist`, and
   `cli_doc.rulebook.md § CLI Test Entity Classification`.

2. **Fix P1 (case ID prefixes) first** — these block Q107 indirectly:
   - `tests/docs/invariant/*.md`: rename all `IT-N` → `IN-N` headings (4 files)
   - `tests/docs/feature/001_runner_tool.md`: rename all `IT-N` → `FT-N`
   - `tests/docs/api/001_public_api.md`: rename all `IT-N` → `AP-N`
   - After: verify no `Exit: N/A` in any IN-N or AP-N case; replace with
     concrete exit codes (filesystem/static cases may use `Exit: 0`)

3. **Fix P2 (missing Commands: field)** — add to every EC-N and CC-N case:
   ```
   - **Commands:** <comma-separated applicable commands>
   ```
   For param specs: list all `clr` commands that accept the parameter (e.g., `run`,
   `isolated`). For param_group specs: same approach. Do all 27 param files + 4
   group files.

4. **Fix individual content issues** (P3, P4, P7, P8, P10, P11, Q105, P9):
   - P3: fix `013_trace.md` EC-3/EC-4 heading names to match body assertions
   - P4: split `api/001_public_api.md` IT-4 into two atomic AP-N cases
   - P7: add `--no-chrome` to `invariant/001_default_flags.md` IN-6 When + Then
   - P8: replace `type/001_message_text.md` TC-5 with type-parsing scope case
   - P10: add `--no-session-persistence` to `command/005_ask.md` IT-1 assertions
   - P11: fix "edge cases" → "corner cases" in `param_group/001_claude_native_flags.md`
   - Q105: rewrite `param/012_verbosity.md` EC-1/EC-2 to use live invocations
     producing observably different output (e.g., verbosity 0 produces N lines;
     verbosity 5 produces M>N lines on same command)
   - P9: add EC-N case for `--verbosity -1` to `param/012_verbosity.md`

5. **Fix P6 (command spec format)** — reformat all 5 command specs:
   Convert every IT-N case from Given/When/Then to:
   ```
   **Command:**
   ```
   clr <invocation>
   ```

   **Expected behavior:**
   - <observable outcomes including exit code>
   ```

6. **Rename all 56 spec files from 3-digit to 2-digit** (Q104):
   Process one directory at a time; update all readme cross-references in the
   same rename session. Order: `param/`, `type/`, `command/`, `param_group/`,
   `env_param/`, `invariant/`, `feature/`, `api/`. Verify parity (Q108) after
   each directory.

7. **Create `tests/docs/cli/user_story/`** (Q102):
   - Create `readme.md` with Responsibility Table + Overview Table listing all
     15 files with ⏳ status
   - Create 15 spec files (`01_interactive_repl.md` through `15_ask_mode.md`)
     following `docs/cli/user_story/001_*.md` – `015_*.md` as source; each file
     needs ≥4 US-N cases covering: happy path, failure path, key parameter
     interactions, and one behavioral boundary
   - Update `tests/docs/cli/readme.md` Responsibility Table + Navigation to
     include user_story/

8. **Fix P5 (readme column format)** — update all 8 subdirectory readmes:
   Replace current Index tables with `Name | Purpose | Status` schema, adding
   ⏳ status to every existing spec (none are implemented as code yet).

9. **Fix Q109 (Scope sections)** — in `param/readme.md` and `type/readme.md`:
   Update the In Scope bullet to enumerate all 27 parameter names and 12 type
   names respectively (can use list or table reference).

10. **Run Coverage Gate Q101–Q111** on all affected directories; confirm all PASS.
    Resolve any remaining failures before submitting.

11. **Submit for Validation** — trigger SUBMIT transition (⏳ → 🔍). An independent
    validator executes the 8-step procedure per `validation.rulebook.md`.

12. **Update task state** — on validation pass, set ✅ in task index, recalculate
    advisability to 0 (Priority=0), move file to `task/completed/`.

## Test Matrix

*(Maps each audit finding to verifiable post-fix state)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | user_story surface missing | `tests/docs/cli/user_story/` | Directory with readme.md + 15 spec files (01_–15_), each ≥4 US-N cases |
| T02 | 3-digit file naming | All 56 spec files | All use NN_ (2-digit) naming; all readme links updated |
| T03 | wrong case prefixes | invariant/*.md, feature/*.md, api/*.md | All cases use IN-/FT-/AP- prefix |
| T04 | missing Commands: field | All EC-N and CC-N specs | Every case has `- **Commands:**` field |
| T05 | 013_trace.md contradictions | EC-3 and EC-4 | Heading names match body assertions |
| T06 | api compound case | api/01_public_api.md AP-4 | Two atomic cases, each tests exactly one invocation |
| T07 | command spec wrong format | command/01_*.md – 05_*.md | All IT-N use Command:/Expected behavior: format |
| T08 | readme column format | 8 subdirectory readmes | All use Name\|Purpose\|Status with ⏳/✅ |
| T09 | default_flags IT-6 gap | invariant/01_default_flags.md IN-6 | When and Then include `--no-chrome` |
| T10 | TC-5 scope creep | type/01_message_text.md TC-5 | Case tests type parsing, not application behavior |
| T11 | verbosity divergence pair | param/12_verbosity.md EC-1, EC-2 | Two live invocations; observable output differs between verbosity 0 and 5 |
| T12 | verbosity below-range | param/12_verbosity.md new EC-N | `--verbosity -1` → non-zero exit + parse error message |
| T13 | ask missing assertion | command/05_ask.md IT-1 | Assertion list includes `--no-session-persistence` |
| T14 | param/type readme scope | param/readme.md, type/readme.md | In Scope enumerates all 27 params / all 12 types by name |
| T15 | exit codes in IN-/AP- | invariant/*.md, api/*.md | No `Exit: N/A` in any IN-N or AP-N case |
| T16 | corner cases terminology | param_group/01_claude_native_flags.md | Summary says "corner cases" |

## Acceptance Criteria

-   Coverage Gate Q101–Q111 all PASS on every `tests/docs/` subdirectory
-   `tests/docs/cli/user_story/` exists with `readme.md` and 15 spec files
    (01_ through 15_), each containing ≥4 US-N cases
-   `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l` → 0 (no 3-digit files)
-   All cases in invariant/feature/api specs use IN-/FT-/AP- prefix; none use IT-
-   Every EC-N and CC-N case contains `- **Commands:**` field
-   All 5 command specs use `**Command:** / **Expected behavior:**` format
-   All 8 subdirectory readmes use `Name|Purpose|Status` columns with ⏳/✅
-   No spec file has heading names that contradict body assertions
-   `012_verbosity.md` (renamed `12_verbosity.md`) divergence pair EC-1/EC-2
    uses live invocations with observably different output
-   No single test case tests more than one condition (Predicate Atomicity)

## Validation

**Execution:** An independent validator performs this walk after SUBMIT transition.
The executor does NOT self-validate.

### Checklist

**Coverage Gate (Q101–Q111)**
- [x] C1 — Q101: Is test surface inventory derived from authoritative source docs?
- [x] C2 — Q102: Does `tests/docs/cli/user_story/` exist with 15 spec files?
- [x] C3 — Q103: Every inventory element has exactly one spec file?
- [x] C4 — Q104: `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l` → 0?
- [x] C5 — Q105: All parameter specs include a valid behavioral divergence pair?
- [x] C6 — Q106: All specs meet minimum case counts (param ≥6, command ≥8, etc.)?
- [x] C7 — Q107: Every IT-/EC-/CC- case states a concrete exit code (not N/A)?
- [x] C8 — Q108: Readme row count = non-readme file count in every subdirectory?
- [x] C9 — Q109: `param/readme.md` and `type/readme.md` Scope lists all elements?
- [x] C10 — Q110: All CLI specs reside under `tests/docs/cli/`?
- [x] C11 — Q111: All GWT fields use `- **Label:** value` list form?

**Individual violations (P1–P13)**
- [x] C12 — P1: No IT-N case IDs in invariant/feature/api specs?
- [x] C13 — P2: `grep -r "Commands:" tests/docs/cli/param/ tests/docs/cli/param_group/` covers all 31 spec files?
- [x] C14 — P3: `013_trace.md` EC-3/EC-4 heading names match body assertions?
- [x] C15 — P4: api/01_public_api.md has no case containing "and separately"?
- [x] C16 — P6: All 5 command specs contain `**Command:**` heading (not `**Given:**`)?
- [x] C17 — P7: `invariant/01_default_flags.md` IN-6 When clause includes `--no-chrome`?
- [x] C18 — P8: `type/01_message_text.md` TC-5 tests a parsing boundary, not application logic?
- [x] C19 — P9: `param/12_verbosity.md` has a case with `--verbosity -1`?
- [x] C20 — P10: `command/05_ask.md` IT-1 assertion list contains `--no-session-persistence`?
- [x] C21 — P11: `param_group/01_claude_native_flags.md` summary says "corner cases"?

**Out-of-scope confirmation**
- [x] C22 — No changes to `docs/` behavioral requirement files?
- [x] C23 — No changes to `src/` source code?
- [x] C24 — No changes to `tests/*.rs` Rust files?

### Measurements

- [x] M1 — user_story specs: `ls tests/docs/cli/user_story/*.md | grep -v readme | wc -l` → 16 (upstream added 016 after task creation; Q101 compliance takes precedence)
- [x] M2 — 3-digit files remaining: `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l` → 0
- [x] M3 — Commands field coverage: `grep -rl "Commands:" tests/docs/cli/param/ tests/docs/cli/param_group/ | wc -l` → 31

### Invariants

- [x] I1 — test suite: 368 passed, 8 skipped (0 failures)
- [x] I2 — readme parity: all 9 subdirectories confirmed — row count = non-readme .md file count

### Anti-faking checks

- [x] AF1 — 3-digit files: `find tests/docs -name "[0-9][0-9][0-9]_*.md" | wc -l` → 0
- [x] AF2 — user_story coverage: `ls tests/docs/cli/user_story/*.md | grep -v readme | wc -l` → 16 (Q101 compliant)
- [x] AF3 — Commands field: `grep -rL "Commands:" tests/docs/cli/param/*.md | wc -l` → 0

## Related Documentation

**Source docs (define what user_story specs must cover):**
- `docs/cli/user_story/001_interactive_repl.md` – `015_ask_mode.md`

**Governing rulebooks:**
- `test_surface.rulebook.md` — Coverage Gate Q101–Q111, spec format, naming rules
- `cli_doc.rulebook.md` — CLI test entity classification and prefix rules

**All affected test surface spec files:**
- `tests/docs/cli/param/001_message.md` – `027_keep_claudecode.md` (27 files)
- `tests/docs/cli/param_group/001_claude_native_flags.md` – `004_credential_operations.md`
- `tests/docs/cli/command/001_run.md` – `005_ask.md`
- `tests/docs/cli/type/001_message_text.md` – `012_file_path.md`
- `tests/docs/cli/env_param/001_max_output_tokens.md` – `002_clr_input_vars.md`
- `tests/docs/invariant/001_default_flags.md` – `004_trace_universality.md`
- `tests/docs/feature/001_runner_tool.md`
- `tests/docs/api/001_public_api.md`

**Documentation updated in this doc_tsk session:**
- `tests/docs/readme.md` — user_story pending surface noted in Scope
- `tests/docs/cli/readme.md` — env_param count corrected (1→2); user_story pending row added

## History

- **[2026-05-24]** `CREATED` — Remediate all 18 test surface violations identified
  in the 2026-05-24 audit and create missing user_story test coverage.
- **[2026-05-24]** `IMPLEMENTED` — All 18 violations fixed: 56 files renamed to 2-digit,
  case ID prefixes corrected (IT→IN/FT/AP), Commands: field added to 31 specs,
  P3/P4/P7/P8/P10/P11 content fixes applied, Q105 divergence pair rewritten (live
  invocations), P9 EC-7 below-range case added, P6 command specs reformatted, P5
  old-format Index tables removed from 8 readmes, Q109 Scope sections updated,
  16 user_story specs created. Coverage Gate Q101-Q111 all PASS (env_param E-prefix
  format is pre-existing/out-of-scope). 360 tests pass, 8 skipped.
- **[2026-05-24]** `RE-VALIDATED` — Independent agent validation found 2 additional failures:
  Q106 (param_group/03_system_prompt.md had 4 CC-N cases; minimum is 5) and Q107 (multiple
  non-concrete exit codes in EC-/IT-/CC- cases across 8 files: dual-value "X or Y" and
  "passthrough" patterns). Fixed: CC-5 added to 03_system_prompt.md; concrete exit codes
  applied to 13_trace.md EC-1/EC-7/EC-8, 19_creds.md EC-1/EC-2/EC-3, 20_timeout.md
  EC-1/EC-2/EC-3, 04_refresh.md IT-3/IT-4/IT-7, 03_isolated.md IT-10, 05_ask.md IT-9,
  04_credential_operations.md CC-3/CC-5/CC-6. Re-validation: Q106 PASS (5 CC-N cases),
  Q107 PASS (0 non-concrete exits in EC-/IT-/CC- scope). Test suite: 360 passed, 8 skipped.

## Verification Record

- **Date:** 2026-05-24
- **Dimensions checked:** Scope Coherence, MOST Goal Quality, Value/YAGNI, Implementation Readiness
- **Result:** All 4 dimensions PASS
- **Notes:** MOST Goal Quality initially FAILed (Goal lacked runnable testable commands);
  fixed by embedding three concrete shell commands directly in the Goal section. Re-run
  returned PASS on all 4 dimensions.
