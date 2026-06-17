# Format Test Spec Operations

- **Actor:** Developer
- **Trigger:** A format doc instance is created or revised in `docs/cli/format/`.
- **Emits:** —

## Add Format Test Spec

1. Identify the corresponding `docs/cli/format/NN_*.md` doc instance
2. Assign the matching ordinal ID
3. Create `NN_{snake_case_name}.md` in this directory with FM- prefixed test cases (min 4)
4. Include: rendering rule assertions, verbosity level variants, JSON validity checks, source cross-references
5. Register in `readme.md` Overview Table: add row with name, purpose, and status ⏳
6. Update status to ✅ when all FM- cases have corresponding passing Rust tests

## Update Format Test Spec

1. Edit the target `NN_*.md` file
2. If test cases added/removed: update the test case index and coverage summary
3. If format scope changed: verify alignment with `docs/cli/format/NN_*.md`
