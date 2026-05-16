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

Adding invariant document `006_quota_always_checked`:

1. Check `readme.md` Overview Table — current highest ID is `005`
2. Create `006_quota_always_checked.md` in this directory
3. Add row: `| 006 | Invariant Name | [006_quota_always_checked.md](006_quota_always_checked.md) | ✅ |`
