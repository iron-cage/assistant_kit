# Invariant Test Documentation Operations

- **Actor:** Developer
- **Trigger:** An invariant doc instance is added to or removed from `docs/invariant/`.
- **Emits:** —

An invariant is a contract. Its mirror spec is not optional documentation — it is the record of
how the contract is enforced, and `docs/invariant/procedure.md` requires it in the same session
the invariant is written.

## Add Invariant Test Spec

1. Take the source instance's file name verbatim from `docs/invariant/readme.md` — the mirror file name matches it exactly (`IN-` is the test *case* ID prefix inside the file, not part of the name)
2. Create `NNN_{same_name}.md` in this directory, numbering its cases `IN-N` continuing from the highest already used across this collection
3. Cover the violation conditions, not just the happy path — a contract test that only asserts conformance never proves the contract binds
4. Register in `readme.md` Responsibility Table: add row `| NNN_name.md | IN- test cases for {name} invariant (docs/invariant/NNN_name.md) | ✅ |`, and update the `All N ... invariants` count in Scope
5. Increment the `tests/docs/invariant/` count in `../../../docs/entity.md`
6. Name the implementing test file in the spec, and implement it

## Remove Invariant Test Spec

1. Remove only when the invariant itself is retired — a contract that still holds keeps its test
2. Delete the `NNN_name.md` file and its row in `readme.md`; update the Scope count
3. Decrement the `tests/docs/invariant/` count in `../../../docs/entity.md`
4. Remove the tests that enforced the retired contract
