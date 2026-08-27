# docs/

### Scope

**Responsibilities:** Documentation for the `claude_context_report_core` crate covering the context report model, the rendered table format, the CLI contract a consuming binary satisfies, and the no-disclosure constraint on rendered output.
**In Scope:** Report model and layer position (`feature/`), rendered table structure and vocabularies (`format/`), runtime output constraints (`invariant/`), and the doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`, not present yet), automated tests (→ `tests/`, not present yet), session-log parsing and state folding (→ `claude_storage_core/docs/`), credential detection (→ `json_redact/docs/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Report model, single responsibility, layer position, CLI contract |
| `format/` | Rendered table structure, columns, closed-set cell vocabularies |
| `invariant/` | No credential, account, or host disclosure in rendered output |
| `entity.md` | Doc Entity index for this crate's documentation scope |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

### Status

Docs-only planned crate — no `Cargo.toml`, not a workspace member. Follows the same pattern as `claude_patch` and `claude_patch_core`: the specification lands first, the manifest and sources follow once the design is accepted.

### Reading Order

1. [`feature/001_context_report.md`](feature/001_context_report.md) — what the crate is for and why it is Layer 0
2. [`format/001_context_report_tables.md`](format/001_context_report_tables.md) — the tables it renders to
3. [`invariant/001_no_private_data.md`](invariant/001_no_private_data.md) — what may never appear in them
4. [`feature/002_cli_contract.md`](feature/002_cli_contract.md) — the command that prints them
