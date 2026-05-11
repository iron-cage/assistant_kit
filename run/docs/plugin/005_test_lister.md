# Plugin: Test lister

- **Status:** ✅ Configurable — `_plugin_list_cmd()` hook in `run/plugins.sh`
- **Controls:** What command enumerates available tests for the `.list` sub-command
- **Mechanism:** `_plugin_list_cmd()` stub in `runbox-run` sets `$list_cmd`; called after `_plugin_list_args()` so `$cargo_env` reflects active plugin state; `cmd_list()` executes `bash -c "$list_cmd"` inside the container

### Notes

Default stub: `list_cmd="${cargo_env}cargo nextest list $CMD_SCOPE $CARGO_FEATURES"`. Override `_plugin_list_cmd()` in `run/plugins.sh` to swap the list command for a different test runner (e.g., `list_cmd="jest --listTests"` for Node). The hook receives `$cargo_env` from `_plugin_list_args()` via bash dynamic scope — the default stub incorporates any env prefix set by the active bin-plugin.

### Example

`./run/runbox .list` triggers `cmd_list()`. With `run/plugins.sh` active and `bin_plugin_volume: /tmp/will_test_targets`, `_plugin_list_args` injects `CARGO_TARGET_DIR` and a volume flag:
```bash
CARGO_TARGET_DIR=/tmp/will_test_targets \
  cargo nextest list --workspace --all-features
```
The `CARGO_TARGET_DIR` redirect routes nextest's artifact output into the `workspace_test_plugin_targets` volume, so repeated `.list` invocations reuse prior compilation. Without `run/plugins.sh`, cmd_list runs `cargo nextest list --workspace --all-features` with no volume or env override.

Output: a newline-separated list of all test names available in the image, e.g.:
```
claude_storage::tests::session_path_creates_parent
claude_storage::tests::session_path_unique_per_id
…
```
With `cargo_features: --no-default-features -F storage_json` in `runbox.yml`, the list command becomes `cargo nextest list --workspace --no-default-features -F storage_json` — listing only tests compilable under that feature set.

### Multi-Ecosystem Examples

Override `_plugin_list_cmd()` in the project's `run/plugins.sh` for non-Rust ecosystems:

**Python (pytest):**
```bash
_plugin_list_cmd() {
  list_cmd="/workspace/.venv/bin/pytest --collect-only -q /workspace/tests/"
}
```
Output: one test node ID per line (`tests/test_example.py::test_add`).

**Node.js (node --test):**
```bash
_plugin_list_cmd() {
  list_cmd="node --test --test-reporter=spec /workspace/tests/"
}
```
Output: spec-format test names from the Node.js built-in test runner.

**Rust (cargo test -- --list, without nextest):**
```bash
_plugin_list_cmd() {
  list_cmd="cargo test --manifest-path /workspace/Cargo.toml -- --list"
}
```
Use when the image does not include cargo-nextest. Output: `test_name: test` per line.
