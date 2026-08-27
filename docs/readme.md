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
| `claude_code_background_task_env_vars.md` | Claude Code CLI background-task env var reference |

## Doc Entity Index

Workspace-level (`docs/`) entities with instance counts, maintained here:

| Scope | Entity | Type | Instances |
|-------|--------|------|-----------|
| workspace (`docs/`) | `feature/` | standard | 1 |
| workspace (`docs/`) | `invariant/` | standard | 6 |
| workspace (`docs/`) | `pattern/` | standard | 1 |
| workspace (`docs/`) | `integration/` | standard | 1 |
| workspace (`docs/`) | `error/` | extension | 6 |

Per-crate documentation scopes. Instance counts live in each crate's own
`docs/readme.md` (the authoritative index for that scope) — they are not
duplicated here, where they would silently drift:

| Crate | Entity dirs |
|-------|-------------|
| `module/assistant/` | `feature/`, `invariant/` |
| `module/assistant_kit/` | `feature/`, `invariant/` |
| `module/claude_assets/` | `feature/`, `invariant/` |
| `module/claude_assets_core/` | `feature/`, `invariant/` |
| `module/claude_auth/` | `api/`, `feature/`, `invariant/` |
| `module/claude_core/` | `api/` |
| `module/claude_daemon_core/` | `api/`, `feature/`, `invariant/` |
| `module/claude_journal/` | `api/`, `entity/`, `feature/`, `invariant/` |
| `module/claude_journal_charts/` | `api/` |
| `module/claude_journal_viewer/` | `cli/`, `entity/`, `feature/`, `invariant/` |
| `module/claude_memory/` | — (reserved skeleton: `docs/` exists, no instances yet) |
| `module/claude_patch/` | `feature/`, `invariant/` (docs-only planned crate) |
| `module/claude_patch_core/` | `feature/`, `invariant/` (docs-only planned crate) |
| `module/claude_profile/` | `algorithm/`, `cli/`, `entity/`, `feature/`, `invariant/`, `pattern/`, `pitfall/`, `research_interactive/`, `schema/`, `state_machine/`, `subprocess/`, `type/` |
| `module/claude_profile_core/` | `api/` |
| `module/claude_pty_core/` | `api/`, `feature/`, `invariant/` |
| `module/claude_quota/` | `api/` |
| `module/claude_runner/` | `algorithm/`, `api/`, `cli/`, `feature/`, `guide/`, `invariant/`, `variable/` |
| `module/claude_runner_core/` | `api/`, `claude_params/`, `data_structure/`, `failure_mode/`, `feature/`, `invariant/`, `pattern/` |
| `module/claude_session_core/` | `api/`, `feature/`, `invariant/` |
| `module/claude_storage/` | `algorithm/`, `cli/`, `feature/`, `invariant/`, `operation/` |
| `module/claude_storage_core/` | `algorithm/`, `api/`, `data_structure/`, `feature/`, `invariant/` |
| `module/claude_version/` | `algorithm/`, `cli/`, `feature/`, `pattern/`, `pitfall/`, `runtime_file/` |
| `module/claude_version_core/` | `algorithm/`, `api/`, `invariant/`, `pattern/` |
| `module/dream/` | `feature/`, `invariant/` |
| `module/json_redact/` | `api/` |
| `module/svg_chart/` | `api/` |

Every crate now carries a `docs/` collection. `claude_memory` is the sole
directory whose `docs/` holds no instances yet — a reserved skeleton with no
crate manifest. Test-side mirrors live under each crate's `tests/docs/`.

### Instance Naming

All doc instances use the `NNN_snake_case_name.md` format with a three-digit zero-padded ID. IDs are unique within their entity directory and are never reused after deletion.
