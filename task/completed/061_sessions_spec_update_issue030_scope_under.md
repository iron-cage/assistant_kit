# TSK-061: Update spec.md — issue-030 path display and scope::under encoding constraint

## Goal

Update `claude_storage/spec.md` § .sessions to accurately describe the
issue-030 path display fix (hyphen-prefixed topic dirs shown when they exist
on disk) and document the scope::under encoding ambiguity constraint.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/spec.md`
  § `.sessions → Path display invariant` (line ~667) — extend to state that
  the displayed path includes any `--topic` component that exists as a real
  hyphen-prefixed directory on disk (e.g., `src/-default_topic`)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/spec.md`
  § `.sessions → Implementation note (issue-024)` (line ~651) — add companion
  note for issue-030 fix and document scope::under known encoding constraint
  (sibling modules with underscore names may match until TSK-060 is resolved)

## Out of Scope

- Code changes (covered in TSK-060)
- CLI docs updates (covered in TSK-062)
- Changes outside spec.md

## Work Procedure

1. `spec.md` § `Path display invariant` — replace current text with:
   - Old: "the project path is always shown as a `~/path/to/project:` header"
   - New: extend to clarify the path is the longest real filesystem prefix —
     the base path PLUS any `--topic` component that exists as a directory
     (e.g., `src/-default_topic` when that directory is on disk); if the topic
     dir does not exist (e.g., `--commit`), only the base is shown; add
     `(fixed: issue-030)` reference

2. `spec.md` § `Implementation note (issue-024)` — append:
   - New note: `(issue-030)` — `decode_project_display` now tries to extend
     the decoded base path with each `--topic` component as a real filesystem
     directory; the longest existing path is used as the display header
   - Known constraint: `scope::under` encoded-prefix matching cannot distinguish
     sibling directories with underscore names (e.g., `claude_storage_core`
     matches when base is `claude_storage`) — tracked in issue-031, fixed by TSK-060

## Validation List

Desired answer for every question is YES.

- [x] Does spec.md § Path display invariant mention topic dirs included when they exist on disk?
- [x] Does spec.md reference issue-030?
- [x] Does spec.md document the scope::under encoding constraint for sibling modules?
- [x] Is the existing issue-024 note preserved and correct?
- [x] Are all Validation Procedure measurements met?

## Validation Procedure

### Measurements

**M1 — issue-030 referenced**
Command: `grep -c "issue-030" module/claude_storage/spec.md`
Before: 0. Expected: ≥1. Deviation: missing if 0.

**M2 — Topic dir mention present**
Command: `grep -c "topic.*dir\|dir.*topic\|hyphen.*prefix" module/claude_storage/spec.md`
Before: 0. Expected: ≥1. Deviation: missing if 0.

**M3 — Encoding constraint documented**
Command: `grep -c "sibling\|encoding.*constraint\|issue-031" module/claude_storage/spec.md`
Before: 0. Expected: ≥1. Deviation: missing if 0.

### Anti-faking checks

**AF1 — No stale path display description**
Check: `grep "project path is always shown" module/claude_storage/spec.md`
Expected: zero matches or updated text that includes the topic-dir extension rule.
