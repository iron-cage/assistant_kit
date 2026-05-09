# Invariant Documentation Operations

- **Actor:** Developer
- **Trigger:** A new invariant is identified or an existing constraint changes.
- **Emits:** —

## Add Invariant Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Invariant Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding test spec for invariant `003_token_limit_always_positive`:

1. Check highest prefix in `tests/docs/invariant/` — current highest is `002`
2. Create `003_token_limit_always_positive.md` with Edge Case Index, Coverage Summary, and IN- test cases (GWT format)
3. Add row to `readme.md`: `| 003_token_limit_always_positive.md | IN specs for token limit positive invariant | ✅ |`
4. Add entry to `tests/docs/inventory.md`
