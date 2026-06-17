# Pitfall Test Spec Operations

- **Actor:** Developer
- **Trigger:** A pitfall doc instance is created or revised in `docs/pitfall/`.
- **Emits:** —

## Add Pitfall Test Spec

1. Identify the corresponding `docs/pitfall/NNN_*.md` doc instance
2. Assign the matching ordinal ID
3. Create `NN_{snake_case_name}.md` in this directory with PF- prefixed test cases (min 3)
4. Include: trap avoidance assertions, mitigation behavior verification, GWT-format test cases, source cross-references
5. Register in `readme.md` Overview Table: add row with name, purpose, and status ⏳
6. Update status to ✅ when all PF- cases have corresponding passing Rust tests

## Update Pitfall Test Spec

1. Edit the target `NN_*.md` file
2. If test cases added/removed: update the test case index and coverage summary
3. If pitfall scope changed: verify alignment with `docs/pitfall/NNN_*.md`
