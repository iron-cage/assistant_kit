# Format Test Documentation Operations

- **Actor:** Developer
- **Trigger:** An export format is added to or removed from `docs/cli/format/`.
- **Emits:** —

## Add Format Test Spec

1. Identify the format's `#` from `docs/cli/format/readme.md` Catalog
2. Create `NN_{format_name}.md` (2-digit zero-padded `#`) in this directory, numbering its cases `FM-N` continuing from the highest already used across this collection
3. Assert output structure, not just exit status — a format test that only checks the command succeeded verifies nothing about the format
4. Register in `readme.md`: add a Responsibility Table row, add an Aggregate Counts row with the spec's case count, and update the `**Total**` and the `All N export formats` count in Scope
5. Increment the `tests/docs/cli/format/` count in `../../../../docs/entity.md`
6. Name the implementing test file in the spec, and implement it

## Update Format Test Spec

1. Edit the target `NN_*.md` file
2. If the case count changed: update this spec's Aggregate Counts row and the `**Total**`

## Remove Format Test Spec

1. Delete the `NN_name.md` file, its Responsibility Table row, and its Aggregate Counts row; update the `**Total**` and the Scope count
2. Decrement the `tests/docs/cli/format/` count in `../../../../docs/entity.md`
3. Remove the tests the spec covered
