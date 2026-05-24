# Parameter Group Test Documentation Operations

- **Actor:** Developer
- **Trigger:** A new parameter group is defined or removed.
- **Emits:** —

## Add Parameter Group Test Spec

1. Identify the group's `#` number from `docs/cli/003_parameter_groups.md`
2. Create `NN_name.md` (2-digit zero-padded `#`) in this directory
3. Register in `readme.md` Responsibility Table: add row `| NN_name.md | Interaction tests for Group N |`

## Remove Parameter Group Test Spec

1. Delete the `NN_name.md` file (NN IDs are not retired)
2. Remove the row from `readme.md` Responsibility Table
