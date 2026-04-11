# Create docs/cli/format/ — CLI Output Format catalog for claude_storage

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Completed)

## Goal

Create the `docs/cli/format/` CLI Output Format Doc Entity for claude_storage, documenting all named output rendering modes (markdown, json, text for `.export`; table, tree, summary for verbosity-driven commands) so that CLI output behavior is fully specified in a semantic-named catalog, verified by the directory containing a readme.md master file and at least one format document per distinct rendering mode (Motivated: doc.rulebook.md § CLI Output Format Doc Entity requires this for CLIs with multiple named output rendering modes; Observable: `docs/cli/format/` directory with readme.md + per-format files; Scoped: only `docs/cli/format/` creation; Testable: `ls docs/cli/format/*.md | wc -l` returns ≥4).

## In Scope

- [NEW DIR] `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/format/` — create with readme.md master file
- [NEW FILE] `docs/cli/format/readme.md` — master file with Taxonomy, Catalog, and Rendering Convention sections
- [NEW FILE] `docs/cli/format/markdown.md` — markdown export format specification
- [NEW FILE] `docs/cli/format/json.md` — JSON export format specification
- [NEW FILE] `docs/cli/format/text.md` — plain text export format specification
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/readme.md` — add `format/` row to Responsibility Table
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/entities.md` — add `cli/format/` to Master Doc Entities Table (semantic naming)

## Out of Scope

- Verbosity-driven output modes (table, tree, summary) — these are verbosity levels, not `format::` parameter values; document in a separate task if needed
- Changes to `docs/cli/commands.md`, `params.md`, or `types.md` (existing content is accurate)
- `docs/` root-level files (covered in separate docs consistency task)
- Source code changes

## Description

The `doc.rulebook.md` vocabulary defines CLI Output Format Doc Entity as `docs/cli/format/` — a semantic-named catalog extension type with Taxonomy, Catalog, and Rendering Convention master file sections. The `cli.rulebook.md` vocabulary lists this as a "recognized optional extension: `format/` (output format catalog, when CLI has multiple named output rendering modes)."

claude_storage's `.export` command accepts `format::markdown|json|text`, representing three distinct named output rendering modes. Each produces structurally different output (markdown with headings and bold, JSON array of raw entries, plain text without markup). This qualifies for the `format/` catalog.

The format documents should reference `docs/cli/params.md § format::` and `docs/cli/types.md § ExportFormat` as the authoritative source for parsing and validation, while focusing on the *rendering specification* — what each format produces structurally, how entries are laid out, what metadata is included/excluded.

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- Format documents must use doc.rulebook.md § Standardized Heading Structure (H1 + H3, no H2)
- Format documents use semantic file naming (not NNN naming) per doc.rulebook.md § CLI Output Format Doc Entity
- Master file (readme.md) must include Taxonomy, Catalog, and Rendering Convention sections

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note doc.rulebook.md constraints on CLI Output Format Doc Entity structure, cli.rulebook.md § Multi-Format Output Standard.
2. **Read existing format documentation** — Read `docs/cli/params.md § format::`, `docs/cli/types.md § ExportFormat`, and `docs/cli/testing/param/format.md` to understand the current format specification.
3. **Read source code** — Read the export routine in `src/cli/mod.rs` to understand actual rendering logic for each format.
4. **Create format/ directory** — Create `docs/cli/format/` with `readme.md` master file containing:
   - Taxonomy section (what output formats exist, how they relate)
   - Catalog section (table of all formats with key properties)
   - Rendering Convention section (shared conventions across formats)
5. **Create per-format documents** — For each of markdown, json, text:
   - Title: `# Format: {Name}`
   - Sections: Purpose, Structure, Example Output, Related (cross-ref to params.md, types.md)
   - Semantic filename: `markdown.md`, `json.md`, `text.md`
6. **Update cli/readme.md** — Add `format/` row to Responsibility Table.
7. **Update entities.md** — Add `cli/format/` to Master Doc Entities Table with semantic naming note.
8. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Acceptance Criteria

- `docs/cli/format/` directory exists with readme.md and 3 format files
- readme.md master file has Taxonomy, Catalog, and Rendering Convention sections
- Each format file documents the rendering specification for one ExportFormat value
- `docs/cli/readme.md` Responsibility Table includes `format/` row
- `docs/entities.md` Master Doc Entities Table includes `cli/format/` row

## Validation Checklist

Desired answer for every question is YES.

**format/ directory structure**
- [ ] Does `docs/cli/format/readme.md` exist?
- [ ] Does `docs/cli/format/markdown.md` exist?
- [ ] Does `docs/cli/format/json.md` exist?
- [ ] Does `docs/cli/format/text.md` exist?

**Master file sections**
- [ ] Does readme.md contain `### Taxonomy` section?
- [ ] Does readme.md contain `### Catalog` section?
- [ ] Does readme.md contain `### Rendering Convention` section?

**Format file structure**
- [ ] Does each format file use H1 + H3 heading structure (no H2)?
- [ ] Does each format file cross-reference `params.md § format::` and `types.md § ExportFormat`?

**Integration**
- [ ] Does `docs/cli/readme.md` Responsibility Table include `format/` row?
- [ ] Does `docs/entities.md` include `cli/format/` in Master Doc Entities Table?

**Out of Scope confirmation**
- [ ] Are `docs/cli/commands.md`, `params.md`, `types.md` unchanged?
- [ ] Are `docs/` root files (other than entities.md) unchanged?

## Validation Procedure

### Measurements

**M1 — Format file count**
Command: `ls /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/format/*.md | wc -l`
Before: N/A (directory doesn't exist). Expected: 4 (readme.md + 3 format files). Deviation: missing files.

**M2 — Master file sections**
Command: `grep -c "### Taxonomy\|### Catalog\|### Rendering Convention" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/format/readme.md`
Before: N/A. Expected: 3. Deviation: missing required sections.

**M3 — cli/readme.md registration**
Command: `grep -c "format/" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/readme.md`
Before: 0. Expected: ≥1. Deviation: format/ not registered.

**M4 — entities.md registration**
Command: `grep -c "cli/format/" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/entities.md`
Before: 0. Expected: ≥1. Deviation: format/ entity not registered.

### Anti-faking checks

**AF1 — Format files have actual rendering content**
Check: `wc -l /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/format/markdown.md`
Expected: ≥20 lines. Why: catches empty placeholder files that satisfy file-existence checks but contain no specification.

**AF2 — No H2 headings in format files**
Check: `grep -c "^## " /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/format/markdown.md /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/format/json.md /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/format/text.md`
Expected: 0 for all files. Why: ensures doc.rulebook.md heading structure compliance (H1 + H3 only).

**AF3 — Cross-references to params.md are present**
Check: `grep -rc "params.md" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/format/`
Expected: ≥3 (one per format file). Why: ensures format files reference the authoritative parameter definition rather than duplicating it.
