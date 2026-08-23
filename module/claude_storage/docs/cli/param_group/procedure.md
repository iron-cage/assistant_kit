# Parameter Group Documentation Operations

- **Actor:** Developer
- **Trigger:** Parameters spanning multiple commands are recognized as controlling one operational concern, or an existing group's membership changes.
- **Emits:** —

The full entity operation is canonical at `cli_doc_des.rulebook.md § Entity Operations : Add Parameter Group · OC058`
and `§ Entity Operations : Remove Parameter Group · OC063`. Run that operation; the steps below name only
the registries local to this crate that it has to land in.

A parameter group is the looser grouping: parameters sharing an operational concern across
commands. It is not `../command_group/`, which requires an identical routine function and an
identical parameter set.

## Add Parameter Group Documentation

1. Assign the next `#` (check `readme.md`), create `NN_{snake_case_name}.md`
2. Register in `readme.md`: add a Responsibility Table row, add an Overview row, and update the `All N parameter groups` count in Scope
3. Register in `../readme.md` Completion Matrix: bump the `param_group/*.md (N files)` count
4. Register in `../../entity.md`: increment the `cli/param_group/` count and add a Master Doc Instances Table row
5. Register in `../../doc_graph.yml`: add the node and its edges, update `meta.node_count`/`edge_count`/`component_count`
6. Add the reciprocal back-reference rows: every member parameter's `### Referenced Parameter Groups` and every consuming command's `### Referenced Parameter Groups` gain a row in the same session
7. Create the mirror test spec `../../../tests/docs/cli/param_group/NN_{same_name}.md`, register it there, and increment the `tests/docs/cli/param_group/` count in `../../entity.md`

## Update Parameter Group Documentation

1. Edit the target `NN_*.md` file
2. If membership changed: update both directions of every affected back-reference section, and the mirror test spec's interaction cases in the same session
3. If name or purpose changed: update the `readme.md` Responsibility Table and Overview rows and the `../../entity.md` instance row
4. If cross-references changed: update the `../../doc_graph.yml` edges and `meta.edge_count`

## Remove Parameter Group Documentation

1. Follow `cli_doc_des.rulebook.md § Entity Operations : Remove Parameter Group · OC063`
2. Reverse every registration from Add steps 2-5 and 7; keep the ID reserved
3. Remove the reciprocal back-reference rows the deleted file had created in every member parameter, command, and user story
