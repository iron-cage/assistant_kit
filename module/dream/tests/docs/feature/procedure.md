# Feature Documentation Operations

- **Actor:** Developer
- **Trigger:** A new feature is added or an existing one is significantly changed.
- **Emits:** —

## Add Feature Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Feature Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding test spec for feature `005_quota_enforcement`:

1. Check highest prefix in `tests/docs/feature/` — current highest is `004`
2. Create `005_quota_enforcement.md` with Edge Case Index, Coverage Summary, and FT- test cases (GWT format)
3. Add row to `readme.md`: `| 005_quota_enforcement.md | FT specs for quota enforcement feature | ✅ |`
4. Add entry to `tests/docs/inventory.md`
