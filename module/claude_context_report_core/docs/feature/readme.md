# Feature Doc Entity

### Scope

- **Purpose**: Document what this crate produces and the command surface a consuming binary exposes to print it.
- **Responsibility**: Index of feature doc instances covering the report model and the CLI contract.
- **In Scope**: Report model types, single-responsibility statement, layer position, boundary against neighbouring crates, CLI arguments and exit codes.
- **Out of Scope**: Rendered table structure (→ [`../format/`](../format/readme.md)); redaction guarantees (→ [`../invariant/`](../invariant/readme.md)).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Context Report](001_context_report.md) | The report model, this crate's single responsibility, layer position, neighbour boundaries | 🔄 |
| 002 | [CLI Contract](002_cli_contract.md) | `clr context` surface — arguments, defaults, exit codes, delegation boundary | 🔄 |

### Status Legend

| Glyph | Meaning |
|-------|---------|
| 🔄 | Specified, not yet implemented |
| ✅ | Specified and implemented |

Both instances are 🔄: this is a docs-only planned crate with no manifest yet, following the same pattern as `claude_patch` and `claude_patch_core`.

### Cross-Collection Dependencies

**This collection depends on**:
- `../format/` — the rendered shape of what the model produces
- `../invariant/` — the redaction guarantee the model must satisfy

**This collection consumed by**:
- `claude_runner/docs/cli/` — once the command is implemented, its instance there references [002_cli_contract.md](002_cli_contract.md) rather than restating it
