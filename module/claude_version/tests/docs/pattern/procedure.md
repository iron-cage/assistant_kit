# Pattern Test Spec Operations

- **Actor:** Developer
- **Trigger:** A pattern doc instance is created or revised in `docs/pattern/`.
- **Emits:** —

## Add Pattern Test Spec

1. Identify the corresponding `docs/pattern/NNN_*.md` doc instance
2. Assign the matching ordinal ID
3. Create `NNN_{snake_case_name}.md` in this directory with PT- prefixed test cases (min 3)
4. Include: behavioral divergence pair, GWT-format test cases, source cross-references
5. Register in `readme.md` Overview Table: add row with name, purpose, and status ⏳
6. Update status to ✅ when all PT- cases have corresponding passing Rust tests

## Update Pattern Test Spec

1. Edit the target `NNN_*.md` file
2. If test cases added/removed: update the test case index and coverage summary
3. If pattern scope changed: verify alignment with `docs/pattern/NNN_*.md`
