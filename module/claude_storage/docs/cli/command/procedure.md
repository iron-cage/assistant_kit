# Command Documentation Operations

- **Actor:** Developer
- **Trigger:** A command is added to, changed in, or removed from the `clg` CLI.
- **Emits:** —

The full entity operation is canonical at `cli_doc_des.rulebook.md § Entity Operations : Add Command · OC055`
and `§ Entity Operations : Remove Command · OC060`. Run that operation; the steps below name only
the registries local to this crate that it has to land in.

## Add Command Documentation

1. Run the Representation Absorption Test against every existing command **before** choosing a new command name — a candidate that passes is a pre-configured alias, not a new command. Record the verdict in the new instance file and in `../command_group/readme.md` (`cli_doc_des.rulebook.md § Commands Documentation : Representation Absorption Test · OC166`)
2. Assign the next `#` (check the Commands Table in `readme.md`), create `NN_{snake_case_name}.md`
3. Register in `readme.md`: add a Responsibility Table row, add a Commands Table row, and update the `All N registered commands` count in Scope
4. Add the command's Singleton Group row to `../command_group/readme.md` **All Groups**, and update its total-count sentence
5. Register in `../readme.md` Completion Matrix: bump the `command/*.md (N files)` count
6. Register in `../../entity.md`: increment the `cli/command/` count and add a Master Doc Instances Table row
7. Register in `../../doc_graph.yml`: add the node and its edges, update `meta.node_count`/`edge_count`/`component_count`
8. Add the reciprocal back-reference rows the new file's `### Referenced Parameters` / `### Referenced Parameter Groups` / `### Referenced User Stories` sections create — every reference is bidirectional
9. Create the mirror test spec `../../../tests/docs/cli/command/NN_{same_name}.md`, register it there, and increment the `tests/docs/cli/command/` count in `../../entity.md`

## Update Command Documentation

1. Edit the target `NN_*.md` file
2. If syntax, parameters, or exit codes changed: update the mirror test spec and the tests in the same session
3. If name or purpose changed: update the `readme.md` Responsibility Table and Commands Table rows and the `../../entity.md` instance row
4. If references changed: update both directions of every affected back-reference section, and the `../../doc_graph.yml` edges plus `meta.edge_count`

## Remove Command Documentation

1. Follow `cli_doc_des.rulebook.md § Entity Operations : Remove Command · OC060` — deprecate first (status `❌`, file retained), delete only once the routine leaves `src/cli_main.rs`'s `routines` map
2. Drop the group row from `../command_group/readme.md` **All Groups** and update its total-count sentence
3. Reverse every registration from Add steps 3, 5, 6, 7, and 9; keep the ID reserved
4. Remove the reciprocal back-reference rows the deleted file's reference sections had created elsewhere
