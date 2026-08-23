# Pitfall Documentation Operations

- **Actor:** Developer
- **Trigger:** A bug fix, manual test, or code review exposes a mistake pattern likely to recur.
- **Emits:** —

`pitfall` is a project-specific extension entity, not one of the canonical `docs/cli/` entities
in `cli_doc_des.rulebook.md § Completion Levels : Docs CLI Entities Table · OC081` — it is
registered locally, per `§ Architecture : Entity Type Extension Protocol`. ID assignment and
permanence still follow `doc_des.rulebook.md § Collection : Doc Instance Lifecycle`.

A pitfall documents a *recurring* mistake pattern. A one-off defect belongs in the bug fix's own
source comment and test documentation (`style/l1_imp.rulebook.md`,
`l2_imp_organization.rulebook.md`), not here.

## Add Pitfall Documentation

1. Confirm recurrence: the same mistake shape has appeared, or would predictably appear, in more than one place. If it has not, stop — record it in the fix's own documentation instead
2. Assign the next `#` (check `readme.md`), create `NN_{snake_case_name}.md` with the local section set: `### Scope`, `### Pitfall`, `### Trigger`, `### Required Pattern`, `### Referenced Commands`, `### Sources`
3. Cite concrete evidence in `### Sources` — the bug IDs, commits, or files where the pattern actually appeared. A pitfall with no evidence is speculation
4. Register in `readme.md` Responsibility Table: add row with file name and a 3-10 word responsibility
5. Register in `../readme.md` Completion Matrix: bump the `pitfall/*.md (N files)` count
6. Register in `../../entity.md`: increment the `cli/pitfall/` count and add a Master Doc Instances Table row
7. Register in `../../doc_graph.yml`: add the node and its edges, update `meta.node_count`/`edge_count`/`component_count`
8. Create the mirror test spec `../../../tests/docs/cli/pitfall/NN_{same_name}.md` (`PF-` is the test *case* ID prefix inside the file, not part of the file name), register it there with its `All N CLI implementation pitfalls` count, and increment the `tests/docs/cli/pitfall/` count in `../../entity.md`

## Update Pitfall Documentation

1. Edit the target `NN_*.md` file
2. If a new occurrence is found: add it to `### Sources` and add the affected command to `### Referenced Commands` — recurrence evidence accumulates, it is not overwritten
3. If the Required Pattern changed: update the mirror test spec and the tests enforcing it in the same session
4. If cross-references changed: update the `../../doc_graph.yml` edges and `meta.edge_count`

## Retire Pitfall Documentation

1. Retire only when the pattern is structurally impossible, not merely absent — state which change made it unreachable
2. Keep the `readme.md` row and rewrite its responsibility to say so; the ID stays reserved
3. Reverse the registrations from Add steps 5, 6, 7, and 8
