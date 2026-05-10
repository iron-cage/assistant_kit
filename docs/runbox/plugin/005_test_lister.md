# Plugin: Test lister

- **Status:** 🔒 Hardcoded — in `docker-run`
- **Controls:** What command enumerates available tests for the `.list` sub-command
- **Mechanism:** `cargo nextest list $CMD_SCOPE $CARGO_FEATURES` in `cmd_list` function; feature flags are configurable via `cargo_features` parameter

### Notes

Tied to nextest. The command structure (`cargo nextest list`) is hardcoded; the feature flags it uses follow the `cargo_features` parameter. Changing the test runner would require updating `cmd_list` in `docker-run`.

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
With `cargo_features: --no-default-features -F storage_json` in `runbox.yml`, the list command becomes `cargo nextest list --workspace --no-default-features -F storage_json` — listing only tests compilable under that feature set.
