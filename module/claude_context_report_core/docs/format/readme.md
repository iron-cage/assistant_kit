# Format Doc Entity

### Scope

- **Purpose**: Specify the rendered shape of a context report — the tables a consumer prints, their columns, and their cell vocabularies.
- **Responsibility**: Master file for the `format` collection — lists all format instances describing report output structure.
- **In Scope**: Table column definitions, closed-set cell vocabularies, ordering and stability rules, the weight glyph scale, placeholder tokens.
- **Out of Scope**: The in-memory report model (→ [`../feature/`](../feature/readme.md)); redaction policy (→ [`../invariant/`](../invariant/readme.md)).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Context Report Tables](001_context_report_tables.md) | The three report tables, their columns, vocabularies, and ordering rules | 🔄 |

### Type-Specific Requirements

All `format` doc instances must include:

1. **Title**: `# Format: {Concept Name}`
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Abstract** (H3): What the format is for, in prose
4. **Column tables** (H3 per table): one row per column with Type, Required, Meaning
5. **Closed-set vocabularies** (H3 or inline): every enum column's permitted values, exhaustively
6. **Cross-References** (H3): flat table with `Type | File | Responsibility` columns

### Placeholder Discipline

No instance in this collection may contain a real path, host, account, or session identifier. Host-specific values appear only as placeholder tokens defined in [001_context_report_tables.md](001_context_report_tables.md). This is a documentation-side application of the runtime guarantee in [`../invariant/001_no_private_data.md`](../invariant/001_no_private_data.md) — a format spec is the artifact most likely to be copied verbatim, so it must be safe by construction.

### Cross-Collection Dependencies

**This collection depends on**:
- `../feature/` — the report model these formats render

**This collection consumed by**:
- Any binary rendering a context report
