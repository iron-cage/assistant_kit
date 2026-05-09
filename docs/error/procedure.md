# Error Documentation Operations

- **Actor:** Developer
- **Trigger:** A new Claude Code error message is discovered or an existing error doc needs revision.
- **Emits:** —

## Add Error Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Include type-specific sections required by `readme.md` Scope: Abstract (H3), Trigger Conditions (H3), Recovery (H3)
4. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Error Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row
