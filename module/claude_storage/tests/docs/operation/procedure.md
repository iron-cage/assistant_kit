# Operation Test Documentation Operations

- **Actor:** Developer
- **Trigger:** An operation doc instance is added to or removed from `docs/operation/`.
- **Emits:** —

## Add Operation Test Spec

1. Take the source instance's file name verbatim from `docs/operation/readme.md` — the mirror file name matches it exactly (`OP-` is the test *case* ID prefix inside the file, not part of the name)
2. Create `NNN_{same_name}.md` in this directory, numbering its cases `OP-N` continuing from the highest already used across this collection
3. Register in `readme.md` Responsibility Table: add row `| NNN_name.md | OP- test cases for the {name} (docs/operation/NNN_name.md) | ✅ |`
4. Increment the `tests/docs/operation/` count in `../../../docs/entity.md`
5. Name the implementing test file in the spec, and implement it — an operation whose documented steps are never executed by a test drifts silently from the code

## Remove Operation Test Spec

1. Delete the `NNN_name.md` file and its row in `readme.md`
2. Decrement the `tests/docs/operation/` count in `../../../docs/entity.md`
3. Remove the tests the spec covered, unless another spec still claims them
