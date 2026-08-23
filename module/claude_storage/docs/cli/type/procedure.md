# Type Documentation Operations

- **Actor:** Developer
- **Trigger:** A semantic newtype constraining a CLI parameter value is added, changed, or removed.
- **Emits:** —

The full entity operation is canonical at `cli_doc_des.rulebook.md § Entity Operations : Add Type · OC057`
and `§ Entity Operations : Remove Type · OC062`. Run that operation; the steps below name only
the registries local to this crate that it has to land in.

Every parameter uses a named type with validation constraints — never a bare primitive. A new
parameter whose value shape no existing type covers needs its type documented here first.

## Add Type Documentation

1. Assign the next `#` (check the Type Index in `readme.md`), create `NN_{snake_case_name}.md`
2. Register in `readme.md`: add a Responsibility Table row, add a Type Index row, and update the `All N semantic types` count in Scope
3. Register in `../readme.md` Completion Matrix: bump the `type/*.md (N files)` count
4. Register in `../../entity.md`: increment the `cli/type/` count and add a Master Doc Instances Table row
5. Register in `../../doc_graph.yml`: add the node and its edges, update `meta.node_count`/`edge_count`/`component_count`
6. Add the reciprocal back-reference rows: every parameter using the type gains a `### Referenced Type` entry, and the new file's `### Referenced Commands` / `### Referenced Parameters` sections are filled from those same relationships
7. Create the mirror test spec `../../../tests/docs/cli/type/NN_{same_name}.md`, register it there, and increment the `tests/docs/cli/type/` count in `../../entity.md`

## Update Type Documentation

1. Edit the target `NN_*.md` file
2. If validation rules or valid values changed: update the mirror test spec, the parsing tests, and every parameter file whose `Valid Values` column restates the constraint — in the same session
3. If name or purpose changed: update the `readme.md` Responsibility Table and Type Index rows and the `../../entity.md` instance row
4. If cross-references changed: update the `../../doc_graph.yml` edges and `meta.edge_count`

## Remove Type Documentation

1. Follow `cli_doc_des.rulebook.md § Entity Operations : Remove Type · OC062` — a type with any parameter still referencing it is not removable; retarget those parameters first
2. Reverse every registration from Add steps 2-5 and 7; keep the ID reserved
3. Remove the reciprocal `### Referenced Type` entries the deleted file had claimed
