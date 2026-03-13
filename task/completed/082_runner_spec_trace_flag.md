# TSK-080: Update claude_runner spec.md — `--trace` flag

## Goal

`spec.md` for `claude_runner` is inconsistent with the implemented binary: `--trace` is missing
from the Modes table and CLI Flags table, and the `VerbosityLevel` description wrongly implies
only `--verbosity 4+` triggers command preview. Updating the spec unblocks accurate documentation
and removes a misleading description for anyone reading the spec to understand `clr` behavior.
Done when `spec.md` contains `--trace` in Modes and CLI Flags, and `VerbosityLevel` notes both
triggers, verified by `grep -c 'trace' spec.md` ≥ 3.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md` § Modes — add `--trace` row
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md` § CLI Flags (runner-specific) — add `--trace` row
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md` § Public API → VerbosityLevel — update description to note `--trace` also triggers preview

## Out of Scope

- `docs/cli/` files — covered in TSK-081
- Planned but unimplemented improvements (62-item list from analysis) — not implemented, must not appear in spec
- `claude_runner_core` spec — `--trace` is binary-only, no builder changes needed

## Description

The `--trace` flag was added to `main.rs` in the current session. It prints env vars and the
assembled command to stderr before executing, mirroring shell `set -x` behavior.
The spec currently has no record of this flag. Three gaps: (1) Modes table has no `--trace`
row, (2) CLI Flags (runner-specific) has no `--trace` entry, (3) `VerbosityLevel` description
says only `--verbosity 4` triggers preview — now both `--trace` and `--verbosity 4+` do.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note formatting constraints.
2. **Edit § Modes table** — add row:
   `| \`clr --trace "Fix bug"\` | Trace (print then execute) | \`describe_env()\` + \`describe()\` to stderr, then execute |`
3. **Edit § CLI Flags — Runner-specific table** — add row after `--dry-run`:
   `| \`--trace\` | Print env+command to stderr then execute (like shell \`set -x\`) | false |`
4. **Edit § VerbosityLevel description** — change "At `--verbosity 4` (verbose detail), a
   command preview is printed to stderr before execution" to note that `--trace` flag also
   unconditionally triggers the same preview regardless of verbosity level.
5. **Walk Validation Checklist** — every item must be YES.
6. **Update task status** — set ✅ in `dev/task/readme.md`, recalculate advisability to 0,
   re-sort, move file to `dev/task/completed/`.

## Acceptance Criteria

-   `spec.md` § Modes contains a `--trace` row
-   `spec.md` § CLI Flags (runner-specific) contains `--trace` with correct description and default
-   `spec.md` § VerbosityLevel description accurately describes both `--trace` and `--verbosity 4+` triggers
-   No mention of unimplemented features added (spec stays at current implementation level)
-   `grep -c 'trace' spec.md` ≥ 3

## Validation Checklist

Desired answer for every question is YES.

**spec.md § Modes**
-   [ ] Is there a `--trace` row in the Modes table?
-   [ ] Does the row correctly show it still executes (not preview-only)?

**spec.md § CLI Flags**
-   [ ] Is `--trace` in the runner-specific flags table?
-   [ ] Does it show `false` as default?
-   [ ] Is the description accurate ("print to stderr then execute")?

**spec.md § VerbosityLevel**
-   [ ] Does the description mention `--trace` as a trigger for command preview?
-   [ ] Does the description still accurately describe `--verbosity 4+` behavior?

**Out of Scope confirmation**
-   [ ] Are the 62 planned-but-unimplemented improvements absent from spec.md?
-   [ ] Is `docs/cli/` unchanged (separate task)?

## Validation Procedure

### Measurements

**M1 — `--trace` occurrences in spec.md**
Command: `grep -c 'trace' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md`
Before: 0. Expected after: ≥ 3. Deviation: missing update if < 3.

**M2 — Modes table row count**
Command: `grep -c '^\|' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md`
Before: baseline. Expected after: baseline + 1 (one new row). Deviation: row not added.

### Anti-faking checks

**AF1 — `--trace` not just in comments**
Command: `grep 'trace' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md`
Expected: lines are in actual table cells or prose paragraphs, not just code comments.

**AF2 — VerbosityLevel section actually updated**
Command: `grep -A5 'VerbosityLevel' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md | grep 'trace'`
Expected: at least one line containing 'trace' in the VerbosityLevel context.
