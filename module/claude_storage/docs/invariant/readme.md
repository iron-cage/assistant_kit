# Invariant Doc Entity

### Scope

- **Purpose**: Document behavioral contracts the implementation must uphold at all times.
- **Responsibility**: Path encoding contract, session family grouping, JSONL entry type format.
- **In Scope**: Encode/decode rules, grouping invariants, format constraints with violation conditions.
- **Out of Scope**: CLI command specs (→ `../cli/command/`), implementation details (→ source code).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `001_path_encoding.md` | Path encode/decode contract |
| `002_session_family.md` | Session family grouping contract |
| `003_entry_type_format.md` | JSONL entry type field contract |
