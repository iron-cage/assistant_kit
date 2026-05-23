# Feature Test Spec Operations

- **Actor:** Developer
- **Trigger:** A feature doc instance is created or revised in `docs/feature/`.
- **Emits:** —

## Add Feature Test Spec

1. Identify the corresponding `docs/feature/NNN_*.md` doc instance
2. Assign the matching ordinal ID (mirrors the doc instance ID)
3. Create `NNN_{snake_case_name}.md` in this directory with FT- prefixed test cases (min 4)
4. Include: behavioral divergence pair, GWT-format test cases, source cross-references
5. Register in `readme.md` Overview Table: add row with name, purpose, and status ⏳
6. Update status to ✅ when all FT- cases have corresponding passing Rust tests

## Update Feature Test Spec

1. Edit the target `NNN_*.md` file
2. If test cases added/removed: update the test case index and coverage summary
3. If feature scope changed: verify alignment with `docs/feature/NNN_*.md`
