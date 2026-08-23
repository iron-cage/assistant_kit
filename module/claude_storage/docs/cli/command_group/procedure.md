# Command Group Documentation Operations

- **Actor:** Developer
- **Trigger:** A command is added to or removed from the CLI — every such change re-partitions this collection, whether or not a group file results.
- **Emits:** —

The entity operation is canonical at `cli_doc_des.rulebook.md § Entity Operations : Add Command Group · OC163`
and `§ Entity Operations : Remove Command Group · OC164`; the qualifying test is
`§ Commands Documentation : Representation Absorption Test · OC166` and
`§ Commands Documentation : Command Group Minimization · OC167`.

This collection is index-only in this crate: all 16 groups are singletons, so `readme.md` carries
the whole partition and no per-group instance file exists. That is the correct state under
Command Group Total Partition, not an incomplete collection — a dedicated file is warranted only
when a group has two or more members, because only then is there an equivalence claim to
document. Read `readme.md` § Navigation before concluding a file is missing.

## Evaluate a Candidate Pair

1. Run the Representation Absorption Test: the pair qualifies only when **both** hold — identical routine function, and identical CLI-facing parameter set differing at most in defaults
2. A shared *private helper* (`resolve_cmd_path`, `create_storage`, `resolve_scoped_projects`) does not satisfy criterion 1 — that is ordinary code reuse
3. Verify criterion 1 against source, not prose: `grep -rn '\b<routine>\b' src/ | grep -v 'pub fn <routine>'`, and confirm every remaining match is a `src/cli_main.rs` dispatch-map entry, a `src/cli/mod.rs` re-export, or a doc comment — never an actual call from another routine's body
4. Record the verdict either way — qualifying pairs become a group, non-qualifying near-misses get a row in **Evaluated, Not Qualifying** with the specific criterion that failed

## Add a Singleton Group (the common case)

1. Add the group's row to **All Groups** in `readme.md`, citing the routine function with its `src/` file and line
2. Update the total-count sentence below the table and the `(N singleton groups)` label in `../readme.md` Completion Matrix
3. Add an **Evaluated, Not Qualifying** row for each near-miss the new command surfaced — a shared private helper or a documented behavioral-parity claim is a near-miss worth recording
4. Record the pre-addition Representation Absorption Test verdict in the command's own `../command/NN_*.md` and append the cross-call sweep result to the total-count sentence
5. Update the pairwise-coverage sentence at the end of **Evaluated, Not Qualifying** (`N total minus the M listed above`)
6. Update the Evidentiary Basis section in `../../../tests/docs/cli/command_group/readme.md` so the zero-file verdict still matches the current command set

## Promote to a Multi-Member Group

1. Create `NN_{snake_case_name}.md` for the group, documenting membership, the shared routine, and every default divergence
2. Replace the affected singleton rows in **All Groups** with the merged group's row and link the new file
3. Register the file in the `readme.md` Responsibility Table and replace § Navigation's `(none — …)` note with a link to it
4. Register in `../../entity.md`: increment the `cli/command_group/` count and add a Master Doc Instances Table row
5. Register in `../../doc_graph.yml`: add the node and its edges, update `meta.node_count`/`edge_count`/`component_count`
6. Add `### Referenced Command Group` sections to each member command in `../command/`
7. Create the mirror test spec in `../../../tests/docs/cli/command_group/`, register it, and add the collection's row to `../../entity.md`

## Remove a Group

1. A group row leaves **All Groups** only when its routine leaves `src/cli_main.rs`'s `routines` map — deprecating the command is not enough (see `readme.md` § Command Removal for the worked `.list` case)
2. Reverse the registrations the group created, and update both total-count sentences and the pairwise-coverage sentence
3. If the removal was a deliberate capability absorption rather than a Representation Absorption Test merge, say so explicitly in `readme.md` — the two have different justifications and must not be conflated
