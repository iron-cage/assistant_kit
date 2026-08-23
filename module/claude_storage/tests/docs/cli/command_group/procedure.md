# Command Group Test Documentation Operations

- **Actor:** Developer
- **Trigger:** A command is added to or removed from the CLI, or a `command_group` in `docs/cli/command_group/` gains its first second member.
- **Emits:** —

This mirror holds no per-group spec files, because `claude_storage` has no multi-member
`command_group` — all 16 groups are singletons. `readme.md` § Evidentiary Basis carries the
verdict instead. The zero-file state is the correct outcome of the operations below, not a gap.

## Refresh the Zero-Group Verdict (on every command add or remove)

1. Re-count `src/cli_main.rs`'s `routines` phf map and update the command-name/function-identifier count in `readme.md` § Evidentiary Basis point 1
2. Re-run the cross-call sweep for the new or removed routine: `grep -rn '\b<routine>\b' src/ | grep -v 'pub fn <routine>'`, and confirm every match is a dispatch-map entry, a `src/cli/mod.rs` re-export, or a doc comment. Update point 2 if the result changed
3. Update the `(N files, one per command)` count in the paragraph below the two points to match `../command/`
4. If the sweep found an actual cross-routine call, the zero-group verdict no longer holds — stop and follow Add Group Test Spec below

## Add Group Test Spec (only when a group gains a second member)

1. Take the group's `#` from `docs/cli/command_group/readme.md` **All Groups**, create `NN_{group_name}.md` in this directory
2. Specify the equivalence claim as a test: the members' shared routine must produce identical output for identical inputs, differing only where their documented defaults differ
3. Register in `readme.md` Responsibility Table, and replace § Navigation's `(none — zero qualifying groups)` note with a link
4. Increment the `tests/docs/cli/command_group/` count in `../../../../docs/entity.md`
5. Rewrite § Evidentiary Basis — it currently records a zero-group outcome that no longer holds

## Remove Group Test Spec

1. Delete the `NN_name.md` file and its `readme.md` row; restore the § Navigation note if this was the last one
2. Decrement the `tests/docs/cli/command_group/` count in `../../../../docs/entity.md`
3. Re-derive § Evidentiary Basis from the current `routines` map
