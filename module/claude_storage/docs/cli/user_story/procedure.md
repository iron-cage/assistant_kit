# User Story Operations

- **Actor:** Developer
- **Trigger:** A new user goal, persona scenario, or acceptance criterion is identified; or an existing story becomes obsolete.

## Add User Story

1. Choose next NNN number (3-digit, check User Story Index for current highest)
2. Create `NNN_{slug}.md` following canonical template (`cli_doc.rulebook.md § Documentation Style : User Stories`)
3. Add row to `readme.md` User Story Index table
4. For each command in `### Referenced Commands`: add a back-reference row to that command's `### Referenced User Stories` in `command/NN_name.md`
5. For each parameter in `### Referenced Parameters`: add a back-reference row to that parameter's `### Referenced User Stories` in `param/NN_name.md`
6. For each group in `### Referenced Parameter Groups`: add a back-reference row to that group's `### Referenced User Stories` in `param_group/NN_name.md`
7. For each format in `### Referenced Formats`: add a back-reference row to that format's `### Referenced User Stories` in `format/NN_*.md`
8. For each story in `### Related User Stories`: add a reciprocal row to that story's `### Related User Stories`
9. Update `docs/entities.md` — increment `cli/user_story/` Instances column
10. Run bidirectional integrity check: every forward reference must have a back-reference

## Remove User Story

1. Remove `NNN_{slug}.md`; remove its row from `readme.md` User Story Index
2. Remove its back-reference row from `### Referenced User Stories` in every command it referenced; if any command drops below 1 row, identify a replacement before completing
3. Remove its back-reference row from every parameter's `### Referenced User Stories`; same minimum-1 check
4. Remove its back-reference row from every group's `### Referenced User Stories`; same minimum-1 check
5. Remove its back-reference row from every format's `### Referenced User Stories`; verify each format retains ≥1 row
6. Remove its row from every related story's `### Related User Stories`
7. Update `docs/entities.md` — decrement `cli/user_story/` Instances column
