# Parameter Test Documentation Operations

- **Actor:** Developer
- **Trigger:** A new parameter is added to or removed from the CLI.
- **Emits:** —

## Add Parameter Test Spec

1. Identify the parameter's `#` number from `docs/cli/004_params.md`
2. Create `NN_name.md` (2-digit zero-padded `#`) in this directory
3. Register in `readme.md` Responsibility Table: add row `| NN_name.md | Edge case tests for param:: |`

## Remove Parameter Test Spec

1. Delete the `NN_name.md` file (NN IDs are not retired)
2. Remove the row from `readme.md` Responsibility Table
