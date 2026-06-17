# Catalog Documentation Operations

- **Actor:** Developer
- **Trigger:** A new design decision is identified or an existing decision is revised.
- **Emits:** —

## Add Catalog Instance

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status
4. Add cross-references from any related feature or pattern doc instances via `### Catalogs`

## Update Catalog Instance

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Retire Catalog Instance

1. Remove the `NNN_*.md` file
2. Remove its row from `readme.md` Overview Table
3. Remove any `### Catalogs` cross-reference rows pointing to this instance from other doc instances
