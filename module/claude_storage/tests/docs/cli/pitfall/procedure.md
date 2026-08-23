# Pitfall Test Documentation Operations

- **Actor:** Developer
- **Trigger:** A pitfall doc instance is added to or removed from `docs/cli/pitfall/`.
- **Emits:** —

## Add Pitfall Test Spec

1. Take the source instance's file name verbatim from `docs/cli/pitfall/readme.md` — the mirror file name matches it exactly (`PF-` is the test *case* ID prefix inside the file, not part of the name)
2. Create `NN_{same_name}.md` in this directory, numbering its cases `PF-N` continuing from the highest already used across this collection
3. Write each case to fail when the pitfall's mistake pattern is present, not merely to pass when it is absent — a pitfall test that cannot fail is the pitfall `04_vacuous_assertions_mask_stubs.md` warns about
4. Register in `readme.md` Responsibility Table: add row `| NN_name.md | PF- test cases for {name} pitfall (docs/cli/pitfall/NN_name.md) | ✅ |`, and update the `All N CLI implementation pitfalls` count in Scope
5. Increment the `tests/docs/cli/pitfall/` count in `../../../../docs/entity.md`
6. Name the implementing test file in the spec, and implement it

## Remove Pitfall Test Spec

1. Remove only when the source pitfall is retired — a pattern that is merely absent still needs its guard
2. Delete the `NN_name.md` file and its row in `readme.md`; update the Scope count
3. Decrement the `tests/docs/cli/pitfall/` count in `../../../../docs/entity.md`
4. Remove the tests the spec covered
