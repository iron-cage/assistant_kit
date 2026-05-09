# Parameter: `cargo_features`

- **Status:** 🔒 Hardcoded — in `docker-run`
- **Current State:** `--all-features`
- **Where It Flows:** `cargo nextest list $CMD_SCOPE --all-features` in `cmd_list` function

### Notes

Hardcoded in `docker-run`. Projects with conflicting feature combinations need `--no-default-features -F specific_feature` instead.

### Example

`./run/docker .list` triggers `cmd_list()`:
```bash
CARGO_TARGET_DIR=/tmp/will_test_targets cargo nextest list --workspace --all-features
```
(`CARGO_TARGET_DIR` is set because `bin_plugin_volume` is configured — redirects build artifacts into the persistent plugin-targets volume so repeated `.list` calls reuse prior compilation.) `--all-features` is a literal in `docker-run`; it is not in `runbox.yml`. A workspace with mutually exclusive features that conflict under `--all-features` requires patching `cmd_list` in `docker-run` to use `--no-default-features -F specific_feature`.
