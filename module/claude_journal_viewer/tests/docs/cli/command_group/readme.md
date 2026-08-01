# Command Group Tests

### Scope

- **Purpose**: Document structural-equivalence tests for `clj` command groups.
- **Responsibility**: Index of per-command-group test files verifying shared-handler and shared-parameter-set claims.
- **In Scope**: Command group test files, when qualifying groups exist.
- **Out of Scope**: Per-command tests (→ `command/`), per-parameter-group interaction tests (→ `param_group/`).

Per-group structural-equivalence test indices for `clj`. See [command_group/readme.md](../../../../docs/cli/command_group/readme.md) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| *(none)* | No multi-member command_group exists — all 8 groups in the spec are singletons; see spec's All Groups table | N/A |

**Total:** 0 test files. `docs/cli/command_group/readme.md` documents 8 Singleton Groups (one per command, per the Total Partition requirement) and zero multi-member groups among the 8 documented `clj` commands — every command dispatches through a distinct implementing function in `src/output.rs`/`src/cli_main.rs`, so no shared-handler pair exists to write a structural-equivalence test for. Structural-equivalence test files are required only for multi-member groups, where an equivalence claim actually exists to test — a Singleton Group has no sibling to prove equivalence against.

No existing equivalence test in `tests/viewer_integration_test.rs` (EC-1 through EC-13) compares `clj .list` output to `ast .journal.list` output, or any other cross-layer (`cli_main.rs` vs. `routines.rs`) pair — that integration suite exercises only the `clj` binary via subprocess and never builds with the optional `routines` feature. This is a genuine coverage gap, not a claim of an existing test being indexed here: if the `routines` feature's cross-layer parity (`cmd_list`/`list_routine`, `cmd_stats`/`stats_routine`, `cmd_search`/`search_routine`, `cmd_status`/`status_routine`, `cmd_export`/`export_routine` all calling the same `output::*_output` function) is ever worth testing directly, that test does not exist yet and would need to be written, not indexed from here.
