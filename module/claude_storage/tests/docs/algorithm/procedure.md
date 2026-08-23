# Algorithm Test Documentation Operations

- **Actor:** Developer
- **Trigger:** An algorithm doc instance is added to or removed from `docs/algorithm/`.
- **Emits:** —

## Add Algorithm Test Spec

1. Take the source instance's file name verbatim from `docs/algorithm/readme.md` — the mirror file name matches it exactly (`AL-` is the test *case* ID prefix inside the file, not part of the name)
2. Create `NNN_{same_name}.md` in this directory, numbering its cases `AL-N` continuing from the highest already used across this collection
3. Register in `readme.md` Responsibility Table: add row `| NNN_name.md | AL- test cases for {name} algorithm (docs/algorithm/NNN_name.md) | ✅ |`
4. Increment the `tests/docs/algorithm/` count in `../../../docs/entity.md`
5. Name the implementing test file in the spec, and implement it — a spec with no test is a plan, not verification

## Remove Algorithm Test Spec

1. Delete the `NNN_name.md` file and its row in `readme.md`
2. Decrement the `tests/docs/algorithm/` count in `../../../docs/entity.md`
3. Remove the tests the spec covered, unless another spec still claims them
