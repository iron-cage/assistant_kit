# Parameter Documentation Operations

- **Actor:** Developer
- **Trigger:** A CLI parameter is added, changed, or removed.
- **Emits:** —

The full entity operation is canonical at `cli_doc_des.rulebook.md § Entity Operations : Add Parameter · OC056`
and `§ Entity Operations : Remove Parameter · OC061`. Run that operation; the steps below name only
the registries local to this crate that it has to land in.

## Add Parameter Documentation

1. Assign the next `#` (check the Parameters Table in `readme.md`), create `NN_{snake_case_name}.md`
2. Register in `readme.md`: add a Responsibility Table row, add a Parameters Table row, update the `All N CLI parameters` count in Scope, and extend the `**Total:**` sentence with why the parameter was introduced
3. Register in `../readme.md` Completion Matrix: bump the `param/*.md (N files)` count
4. Register in `../../entity.md`: increment the `cli/param/` count and add a Master Doc Instances Table row
5. Register in `../../doc_graph.yml`: add the node and its edges, update `meta.node_count`/`edge_count`/`component_count`
6. Add the reciprocal back-reference rows for the new file's `### Referenced Type`, `### Referenced Parameter Groups`, `### Referenced Commands`, and `### Referenced User Stories` sections — every consuming command's `### Referenced Parameters` gains a row in the same session
7. Create the mirror test spec `../../../tests/docs/cli/param/NN_{same_name}.md`, register it there, and increment the `tests/docs/cli/param/` count in `../../entity.md`

## Update Parameter Documentation

1. Edit the target `NN_*.md` file
2. If type, default, or valid values changed: update the `readme.md` Parameters Table row, the mirror test spec, and the tests in the same session
3. If the consuming command set changed: update both directions of every affected back-reference section and the `Used In` count in the Parameters Table
4. If cross-references changed: update the `../../doc_graph.yml` edges and `meta.edge_count`

## Remove Parameter Documentation

1. Follow `cli_doc_des.rulebook.md § Entity Operations : Remove Parameter · OC061`
2. Reverse every registration from Add steps 2-5 and 7; keep the ID reserved and never reuse it
3. Remove the reciprocal back-reference rows the deleted file had created in every command, type, group, and user story it referenced
