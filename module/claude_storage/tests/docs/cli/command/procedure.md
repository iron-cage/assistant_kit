# Command Test Documentation Operations

- **Actor:** Developer
- **Trigger:** A new command is added to or removed from the CLI.
- **Emits:** —

## Add Command Test Spec

1. Identify the command's `#` number from `docs/cli/001_commands.md`
2. Create `NN_name.md` (2-digit zero-padded `#`) in this directory
3. Register in `readme.md` Responsibility Table: add row `| NN_name.md | Integration tests for .command |`

## Remove Command Test Spec

1. Delete the `NN_name.md` file (NN IDs are not retired — no retired-ID row preserved)
2. Remove the row from `readme.md` Responsibility Table
