# Output Format Documentation Operations

- **Actor:** Developer
- **Trigger:** A new export format is added to the `.export` command.
- **Emits:** —

## Add Format Documentation

1. Assign the next available ID (check `readme.md` Catalog for current highest `#`, increment by 1)
2. Create `NN_{format_name}.md` (2-digit zero-padded `#`) in this directory
3. Register in `readme.md` Catalog: add row with `#`, format name, trigger, file link, extension, machine-parseable flag

## Update Format Documentation

1. Edit the target `NNN_*.md` file
2. If format name or trigger changed: update `readme.md` Catalog row
