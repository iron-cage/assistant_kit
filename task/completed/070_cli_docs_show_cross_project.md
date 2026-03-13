# TSK-070 — Update CLI docs: `.show` cross-project behavior + Scope Configuration group note

## Status

✅ (Complete)

## Metadata

- **Value:** 5
- **Easiness:** 8
- **Priority:** 0
- **Safety:** 9
- **Advisability:** 0

## Goal

Two CLI documentation gaps remain in `docs/cli/`: `commands.md` does not document that
`.show session_id::ID` (without `project::`) searches globally; `parameter_groups.md` Session
Identification group does not explain the global-search fallback. The Scope Configuration
group threshold note item is superseded — `.show` is now a full member of Group 5 (6 commands
total, threshold note removed during scope expansion work). After this task the two remaining
gaps are closed. Verified by grep checks confirming the new wording is present.

**MOST criteria:**
- **Motivated:** A user who discovers sessions via `.sessions scope::under` will attempt
  `.show session_id::ID` from their working directory. Current CLI docs imply the lookup is
  scoped to the current project, producing silent failure. Accurate docs prevent this confusion
  and set correct expectations before the implementation lands.
- **Observable:** `commands.md` `.show` section gains a global-search note; `parameter_groups.md`
  Session Identification section gains a global-search sentence; Scope Configuration note
  references `.show` as a pending member.
- **Scoped:** Only `docs/cli/commands.md` and `docs/cli/parameter_groups.md` — no `spec.md`
  changes (task 069), no `params.md` structural changes (no new parameters), no code changes.
- **Testable:** `grep` confirms "global" in each targeted section; command count in Commands Table
  is unchanged at 9.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md`
  - In the `.show` command section, update the `session_id::` parameter row description to note
    that when `project::` is absent, all projects are searched globally for the session ID
  - Add (or update) a Notes block in the `.show` section: "When `session_id::` is given without
    `project::`, the storage is searched globally; the first project containing a matching session
    is used. Supply `project::` to restrict lookup to a specific project."

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/parameter_groups.md`
  - In the Session Identification group section: add a sentence to the Purpose or Semantic
    Coherence description explaining that when used without an accompanying `project::` parameter,
    `session_id::` triggers a global search across all projects
  - ~~Scope Configuration threshold note~~ — **superseded**: `.show` was added as a full
    member of Group 5 during scope expansion; no threshold note exists; item complete

## Out of Scope

- `params.md` structural changes — `session_id::` type and description do not change
- `dictionary.md` changes — Session and Session Filter definitions already accurate
- Code implementation of global search (depends on task 069 spec; separate implementation task)
- `commands.md` command count update — behavioral change does not add/remove commands
- Changes to `spec.md` (task 069)

## Description

`parameter_groups.md` Session Identification (group 3) documents `session_id::` as identifying
"a specific session for direct access" — accurate but silent on what project scope is searched.
The Scope Configuration group (group 5) already has a threshold note anticipating future commands
joining the group; that note should be updated to name `.show` as the concrete pending member.

Both changes are documentation-only. They describe intended behavior (the spec contract from
task 069) so users understand the semantics before the implementation is merged. This is the
standard pattern: spec first (069), CLI docs second (070), implementation third.

Related: task 069 (spec.md), task 068 (session path commands — parallel work).

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Non-code task: TDD cycle omitted; Validation Checklist still required per tsk.rulebook.md
-   Unique Responsibility: do not duplicate spec.md wording verbatim; CLI docs describe
    user-facing behavior, spec describes behavioral contract

## Work Procedure

Execute in order. Do not skip or reorder steps.

*(Non-code task — omits TDD steps 2–6 from the standard template.)*

1. **Read rulebooks** — `kbase .rulebooks`; note constraints on CLI docs format
   (cli_design.rulebook.md, documentation.rulebook.md).
2. **Update `commands.md`** — in the `.show` section, update the `session_id::` row description
   and add/update the Notes block with global-search semantics.
3. **Update `parameter_groups.md` Session Identification** — add global-search fallback sentence
   to the group's Purpose description.
4. **Update `parameter_groups.md` Scope Configuration note** — revise the threshold note to name
   `.show` as a pending member once cross-project implementation is complete.
5. **Walk Validation Checklist** — every answer must be YES. A NO blocks delivery.
6. **Update task status** — set 📥 → ✅ in `task/readme.md`, recalculate advisability
   (Priority=0, Advisability=0), re-sort index, move file to `task/completed/`.

## Acceptance Criteria

-   `commands.md` `.show` section documents the global-search fallback for `session_id::` without
    `project::`
-   `commands.md` `.show` section notes that `project::` can be supplied to restrict to one project
-   `parameter_groups.md` Session Identification group documents the global-search semantics
-   `parameter_groups.md` Scope Configuration note references `.show` as a pending member
-   Commands Table total count remains 9 (no commands added or removed)
-   No parameter type, default value, or alias is changed in either file

## Validation Checklist

Desired answer for every question is YES.

**`commands.md` `.show` section**
-   [ ] Does the `session_id::` parameter row description mention global search when `project::` is absent?
-   [ ] Does the `.show` Notes block (or equivalent) state that `project::` restricts search to one project?
-   [ ] Is the Commands Table total still `9 commands`?

**`parameter_groups.md` Session Identification**
-   [ ] Does the Session Identification Purpose or description mention global-search fallback when `session_id::` is used without `project::`?
-   [ ] Is the Semantic Coherence Test question for `session_id::` still present and accurate?

**`parameter_groups.md` Scope Configuration**
-   [x] ~~Threshold note references `.show` as pending member~~ — superseded: `.show` is a full member of Group 5; Overview table shows `6 commands`

**Preservation**
-   [ ] Are all parameter types, defaults, and aliases in both files unchanged?
-   [ ] Are parameter group memberships consistent with the scope expansion (Group 5 = 6 commands)?

## Validation Procedure

### Measurements

**M1 — global-search note in `commands.md` `.show` section**
Baseline: `grep -c "global" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md` → 3 (all in `.status` description or `.sessions` examples, none in `.show` section).
Expected after: ≥4 total, with ≥1 occurrence within the `.show` section.
Deviation (count unchanged at 3): note was not added to `.show`.

**M2 — Session Identification group updated in `parameter_groups.md`**
Baseline: the Session Identification section contains 0 occurrences of "global".
Expected after: ≥1 occurrence of "global" within the Session Identification section.
Deviation: group not updated.
Check: `awk '/## Session Identification/,/^---/' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/parameter_groups.md | grep -c "global"`

**M3 — Scope Configuration membership** *(superseded)*
`.show` is already a full member of Group 5. Overview table shows `6 commands`. No threshold note exists.
Verify: `grep "Scope Configuration" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/parameter_groups.md` → must include `6 commands`.

### Anti-faking checks

**AF1 — command count not inflated in Commands Table**
`grep "Total:" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md` → must still read `9 commands`.
Global-search is a behavioral update, not a new command; the count must be unchanged.

**AF2 — no stale current-project implication in `.show` `session_id::` row**
`awk '/Command :: 3/,/---/' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md | grep -i "current project"` → expect 0 matches in the `session_id::` row description.
Confirms misleading wording was replaced, not left alongside the new text.
