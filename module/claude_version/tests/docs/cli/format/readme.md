# Format Test Surface

### Scope

- **Purpose**: Test case specifications for claude_version output format doc instances.
- **Responsibility**: Per-format FM- test specs verifying output format rendering contracts.
- **In Scope**: Format rendering rules, verbosity interaction, JSON validity, text label behavior (FM- prefix, min 4 cases per spec).
- **Out of Scope**: Format parameter parsing (→ `../param/05_format.md`), command integration tests (→ `../command/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 01_text.md | FM- test cases for human-readable text format rendering | ✅ |
| 02_json.md | FM- test cases for machine-readable JSON format rendering | ✅ |
| procedure.md | Workflow for creating and updating format test specs | ✅ |
