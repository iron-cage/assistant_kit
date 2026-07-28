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

Adding invariant document `002_kind_filter_validation`:

1. Check `readme.md` Overview Table — current highest ID is `001`
2. Create `002_kind_filter_validation.md` in this directory
3. Add row: `| 002 | Kind Filter Validation | [002_kind_filter_validation.md](002_kind_filter_validation.md) | 🔄 |`
