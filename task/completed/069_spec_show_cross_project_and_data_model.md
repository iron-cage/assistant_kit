# TSK-069 — Update spec.md: Entry/API Message data model + `.show` cross-project behavior

## Status

✅ (Complete)

## Metadata

- **Value:** 6
- **Easiness:** 8
- **Priority:** 0
- **Safety:** 8
- **Advisability:** 0

## Goal

`spec.md` has no vocabulary/data model section, no cross-reference to `storage_organization.md`'s
conceptual model, and specifies `session_id:: alone` as searching the current project only — which
contradicts the intended global-search fallback. After this task: (a) `spec.md` contains a
`## data model` section linking to `docs/storage_organization.md#conceptual-model` with Entry
storage-envelope / API-Message duality documented, and (b) the `.show` smart behavior row for
`session_id only` is updated to describe global search across all projects with `project::` as
an optional pin. Verified by `grep` checks confirming all additions are present.

**MOST criteria:**
- **Motivated:** `dictionary.md` and `storage_organization.md` already document the Entry duality
  and four-level hierarchy; `spec.md` is out of sync, and the `.show` cross-project behavior has
  no authoritative specification anywhere, causing implementors to default to current-project-only
  lookup which silently fails for sessions discovered via `.sessions scope::under`.
- **Observable:** `spec.md` gains a `## data model` section, a link to
  `docs/storage_organization.md#conceptual-model`, and an updated `.show` smart behavior row.
- **Scoped:** Only `spec.md` — no code changes, no CLI docs changes (task 070 covers those).
- **Testable:** `grep -c "## data model" spec.md` returns ≥1; `grep` finds `.show` global-search
  wording in the `.show` section; link to `storage_organization.md#conceptual-model` is present.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/spec.md`
  - Add `## data model` section (after `## responsibility`, before `## design principles`):
    - Four-level containment hierarchy: Storage Root → Project → Session → Entry
    - Link to `docs/storage_organization.md#conceptual-model` for diagrams
    - Entry definition: storage envelope fields (`uuid`, `parentUuid`, `timestamp`, `sessionId`,
      `isSidechain`, `cwd`, `gitBranch`) wrapping a `message` payload (Claude API Message:
      `role`, `content`, `model`, `usage`); note that `type` and `message.role` carry the same
      `user`/`assistant` values but belong to different layers (storage envelope vs API payload)
  - Update `.show` smart behavior table:
    - Change `session_id only` row from "Shows that session in current project" to "Searches all
      projects globally for the session; returns first match"
    - Add note: when `project::` is also given, the global search is skipped and only that
      project is checked

## Out of Scope

- Changes to `docs/cli/commands.md`, `parameter_groups.md`, or `params.md` (task 070)
- Code implementation of the global search behavior (separate implementation task)
- Changes to `storage_organization.md` or `dictionary.md` (already updated in prior work)
- Adding vocabulary beyond Entry/API Message duality and the `.show` behavior fix
- New `## vocabulary` section beyond what fits naturally in `## data model`

## Description

`dictionary.md` already defines Entry as "a storage envelope wrapping a `message` field that holds
the Claude API Message payload" and `storage_organization.md` already has a `## conceptual model`
section with four-level hierarchy diagrams. The spec is the authoritative source of truth but
currently lacks both.

Separately, the `.show` smart behavior table specifies `session_id only` → "Shows that session in
current project." The identified gap: when a user runs `.sessions scope::under` and discovers a
session in a child project, the natural follow-up is `.show session_id::ID` from their working
directory. If `.show` only searches the current project, this silently fails. The fix — global
search when no `project::` is given — needs its contract in `spec.md` before implementation.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Non-code task: TDD cycle omitted; Validation Checklist still required per tsk.rulebook.md

## Work Procedure

Execute in order. Do not skip or reorder steps.

*(Non-code task — omits TDD steps 2–6 from the standard template.)*

1. **Read rulebooks** — `kbase .rulebooks`; note constraints on spec format (spec.rulebook.md).
2. **Add `## data model` section** — insert into `spec.md` between `## responsibility` and
   `## design principles`. Include four-level hierarchy, link to
   `docs/storage_organization.md#conceptual-model`, and Entry duality description.
3. **Update `.show` smart behavior** — change `session_id only` row wording; add note about
   `project::` pin. Locate the correct table in the `.show` section.
4. **Walk Validation Checklist** — every answer must be YES. A NO blocks delivery.
5. **Update task status** — set 📥 → ✅ in `task/readme.md`, recalculate advisability
   (Priority=0, Advisability=0), re-sort index, move file to `task/completed/`.

## Acceptance Criteria

-   `spec.md` contains a `## data model` section with the four-level containment hierarchy
-   The data model section contains a link to `docs/storage_organization.md#conceptual-model`
-   The Entry description explains the storage-envelope / API-Message duality (two layers,
    same `user`/`assistant` values in different fields)
-   The `.show` smart behavior row for `session_id only` describes global search
-   A note below the `.show` table explains that `project::` skips global search
-   All existing spec content (design principles, REQ-NNN requirements, command specs) unchanged

## Validation Checklist

Desired answer for every question is YES.

**Data model section**
-   [ ] Does `spec.md` contain `## data model` as a heading?
-   [ ] Does the data model section list all four levels (Storage Root, Project, Session, Entry)?
-   [ ] Does the data model section contain a link to `docs/storage_organization.md#conceptual-model`?
-   [ ] Does the Entry description mention both "storage envelope" and the `message` API payload?
-   [ ] Does the Entry description note that `type` and `message.role` belong to different layers?

**`.show` behavior update**
-   [ ] Does the `.show` smart behavior row for `session_id only` describe a global-search fallback?
-   [ ] Is there a note that `project::` can be given to skip the global search?
-   [ ] Is the stale "current project" wording absent from the `session_id only` row?

**Preservation**
-   [ ] Is all existing spec content (REQ-NNN sections, `.list`, `.search`, etc.) unchanged?

## Validation Procedure

### Measurements

**M1 — `## data model` section presence**
Baseline: `grep -c "## data model" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/spec.md` → 0.
Expected after: ≥1.
Deviation (still 0): section was not added.

**M2 — cross-reference to conceptual-model anchor present**
Baseline: `grep -c "conceptual-model" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/spec.md` → 0 (storage_organization.md appears once already but without the anchor).
Expected after: ≥1 occurrence of `conceptual-model` (the anchor fragment).
Deviation: link missing.

**M3 — `.show` global-search wording present in `.show` section**
Baseline: within the `.show` section, `grep -c "global" spec.md` → 7 total in file but 0 in the `.show` smart behavior area.
Expected after: ≥1 occurrence of "global" within the `.show` smart behavior block.
Deviation: global-search contract not added to `.show`.

### Anti-faking checks

**AF1 — stale "current project" wording removed from `session_id only` row**
`grep -n "session_id only.*current project\|session_id only.*current" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/spec.md` → expect 0 matches.
Confirms the old wording was replaced, not left alongside new text.

**AF2 — data model section is non-trivial**
`grep -A 8 "## data model" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/spec.md` → expect ≥5 non-blank lines after the heading.
Confirms the section has substance, not an empty placeholder.
