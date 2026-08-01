# Command Group Test Spec Operations

- **Actor:** Developer
- **Trigger:** A new command group is defined or an existing group test spec needs revision.
- **Emits:** —

## Add Command Group Test Spec

1. Confirm the candidate pair actually qualifies per `docs/cli/command_group/readme.md`'s Representation Absorption Test (same routine function, same parameter set, defaults-only divergence) before creating a test file — do not file a spec for a candidate that only shares a lower-level helper or a loose product relationship.
2. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
3. Create `NN_{snake_case_group_name}.md` in this directory
4. Include: group summary, structural-equivalence test index pointing to existing per-command equivalence tests (do not re-specify identical Given/When/Then content already covered by `command/*.md`)
5. Register in `readme.md` Overview Table: add row with filename and responsibility
6. Add Navigation entry in parent `../readme.md` Command Groups list (create the list if this is the first qualifying group)

## Update Command Group Test Spec

1. Edit the target `NN_*.md` file
2. If group membership or shared routine changed: update `readme.md` Overview Table row and the source `docs/cli/command_group/NN_*.md` file together
3. If structural-equivalence tests added/removed: update test case index and coverage summary

## Example

Adding a command group test spec once a genuine alias exists (e.g. a future `.version.show` gaining a `.version.info` alias that delegates its full dispatch to `version_show_routine()` with zero parameter divergence):

1. Check `readme.md` Overview Table — currently empty (0 groups qualify)
2. Create `01_version_show_info.md` indexing the existing `.version.show` integration tests under the command_group entity
3. Add row: `| 01_version_show_info.md | Structural-equivalence tests for Group 1 (version.show / version.info) |`
4. Add to parent Navigation: `- [version.show / version.info](command_group/01_version_show_info.md)`
