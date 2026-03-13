# TSK-009: Update CLI reference docs — `.sessions` summary mode + scope default fix

## Goal

Update `docs/cli/commands.md`, `unilang.commands.yaml`, and `readme.md` so the
`.sessions` reference material accurately documents both the new summary-mode
default invocation (TSK-007) and fixes a pre-existing inconsistency where
`commands.md` lists `scope::` default as `local` instead of `under` — confirmed
when `commands.md` contains a "Default invocation" section describing summary mode
and no file shows `scope::` default as `local` for `.sessions`.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md`
  - Line 18 Commands Table: update `.sessions` purpose from "Session-first listing with
    scope control" to "Active-session summary (default) or scoped session list"
  - Line 399 Parameters table: fix `scope::` default from `local` → `under`
  - Add "**Default invocation (summary mode)**" subsection before **Examples**
    explaining that bare `.sessions` produces the single-session summary block;
    any explicit parameter activates list mode
  - Update **Examples** — add a bare `.sessions` example with summary output annotation;
    re-label existing examples as "list mode (explicit parameter given)"
  - Update **Verbosity output format** section — note that verbosity levels apply to
    list mode; summary mode has its own fixed format independent of verbosity
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/unilang.commands.yaml`
  - Line 456 `description`: update from "List sessions with scope control (session-first view)"
    → "Show active-session summary (default) or list sessions with scope control"
  - Line 457 `hint`: update from "List sessions by scope"
    → "Active session summary or scoped list"
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/readme.md`
  - § `.sessions` (line 270): update opening sentence to mention summary mode as default
  - Update **Output format** bullet for verbosity 1: note verbosity applies to list mode
    only; default bare invocation uses summary format

## Out of Scope

- CLI test docs (`docs/cli/testing/`) — covered in TSK-008
- Implementation changes (`src/cli/mod.rs`, tests) — covered in TSK-007
- Other commands' documentation

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read `commands.md` § Command :: 8. `.sessions`** — note the scope:: default
   inconsistency (shows `local`, should be `under`) and absence of summary mode.
2. **Fix Commands Table row** — update purpose text in the `| 8 |` row.
3. **Fix Parameters table** — change `scope::` Default cell from `local` → `under`.
4. **Add "Default invocation" subsection** — insert after the Parameters table and
   before **Examples**; describe summary-mode output format:
   ```
   Active session  {8-char-id}  {age}  {count} entries
   Project  {rel-path-from-cwd}

   Last message:
     {truncated-text}
   ```
   Note: triggers only when ALL of scope, path, session, agent, min_entries, limit,
   verbosity are omitted. Any explicit parameter → list mode.
5. **Update Examples** — annotate existing bare `.sessions` example as now showing
   summary mode; add `# list mode (explicit scope)` comment to examples with parameters.
6. **Update Verbosity output format note** — prepend a sentence clarifying verbosity
   matrix applies to list mode; summary mode is independent of verbosity.
7. **Update `unilang.commands.yaml`** — replace description and hint strings.
8. **Update `readme.md` § .sessions** — update opening sentence and output format list.
9. **Walk Validation Checklist** — every answer YES.
10. **Update task status** — set ✅, move to `task/completed/`.

## Acceptance Criteria

- `commands.md` `scope::` default is `under`, not `local`
- `commands.md` has a "Default invocation (summary mode)" subsection
- `commands.md` summary format block matches TSK-007 Proposed Output exactly
- `unilang.commands.yaml` description and hint mention "active-session summary"
- `readme.md` opening sentence for `.sessions` mentions summary mode
- No reference doc says bare `.sessions` shows a session list or has scope default `local`

## Validation Checklist

Desired answer for every question is YES.

**commands.md — scope default fix**
- [ ] Does the Parameters table show `scope::` default as `under`?
- [ ] Does `grep "local" commands.md` return zero matches inside the `.sessions` default column?

**commands.md — summary mode section**
- [ ] Is there a "Default invocation" subsection in the `.sessions` command section?
- [ ] Does the subsection include the `Active session  {id}  {age}  {count} entries` header line?
- [ ] Does the subsection state the trigger condition (all params omitted)?
- [ ] Does the subsection state the passthrough rule (any explicit param → list mode)?

**commands.md — verbosity note**
- [ ] Does the Verbosity output section note that it applies to list mode only?

**unilang.commands.yaml**
- [ ] Does the `description` field mention "active-session summary"?
- [ ] Does the `hint` field reflect the dual nature (summary or list)?

**readme.md**
- [ ] Does the `.sessions` opening sentence mention summary mode as default?

**Consistency**
- [ ] No reference doc describes bare `.sessions` as showing a session list?
- [ ] Are all Validation Procedure measurements met?

## Validation Procedure

### Measurements

**M1 — scope default fixed in commands.md**
Command: `grep -n "scope::" docs/cli/commands.md | grep -i "local" | grep -i "default"`
Before: 1 match (line ~399). Expected: 0 matches. Deviation: any match = stale.

**M2 — summary subsection present**
Command: `grep -c "Default invocation" docs/cli/commands.md`
Before: 0. Expected: ≥1. Deviation: 0 = subsection missing.

**M3 — YAML description updated**
Command: `grep "active-session summary" unilang.commands.yaml | wc -l`
Before: 0. Expected: ≥1. Deviation: 0 = not updated.

**M4 — readme updated**
Command: `grep -c "summary" readme.md`
Before: 0 (in `.sessions` section). Expected: ≥1. Deviation: 0 = not updated.

### Anti-faking checks

**AF1 — Summary format matches TSK-007**
Check: manually verify that the summary block in `commands.md` matches the Proposed
Output in `task/007_sessions_active_summary_default.md` exactly (same fields, same
order: session line, Project line, blank line, Last message section).

**AF2 — No stale list description for bare invocation**
Command: `grep -n "Current project sessions only" docs/cli/commands.md`
Expected: zero matches (this comment was for the old list-mode bare invocation example;
it must be updated or replaced with summary-mode annotation).

## Outcomes

Applied directly in the same session as creation. All changes confirmed on disk:
- `docs/cli/commands.md`: Commands Table row 8 purpose updated; opening description updated; `scope::` default fixed `local` → `under` (pre-existing inconsistency); "Default invocation (summary mode)" subsection added with exact output format, truncation rule, and trigger condition; Examples bare invocation comment updated; Verbosity section note prepended.
- `unilang.commands.yaml`: description updated to "Show active-session summary (default) or list sessions with scope control"; hint updated to "Active session summary or scoped list".
- `readme.md`: opening sentence updated to lead with summary mode; Output format section replaced with "Default output (summary mode)" block + "List output" section; example comment updated.
- All Validation Procedure measurements satisfied: M1 stale `local` default eliminated from `.sessions` params table, M2 "Default invocation" subsection present, M3 YAML description updated, M4 readme updated.
