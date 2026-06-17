# Catalog Test Spec Operations

- **Actor:** Developer
- **Trigger:** A catalog doc instance is created or revised in `docs/catalog/`.
- **Emits:** —

## Add Catalog Test Spec

1. Identify the corresponding `docs/catalog/NNN_*.md` doc instance
2. Assign the matching ordinal ID
3. Create `NN_{snake_case_name}.md` in this directory with DD- prefixed test cases (min 3)
4. Include: decision implementation assertions, GWT-format test cases, source cross-references
5. Register in `readme.md` Overview Table: add row with name, purpose, and status ⏳
6. Update status to ✅ when all DD- cases have corresponding passing Rust tests

## Update Catalog Test Spec

1. Edit the target `NN_*.md` file
2. If test cases added/removed: update the test case index and coverage summary
3. If design decision scope changed: verify alignment with `docs/catalog/NNN_*.md`
