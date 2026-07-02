# docs/

Documentation for the `assistant` workspace covering behavioral requirements, structural patterns, integration contracts, and invariants.

## Scope

Workspace-level behavioral requirements, structural patterns, integration contracts, and invariants for the `assistant` workspace. Per-crate documentation lives in each module's own `docs/` directory (e.g., `module/assistant/docs/`).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Workspace design and crate inventory |
| `invariant/` | Privacy, versioning, testing, and performance constraints |
| `pattern/` | Four-layer crate dependency architecture pattern |
| `integration/` | Cross-workspace integration protocol |
| `error/` | Claude Code error message catalog |
| `entity.md` | Doc Entity index for all documentation scopes in this workspace |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

## Doc Entity Index

All doc entities across the workspace, including per-crate documentation scopes.

| Scope | Entity | Type | Instances |
|-------|--------|------|-----------|
| workspace (`docs/`) | `feature/` | standard | 1 |
| workspace (`docs/`) | `invariant/` | standard | 5 |
| workspace (`docs/`) | `pattern/` | standard | 1 |
| workspace (`docs/`) | `integration/` | standard | 1 |
| workspace (`docs/`) | `error/` | extension | 6 |
| `module/assistant/` | `feature/` | standard | 1 |
| `module/assistant/` | `invariant/` | standard | 1 |
| `module/claude_journal/` | `feature/` | standard | 3 |
| `module/claude_journal/` | `invariant/` | standard | 3 |
| `module/claude_journal/` | `api/` | standard | 3 |
| `module/claude_journal_viewer/` | `feature/` | standard | 3 |
| `module/claude_journal_viewer/` | `invariant/` | standard | 2 |
| `module/claude_journal_viewer/` | `cli/` | cli | 62 |

### Instance Naming

All doc instances use the `NNN_snake_case_name.md` format with a three-digit zero-padded ID. IDs are unique within their entity directory and are never reused after deletion.
