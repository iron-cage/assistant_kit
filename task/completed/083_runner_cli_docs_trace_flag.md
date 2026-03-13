# TSK-081: Update claude_runner docs/cli/ — `--trace` flag + stale content fixes

## Goal

`docs/cli/` for `claude_runner` is inconsistent with the implemented binary: `--trace` is missing
from six reference files, the `--interactive` pending note is stale (implemented in TSK-044),
and the `workflows.md` header reports "6 total" while listing 7. Updating all six files
unblocks accurate user-facing documentation and removes misleading references.
Done when `--trace` appears in all affected tables, counts are corrected, and the stale note
is removed, verified by `grep -c 'trace' docs/cli/commands.md` ≥ 2.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/readme.md`
  § Navigation — fix "12 parameters" → 13; remove stale `--interactive` pending implementation note
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/commands.md`
  § All Commands table — fix `run` param count 12 → 13
  § Command :: 1 `run` Parameters table — add `--trace` row after `--dry-run`
  § Command :: 1 `run` Execution Modes table — add `--trace` row
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
  § All Parameters heading — fix "12 total" → 13
  § All Parameters summary table — add row 13 for `--trace`
  § Groups note — fix "Parameters 5–12" → "5–13"
  § Quick Reference footer — fix "run = 12 parameters" → 13
  § Body — add Parameter :: 13 section for `--trace`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/parameter_groups.md`
  § All Groups table — fix Runner Control count 8 → 9
  § Group :: 2 Runner Control coherence test — fix "YES for all 8" → 9
  § Group :: 2 Runner Control Parameters table — add `--trace` row
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/workflows.md`
  § All Workflows header — fix "6 total" → 7 (pre-existing off-by-one; 7 are already listed)
  § Body — add Workflow :: 8 for `--trace` (trace execution)

## Out of Scope

- `spec.md` — covered in TSK-080
- `docs/cli/types.md` — `VerbosityLevel` docs are accurate; no `--trace` changes needed
- `docs/cli/dictionary.md` — no `--trace` vocabulary entry required
- Planned-but-unimplemented improvements from the 62-item analysis — must not appear

## Description

The `--trace` flag was added to `main.rs` in TSK-080's preceding implementation session. It prints
env vars and the assembled command to stderr before executing — distinct from `--dry-run` which
shows the command and exits. Six `docs/cli/` files need updates: parameter counts are off by one
in every count reference, `--trace` is absent from two tables in `commands.md`, the summary table
and body in `params.md`, the Runner Control group in `parameter_groups.md`, and there is no
workflow example. Additionally `readme.md` has two pre-existing staleness issues: the "12 parameters"
count in Navigation and the "Implementation Status: `--interactive` flag pending code implementation"
note (implemented in TSK-044, 2026-03-28). `workflows.md` has a pre-existing "6 total" header bug
(7 workflows are already listed); that header must be corrected to "7 total" before adding
Workflow 8.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note CLI documentation constraints.
2. **Edit `readme.md`** — fix Navigation line "12 parameters" → 13; remove the "Implementation
   Status" line that says `--interactive` is pending (it was implemented in TSK-044).
3. **Edit `commands.md`** —
   a. All Commands table: change `run` Params cell `12` → `13`.
   b. Command :: 1 Parameters table: add row after `--dry-run`:
      `| [\`--trace\`](params.md#parameter--13---trace) | bool | false | Print env+command to stderr then execute (like \`set -x\`) |`
   c. Execution Modes table: add row after dry-run row:
      `| \`clr --trace "Fix bug"\` | Trace (print then execute) | \`describe_env()\` + \`describe()\` to stderr, then execute |`
4. **Edit `params.md`** —
   a. Heading: "12 total" → 13.
   b. Summary table: add row `| 13 | \`--trace\` | bool | false | present/absent | Print env+command to stderr then execute | 1 cmd |`
   c. Groups note: "Parameters 5–12" → "Parameters 5–13".
   d. Quick Reference footer: "run = 12 parameters" → "run = 13 parameters".
   e. Body: add Parameter :: 13 section for `--trace` after Parameter :: 12.
5. **Edit `parameter_groups.md`** —
   a. All Groups table: Runner Control row `8` → `9`.
   b. Group :: 2 coherence test line: "YES for all 8" → "YES for all 9".
   c. Group :: 2 Parameters table: add `--trace` row after `--dry-run`:
      `| [\`--trace\`](params.md#parameter--13---trace) | bool | Print env+command to stderr then execute |`
6. **Edit `workflows.md`** —
   a. All Workflows header: "6 total" → "7 total".
   b. Append Workflow :: 8 section for trace execution (show command to stderr then execute).
7. **Walk Validation Checklist** — every item must be YES.
8. **Update task status** — set ✅ in `dev/task/readme.md`, recalculate advisability to 0,
   re-sort, move file to `dev/task/completed/`.

## Acceptance Criteria

-   `docs/cli/readme.md` shows 13 parameters in Navigation; stale `--interactive` pending note absent
-   `docs/cli/commands.md` `run` command shows 13 params; `--trace` in Parameters table and Modes table
-   `docs/cli/params.md` shows 13 total; `--trace` row 13 in summary table and body section; footer updated
-   `docs/cli/parameter_groups.md` Runner Control shows 9 parameters; `--trace` in table and coherence test
-   `docs/cli/workflows.md` header shows "7 total"; Workflow 8 for `--trace` added
-   No mention of unimplemented features added
-   `grep -c 'trace' docs/cli/commands.md` ≥ 2

## Validation Checklist

Desired answer for every question is YES.

**readme.md**
-   [ ] Is the Navigation "parameters" count updated to 13?
-   [ ] Is the stale `--interactive` pending implementation note removed?

**commands.md**
-   [ ] Does the All Commands table show `run` with 13 params?
-   [ ] Is `--trace` in the Command :: 1 Parameters table?
-   [ ] Is `--trace` in the Command :: 1 Execution Modes table?
-   [ ] Does `--trace` Execution Modes row correctly say it executes (not preview-only)?

**params.md**
-   [ ] Does the heading say "13 total"?
-   [ ] Is `--trace` row 13 in the summary table?
-   [ ] Does the Groups note say "5–13"?
-   [ ] Does the Quick Reference footer say "run = 13 parameters"?
-   [ ] Is there a Parameter :: 13 body section for `--trace`?

**parameter_groups.md**
-   [ ] Does the All Groups table show Runner Control with 9 parameters?
-   [ ] Does the coherence test say "YES for all 9"?
-   [ ] Is `--trace` in the Runner Control Parameters table?

**workflows.md**
-   [ ] Does the All Workflows header say "7 total"?
-   [ ] Is Workflow :: 8 for trace execution present?

**Out of Scope confirmation**
-   [ ] Is `spec.md` unchanged (separate task TSK-080)?
-   [ ] Are the 62 planned-but-unimplemented improvements absent from all docs/cli/ files?

## Validation Procedure

### Measurements

**M1 — `--trace` occurrences in commands.md**
Command: `grep -c 'trace' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/commands.md`
Before: 0. Expected after: ≥ 2. Deviation: missing update if < 2.

**M2 — `--trace` occurrences in params.md**
Command: `grep -c 'trace' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
Before: 0. Expected after: ≥ 3. Deviation: missing rows if < 3.

**M3 — `--trace` occurrences in parameter_groups.md**
Command: `grep -c 'trace' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/parameter_groups.md`
Before: 0. Expected after: ≥ 2. Deviation: missing update if < 2.

**M4 — `--trace` occurrences in workflows.md**
Command: `grep -c 'trace' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/workflows.md`
Before: 0. Expected after: ≥ 2. Deviation: workflow not added if < 2.

**M5 — Parameter count in params.md heading**
Command: `grep 'All Parameters' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
Before: "12 total". Expected after: "13 total". Deviation: count not updated.

**M6 — Workflow count header in workflows.md**
Command: `grep 'All Workflows' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/workflows.md`
Before: "6 total". Expected after: "7 total". Deviation: header not fixed.

### Anti-faking checks

**AF1 — `--trace` not just in comments**
Command: `grep 'trace' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/commands.md`
Expected: lines are in actual table cells or prose paragraphs, not just code comments.

**AF2 — stale note actually removed from readme.md**
Command: `grep 'pending' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/readme.md`
Expected: zero matches (note removed).

**AF3 — parameter count in readme.md Navigation updated**
Command: `grep 'parameters' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/readme.md`
Expected: "13 parameters" (not "12 parameters").

**AF4 — Runner Control coherence test updated in parameter_groups.md**
Command: `grep -A2 'Coherence test' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/parameter_groups.md | grep 'YES for all'`
Expected: "YES for all 9" (not "YES for all 8").
