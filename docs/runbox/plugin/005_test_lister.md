# Plugin: Test lister

- **Status:** 🔒 Hardcoded — in `docker-run`
- **Controls:** What command enumerates available tests for the `.list` sub-command
- **Mechanism:** `cargo nextest list $CMD_SCOPE --all-features` hardcoded in `cmd_list` function

### Notes

Tied to nextest. Changing the test runner would break `.list` and require updates to `docker-run`'s `cmd_list` function.

### Example

`./run/docker .list` triggers `cmd_list()`:
```bash
CARGO_TARGET_DIR=/tmp/will_test_targets \
  cargo nextest list --workspace --all-features
```
The `CARGO_TARGET_DIR` redirect routes nextest's artifact output into the `workspace_test_plugin_targets` volume, so repeated `.list` invocations reuse prior compilation. Output: a newline-separated list of all test names available in the image, e.g.:
```
claude_storage::tests::session_path_creates_parent
claude_storage::tests::session_path_unique_per_id
…
```
Switching to `cargo test` would require changing `cmd_list` to `cargo test --workspace -- --list --all-features` and adjusting any tooling that parses the nextest list format (nextest uses a different output layout than `cargo test --list`).
