# Type Test Documentation Operations

- **Actor:** Developer
- **Trigger:** A semantic type is added to or removed from `docs/cli/type/`.
- **Emits:** —

## Add Type Test Spec

1. Identify the type's `#` from `docs/cli/type/readme.md` Type Index
2. Create `NN_{type_name}.md` (2-digit zero-padded `#`) in this directory, numbering its cases `TC-N` continuing from the highest already used across this collection
3. Cover rejection as well as acceptance — a type exists to refuse values, so every constraint needs at least one case that violates it
4. Register in `readme.md`: add a Responsibility Table row, add an Aggregate Counts row with the spec's case count, and update the `**Total**`
5. Increment the `tests/docs/cli/type/` count in `../../../../docs/entity.md`
6. Name the implementing test file in the spec, and implement it

## Update Type Test Spec

1. Edit the target `NN_*.md` file
2. If the source type's validation rules changed: add or revise the affected cases in the same session as the type change
3. If the case count changed: update this spec's Aggregate Counts row and the `**Total**`

## Remove Type Test Spec

1. Delete the `NN_name.md` file, its Responsibility Table row, and its Aggregate Counts row; update the `**Total**`
2. Decrement the `tests/docs/cli/type/` count in `../../../../docs/entity.md`
3. Remove the tests the spec covered
