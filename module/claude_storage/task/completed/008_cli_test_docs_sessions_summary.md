# TSK-008: Update CLI test docs — `.sessions` summary mode

## Goal

Update `docs/cli/testing/` so every test touching the bare `.sessions` invocation
reflects the new summary-mode output introduced by TSK-007 — confirmed when IT-1
is fully rewritten and IT-30 through IT-35 are added covering the summary format,
truncation gate, and filter-passthrough contract.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/command/sessions.md`
  - IT-1: full rewrite — bare invocation now shows **summary mode**, not a session list
  - Add IT-30: Summary header format (session ID 8 chars, age, entry count, project path)
  - Add IT-31: Truncation gate — message ≤ 50 chars shown in full (no ellipsis)
  - Add IT-32: Truncation formula — message > 50 chars shown as `{first30}...{last30}`
  - Add IT-33: Empty state — "No active session found." when scope has no sessions
  - Add IT-34: Filter passthrough — explicit `scope::local` keeps list mode
  - Add IT-35: Filter passthrough — explicit `limit::N` keeps list mode
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/param/scope.md`
  - EC-7: Update body to distinguish "scope defaults to under (for filter resolution)" from "bare invocation shows summary mode (different output format from explicit `scope::under`)"

## Out of Scope

- CLI reference docs (`commands.md`, `unilang.commands.yaml`, `readme.md`) — covered in TSK-009
- Implementation changes (`src/cli/mod.rs`, tests) — covered in TSK-007
- Other parameter test files not affected by summary mode

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read current IT-1** (`sessions.md` § IT-1) — note exact fields claimed: Goal, Setup,
   Command, Expected Output, Verification, Pass Criteria.
2. **Rewrite IT-1** — change goal to "Verify that bare `.sessions` shows summary mode
   output (not a session list)"; update Expected Output to match proposed output format
   from TSK-007; update Verification to assert summary header present and list header
   (`Found N sessions:`) absent.
3. **Update IT-1 in Test Case Index** — rename row title from
   "Default (no args) returns under scope sessions" →
   "Default (no args) shows active-session summary".
4. **Update Test Coverage Summary** in `sessions.md` — replace "Default Behavior: 1 test (IT-1)"
   with "Default Behavior / Summary Mode: 6 tests (IT-1, IT-30–IT-35)".
5. **Add IT-30 through IT-35** — one section per TSK-007 Test Matrix row T01–T06
   (T07 is covered by IT-35 limit passthrough). Each section follows the standard
   Goal / Setup / Command / Expected Output / Verification / Pass Criteria pattern.
6. **Add IT-30..IT-35 rows** to the Test Case Index table.
7. **Rewrite EC-7 body** in `scope.md` — keep the Goal (scope defaults to `under` for
   filter resolution) but update Expected Output and Verification to note that bare
   invocation produces **summary mode output**, not the same as `clg .sessions scope::under`
   (which shows list mode); Pass Criteria: "exit 0 + summary header present + `Found N
   sessions:` absent".
8. **Walk Validation Checklist** — every answer YES.
9. **Update task status** — set ✅, move to `task/completed/`.

## Test Matrix (for new IT-30..IT-35)

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| IT-30 | No args, sessions exist | Summary header format | `Active session  {8-char-id}  {age}  {count} entries` + `Project  {rel_path}` present |
| IT-31 | Last message = 40 chars | Truncation gate | Full message shown; no `...` in output |
| IT-32 | Last message = 60 chars | Truncation formula | Output contains `...`; first 30 + last 30 chars match |
| IT-33 | No sessions in scope | Empty state | `No active session found.` on stdout |
| IT-34 | Explicit `scope::local` given | Filter-active detection | `Found N sessions:` header (list mode) |
| IT-35 | Explicit `limit::5` given | Filter-active detection | `Found N sessions:` header (list mode) |

## Acceptance Criteria

- IT-1 goal, Expected Output, and Pass Criteria describe summary mode (not list)
- IT-1 title row in Test Case Index updated
- Test Coverage Summary updated to include IT-30–IT-35
- IT-30..IT-35 each follow standard test-case format (6 sections)
- EC-7 Pass Criteria states "summary header present + `Found N sessions:` absent"
- No `docs/cli/testing/` file asserts that bare `.sessions` produces a session list
- `w3 .test l::3` passes (test docs are read-only; verify no stale references in test code)

## Validation Checklist

Desired answer for every question is YES.

**sessions.md**
- [ ] Is IT-1 rewritten to describe summary mode output?
- [ ] Does IT-1 Expected Output show the `Active session  {id}...` header?
- [ ] Does IT-1 Verification assert `Found N sessions:` is absent?
- [ ] Is IT-1 row in Test Case Index renamed to reflect summary mode?
- [ ] Are IT-30 through IT-35 present as full test case sections?
- [ ] Is Test Coverage Summary updated with "Summary Mode: 6 tests"?

**scope.md**
- [ ] Is EC-7 Expected Output updated (summary header, not list)?
- [ ] Does EC-7 Pass Criteria explicitly mention "summary header present"?
- [ ] Does EC-7 note the distinction between scope filter defaulting to `under` and output mode?

**Consistency**
- [ ] No remaining assertion in `docs/cli/testing/` claims bare `.sessions` shows a session list?
- [ ] Are all Validation Procedure measurements met?

## Validation Procedure

### Measurements

**M1 — IT-1 rewritten**
Command: `grep -c "summary" docs/cli/testing/command/sessions.md`
Before: 0. Expected: ≥2 (goal + expected output). Deviation: 0 = not rewritten.

**M2 — New test cases added**
Command: `grep -c "^### IT-3[0-5]" docs/cli/testing/command/sessions.md`
Before: 0. Expected: 6. Deviation: <6 = incomplete.

**M3 — Old claim eliminated**
Command: `grep -c "under scope sessions" docs/cli/testing/command/sessions.md`
Before: 2 (index row + section header). Expected: 0. Deviation: >0 = stale text remains.

**M4 — EC-7 updated**
Command: `grep -c "summary" docs/cli/testing/param/scope.md`
Before: 0. Expected: ≥1. Deviation: 0 = EC-7 not updated.

### Anti-faking checks

**AF1 — IT-1 not just title-renamed**
Check: `grep "Found N sessions" docs/cli/testing/command/sessions.md | head -5`
Expected: zero matches in IT-1 section (the phrase may appear in IT-34/IT-35 as
expected output for list-mode cases, but NOT in IT-1 Expected Output).

## Outcomes

Applied directly in the same session as creation. All changes confirmed on disk:
- `docs/cli/testing/command/sessions.md`: IT-1 rewritten to summary mode; IT-30..IT-35 added; Test Coverage Summary updated with Summary Mode and Filter Passthrough categories; file header updated to mention summary mode.
- `docs/cli/testing/param/scope.md`: EC-7 fully rewritten — title updated to "under scope (summary mode output)"; Expected Output shows summary block; Verification checks `Active session` present + `Found N sessions:` absent; setup uses child project to confirm under-scope behavior.
- All Validation Procedure measurements satisfied: M1 `grep -c "summary" sessions.md` ≥ 2, M2 new IT-30..IT-35 sections = 6, M3 stale "under scope sessions" text eliminated, M4 EC-7 updated.
