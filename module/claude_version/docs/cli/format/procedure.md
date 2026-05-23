# Format Catalog Procedure

### Trigger

Add a new file when a new named output format (`format::value`) is introduced to the cm CLI.

### Steps

1. Create `NNN_name.md` where `NNN` is the next sequential three-digit number and `name` matches the format value exactly.
2. Fill in: Scope, rendering rules, field catalog, Referenced Commands, Referenced User Stories.
3. Add a row to `format/readme.md` Format File Index.
4. Add `format::name` to all applicable command parameter tables in `001_commands.md`.
5. Update `005_params.md` `format::` Valid Values list.
6. Update `006_types.md` `OutputFormat` Valid values list.
7. Add `### Referenced User Stories` cross-references in the new format file.
8. Add the new format file node and edges to `../doc_graph.yml`.
