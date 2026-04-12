# Create `docs/` directory structure for `claude_assets`, `claude_assets_core`, and `claude_tools`

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 📥 (Backlog)

## Goal

Create the missing `docs/` directory structure (with `feature/`, `invariant/`, and relevant doc instances) for `claude_assets`, `claude_assets_core`, and `claude_tools` so that all three crates have the same documentation completeness as other mature crates in the workspace (Motivated: all other mature crates — `claude_runner_core`, `claude_storage`, `claude_profile`, `claude_manager` — have `docs/` with behavioral requirements; these three crates were created or retained without docs, leaving behavior undocumented; Observable: `docs/feature/`, `docs/invariant/` subdirectories with at least one doc instance each in all three crates; Scoped: three crates' `docs/` directories only; Testable: `find module/claude_assets module/claude_assets_core module/claude_tools -name "*.md" -path "*/docs/*"` returns ≥ 6 files).

## In Scope

**`module/claude_assets_core/docs/`** — new directory:
- `docs/readme.md` — directory index with doc type registration
- `docs/feature/readme.md` — feature doc instance index
- `docs/feature/001_artifact_installer.md` — Feature doc: symlink-based installer design (ArtifactKind, ArtifactLayout, AssetPaths, install/uninstall semantics, symlink-only invariant)
- `docs/invariant/readme.md` — invariant doc instance index
- `docs/invariant/001_symlink_only.md` — Invariant doc: install() MUST use `std::os::unix::fs::symlink()` only; refuses to copy files; data-loss guard via `symlink_metadata()`

**`module/claude_assets/docs/`** — new directory:
- `docs/readme.md` — directory index with doc type registration
- `docs/feature/readme.md` — feature doc instance index
- `docs/feature/001_asset_cli.md` — Feature doc: `.list`, `.install`, `.uninstall`, `.kinds` CLI commands; `$PRO_CLAUDE` resolution; adapter alias expansions (`v::` → `verbosity::`)
- `docs/invariant/readme.md` — invariant doc instance index
- `docs/invariant/001_source_root_resolution.md` — Invariant doc: `$PRO_CLAUDE` env var required; fallback to `$PRO/genai/claude/` when `$PRO` set; errors when neither available

**`module/claude_tools/docs/`** — new directory:
- `docs/readme.md` — directory index with doc type registration
- `docs/feature/readme.md` — feature doc instance index
- `docs/feature/001_super_app_aggregation.md` — Feature doc: programmatic command registration via `register_commands()` from each Layer 2 crate; single `clt` binary; feature-gated aggregation
- `docs/invariant/readme.md` — invariant doc instance index
- `docs/invariant/001_aggregation_completeness.md` — Invariant doc: every Layer 2 crate registered in `claude_tools` must expose `register_commands()` and `COMMANDS_YAML`; no Layer 2 CLI command can exist outside `clt`

**Registration updates (3 files):**
- `module/claude_assets_core/readme.md` — add `docs/` row
- `module/claude_assets/readme.md` — add `docs/` row
- `module/claude_tools/readme.md` — add `docs/` row

## Out of Scope

- Changes to any source code (`src/`)
- Changes to any existing doc instances in other crates
- API documentation (covered by doc comments in `src/`)

## Description

Three crates in the workspace — `claude_assets_core`, `claude_assets`, and `claude_tools` — were created or evolved without the `docs/` directory structure that all other mature crates have. This leaves their behavioral requirements undocumented at the feature/invariant level.

`claude_assets_core` and `claude_assets` were created together in TSK-089 as a focused new crate pair; `claude_tools` predates the docs convention and was never backfilled.

Each `docs/` structure follows the pattern established by `claude_runner_core/docs/`: a root `docs/readme.md`, `feature/` and `invariant/` subdirectories each with a `readme.md` index, and at least one numbered doc instance in each collection. Doc instances follow the `doc.rulebook.md` format: `# Type: Name`, `### Scope`, `### Design`, `### Cross-References`.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- All doc instances must follow `doc.rulebook.md` § Standardized Heading Structure (H1 title, H3 sections, no H2)
- Title format: `# Feature: Name` or `# Invariant: Name`
- Every new directory must be registered in its parent `readme.md`
- Content must be accurate to the current implementation — read source files before writing

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; internalize `doc.rulebook.md` § Doc Entity : Common Doc Instance Requirements and § Doc Entity : Standardized Heading Structure.
2. **Read reference crate** — Read `module/claude_runner_core/docs/` structure as the established pattern.
3. **Read claude_assets_core source** — Read `src/artifact.rs`, `src/install.rs`, `src/paths.rs` to write accurate feature/invariant docs.
4. **Create `claude_assets_core/docs/`** — Create all 5 files (readme.md, feature/readme.md, feature/001_artifact_installer.md, invariant/readme.md, invariant/001_symlink_only.md).
5. **Read claude_assets source** — Read `src/commands.rs`, `src/adapter.rs`, `unilang.commands.yaml` to write accurate feature/invariant docs.
6. **Create `claude_assets/docs/`** — Create all 5 files.
7. **Read claude_tools source** — Read `src/main.rs` and `Cargo.toml` to write accurate feature/invariant docs.
8. **Create `claude_tools/docs/`** — Create all 5 files.
9. **Register in parent readmes** — Add `docs/` row to each of the three crate-root `readme.md` files.
10. **Verify** — `find module/claude_assets module/claude_assets_core module/claude_tools -name "*.md" -path "*/docs/*"` → ≥ 15 files total.
11. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `find .../docs -name "*.md"` | claude_assets_core | ≥ 5 doc files |
| T02 | `find .../docs -name "*.md"` | claude_assets | ≥ 5 doc files |
| T03 | `find .../docs -name "*.md"` | claude_tools | ≥ 5 doc files |
| T04 | H2 heading check | All new doc instances | Zero `## ` (H2) headings in doc instances |
| T05 | Title format check | All new doc instances | All titles match `# Feature:` or `# Invariant:` pattern |

## Acceptance Criteria

- `docs/feature/` and `docs/invariant/` directories exist in all three crates
- Each has at least one numbered doc instance (e.g., `001_*.md`) per collection type
- All doc instances use flat H1/H3 heading structure (no H2)
- All doc instances have accurate `### Scope` and `### Cross-References` sections
- Parent `readme.md` files for all three crates list the new `docs/` directory
- `w3 .test level::3` still passes (documentation-only changes, no regressions)

## Validation

### Checklist

Desired answer for every question is YES.

**Directory existence**
- [ ] C1 — Do `docs/feature/` and `docs/invariant/` exist in `claude_assets_core/`?
- [ ] C2 — Do `docs/feature/` and `docs/invariant/` exist in `claude_assets/`?
- [ ] C3 — Do `docs/feature/` and `docs/invariant/` exist in `claude_tools/`?

**Doc instance quality**
- [ ] C4 — Does every new doc instance have a `# Type: Name` H1 title?
- [ ] C5 — Does every new doc instance have a `### Scope` section?
- [ ] C6 — Does every new doc instance have a `### Cross-References` table?
- [ ] C7 — Are all doc instances free of H2 headings (`## `)?

**Registration**
- [ ] C8 — Does each crate's `readme.md` list its `docs/` directory?

**Out of Scope confirmation**
- [ ] C9 — Are no source files (`src/`) modified?
- [ ] C10 — Are no existing doc instances in other crates modified?

### Measurements

- [ ] M1 — total new doc files: `find module/claude_assets module/claude_assets_core module/claude_tools -name "*.md" -path "*/docs/*" | wc -l` → ≥ 15 (was: 0)
- [ ] M2 — test suite: `w3 .test level::3` → 0 failures (documentation-only, no regressions)

### Anti-faking checks

- [ ] AF1 — no H2 in doc instances: `grep -r "^## " module/claude_assets/docs module/claude_assets_core/docs module/claude_tools/docs` → 0 matches
- [ ] AF2 — title format: `grep -rL "^# Feature:\|^# Invariant:" module/claude_assets/docs/feature module/claude_assets_core/docs/feature module/claude_tools/docs/feature` → 0 files missing the prefix
- [ ] AF3 — cross-refs present: `grep -rL "### Cross-References" module/claude_assets/docs/feature module/claude_assets_core/docs/feature` → 0 files missing the section

## Outcomes

Created 15 new documentation files across three crates:

**claude_assets_core/docs/** (5 files):
- `docs/readme.md` — directory index
- `docs/feature/readme.md` — feature doc entity index
- `docs/feature/001_artifact_installer.md` — ArtifactKind taxonomy, ArtifactLayout, AssetPaths resolution, install/uninstall semantics and idempotency
- `docs/invariant/readme.md` — invariant doc entity index
- `docs/invariant/001_symlink_only.md` — symlink-only constraint, data-loss guards via symlink_metadata()

**claude_assets/docs/** (5 files):
- `docs/readme.md` — directory index
- `docs/feature/readme.md` — feature doc entity index
- `docs/feature/001_asset_cli.md` — .list/.install/.uninstall/.kinds commands, adapter alias expansion, 5-phase unilang pipeline, exit codes
- `docs/invariant/readme.md` — invariant doc entity index
- `docs/invariant/001_source_root_resolution.md` — $PRO_CLAUDE resolution order, .kinds graceful degradation exception

**claude_tools/docs/** (5 files):
- `docs/readme.md` — directory index
- `docs/feature/readme.md` — feature doc entity index
- `docs/feature/001_super_app_aggregation.md` — build_registry() registration order, static YAML aggregation, adapter reuse, .claude stub, feature gate
- `docs/invariant/readme.md` — invariant doc entity index
- `docs/invariant/001_aggregation_completeness.md` — register_commands() + COMMANDS_YAML contract for all Layer 2 crates

**Registration updates** (3 parent readmes updated with `docs/` row):
- `module/claude_assets_core/readme.md`
- `module/claude_assets/readme.md`
- `module/claude_tools/readme.md`

All 15 doc instances verified: zero H2 headings, all titled with `# Feature:` or `# Invariant:`, all have `### Scope` and `### Cross-References` sections.
