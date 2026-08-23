# Feature Test Documentation Operations

- **Actor:** Developer
- **Trigger:** A feature doc instance is added to or removed from `docs/feature/`.
- **Emits:** —

## Add Feature Test Spec

1. Take the source instance's file name verbatim from `docs/feature/readme.md` — the mirror file name matches it exactly (`FT-` is the test *case* ID prefix inside the file, not part of the name)
2. Create `NNN_{same_name}.md` in this directory, numbering its cases `FT-N` continuing from the highest already used across this collection
3. Register in `readme.md` Responsibility Table: add row `| NNN_name.md | FT- test cases for the {name} feature (docs/feature/NNN_name.md) | ✅ |`
4. Increment the `tests/docs/feature/` count in `../../../docs/entity.md`
5. Name the implementing test file in the spec, and implement it — a spec with no test is a plan, not verification

## Remove Feature Test Spec

1. Delete the `NNN_name.md` file and its row in `readme.md`
2. Decrement the `tests/docs/feature/` count in `../../../docs/entity.md`
3. Remove the tests the spec covered, unless another spec still claims them
