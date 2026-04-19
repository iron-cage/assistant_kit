# Fix docs/ consistency — stale counts, isolated graph components, readme accuracy

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Completed)

## Goal

Fix two documentation consistency gaps in `claude_storage/docs/` so that doc_graph.yml component structure and readme.md command counts match the actual state of the CLI and documentation, verified by grep-based measurements showing zero stale values (Motivated: downstream tooling and human readers rely on accurate counts; Observable: doc_graph.yml, readme.md updated; Scoped: only `docs/` root-level files; Testable: all measurements in Validation Procedure pass).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/readme.md` — fix stale "9 commands" reference (actual: 13 commands, 12 stable + 1 deprecated)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/doc_graph.yml` — connect isolated components (feature/001 ↔ operation/001) by adding a cross-reference edge, update meta counts

## Out of Scope

- `docs/cli/` files (covered in separate CLI documentation task)
- `docs/feature/001_cli_tool.md` content updates (existing content is accurate)
- `docs/operation/001_migration_guide.md` content updates
- Source code changes

## Description

Post-migration consistency audit revealed three documentation drift issues in the `docs/` root:

1. **readme.md line 39**: States "All 9 commands" but `docs/cli/commands.md` lists 13 commands (12 stable + 1 deprecated). The count was never updated after commands were added.

2. **doc_graph.yml**: Contains 2 fully isolated components (feature/001 and operation/001), each with `isolated: true`. The gap notes in the YAML itself describe how to merge them: add a cross-reference from feature/001 to operation/001. This requires also adding a `### Related` or `### See Also` link in `docs/feature/001_cli_tool.md` pointing to operation/001.

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note doc.rulebook.md constraints on doc_graph.yml structure.
2. **Fix readme.md** — `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/readme.md` line 39: change "All 9 commands" to "All 13 commands" (or whatever the accurate count is per `docs/cli/commands.md`).
3. **Connect doc_graph.yml components** — Add an edge `feature/001 → operation/001` (or vice versa) and update meta: `edge_count: 0 → 1`, `component_count: 2 → 1`. Remove `isolated: true` from both components and merge into one component.
4. **Add cross-reference in feature/001** — Add a `### See Also` or `### Related` section in `docs/feature/001_cli_tool.md` linking to `operation/001_migration_guide.md` to justify the doc_graph edge.
5. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Acceptance Criteria

- readme.md command count matches `docs/cli/commands.md` Commands Table row count
- doc_graph.yml has zero isolated components (all nodes in one connected component)
- All cross-references resolve to existing files

## Validation Checklist

Desired answer for every question is YES.

**readme.md accuracy**
- [ ] Does readme.md § CLI Documentation reference the correct command count?
- [ ] Does the count match `docs/cli/commands.md` Commands Table?

**doc_graph.yml connectivity**
- [ ] Does doc_graph.yml have `component_count: 1`?
- [ ] Does doc_graph.yml have `edge_count: ≥ 1`?
- [ ] Are all components `isolated: false`?

**Out of Scope confirmation**
- [ ] Are docs/cli/ files unchanged by this task?

## Validation Procedure

### Measurements

**M1 — Command count in readme.md**
Command: `grep -c "All 13 commands" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/readme.md`
Before: 0 (says "9 commands"). Expected: ≥1. Deviation: stale count remains.

**M2 — Graph component count**
Command: `grep "component_count:" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/doc_graph.yml`
Before: `component_count: 2`. Expected: `component_count: 1`. Deviation: isolated components remain.

**M3 — Edge count**
Command: `grep "edge_count:" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/doc_graph.yml`
Before: `edge_count: 0`. Expected: `edge_count: 1` (or more). Deviation: no connectivity.

### Anti-faking checks

**AF1 — Cross-reference actually exists in feature/001**
Check: `grep -c "operation/001" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/feature/001_cli_tool.md`
Expected: ≥1. Why: ensures the doc_graph edge is backed by an actual cross-reference in the source document, not just a graph-only entry.

**AF2 — No "9 commands" remnant**
Check: `grep -c "9 commands" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/readme.md`
Expected: 0. Why: catches partial fix where count was added but old count not removed.

## Outcomes

All measurements and anti-faking checks passed:

- **M1**: `grep -c "All 13 commands"` → 1 (was 0)
- **M2**: `component_count: 1` (was 2)
- **M3**: `edge_count: 1` (was 0)
- **AF1**: `grep -c "operation/001" feature/001_cli_tool.md` → 1
- **AF2**: `grep -c "9 commands" readme.md` → 0

Files modified:
- `docs/readme.md` — "All 9 commands" → "All 13 commands"
- `docs/doc_graph.yml` — merged 2 isolated components into 1 connected component with 1 edge
- `docs/feature/001_cli_tool.md` — added `### See Also` cross-reference to operation/001
