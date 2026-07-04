# User Story Test Spec Operations

- **Actor:** Developer / QA
- **Trigger:** A user story doc instance is created or revised in `docs/cli/user_story/`.
- **Emits:** —

## Add User Story Test Spec

1. Identify the corresponding `docs/cli/user_story/NNN_*.md` doc instance
2. Assign the matching ordinal ID
3. Create `NNN_{snake_case_name}.md` in this directory with US- prefixed test cases (min 3)
4. Include: persona goal verification, GWT-format acceptance tests, end-to-end workflow coverage, source cross-references
5. Register in `readme.md` Responsibility Table: add row with name, purpose, and status ⏳
6. Update status to ✅ when all US- cases have corresponding passing Rust tests

## Update User Story Test Spec

1. Edit the target `NNN_*.md` file
2. If test cases added/removed: update the test case index and coverage summary
3. If user story scope changed: verify alignment with `docs/cli/user_story/NNN_*.md`
