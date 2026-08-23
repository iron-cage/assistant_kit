# Invariant Documentation Operations

- **Actor:** Developer
- **Trigger:** A behavioral contract the implementation must uphold is identified, changed, or retired.
- **Emits:** —

Mechanics of ID assignment, permanence, and deprecation are canonical at
`doc_des.rulebook.md § Collection : Doc Instance Lifecycle`. The steps below name only
what is local to this crate.

## Add Invariant Documentation

1. Assign the next available ID (check `readme.md` Responsibility Table for the current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Responsibility Table: add row with file name and a 3-10 word responsibility
4. Register in `../entity.md`: increment the `invariant/` count in the Master Doc Entities Table, add a row to the Master Doc Instances Table
5. Register in `../doc_graph.yml`: add the node, add every edge its text creates, update `meta.node_count`/`edge_count`/`component_count`
6. Create the mirror test spec `../../tests/docs/invariant/NNN_{same_name}.md` (same file name as the source instance; `IN-` is the test *case* ID prefix inside the file, not part of the file name) and register it in that directory's `readme.md`, then increment the `tests/docs/invariant/` count in `../entity.md`
7. Implement the contract test — an invariant with no enforcing test is a claim, not a contract

## Update Invariant Documentation

1. Edit the target `NNN_*.md` file
2. If the contract itself changed: update the mirror test spec and the test that enforces it in the same session
3. If name or responsibility changed: update the `readme.md` row and the `../entity.md` instance row
4. If cross-references changed: update the affected `../doc_graph.yml` edges and `meta.edge_count`

## Retire Invariant Documentation

1. Keep the `readme.md` row and rewrite its responsibility to state the contract is retired — the ID stays reserved and is never reused for a different invariant
2. Delete the node and its edges from `../doc_graph.yml`; update all three `meta` counts
3. Delete the mirror test spec, deregister it from `../../tests/docs/invariant/readme.md`, and remove the test that enforced the retired contract
4. Decrement both `invariant/` counts in `../entity.md`; keep the Master Doc Instances Table row so the reserved ID stays visible
