# Failure Mode Documentation Operations

- **Actor:** Developer
- **Trigger:** A new silent failure mode of the `claude` CLI is discovered or confirmed.
- **Emits:** —

## Add Failure Mode Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status
4. Register in `readme.md` Silent Fails Table: add row with #, Name, Fail Mode, Detection Channel, Sentinel
5. Add node to `../doc_graph.yml` under `nodes:` and any relevant edges under `edges:`; increment `node_count`

## Update Failure Mode Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update both tables in `readme.md`

## Example

Adding failure mode document `005_signal_exit_codes`:

1. Check `readme.md` Overview Table — current highest ID is `004`
2. Create `005_signal_exit_codes.md` in this directory
3. Add row to Overview Table: `| 005 | Signal Exit Codes | Exit 128+N means killed by signal | ✅ |`
4. Add row to Silent Fails Table: `| 005 | [Signal Exit Codes](005_signal_exit_codes.md) | Process killed by signal | exit code | \`exit_code > 128\` |`
