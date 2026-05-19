# Parameter: `cargo_features`

- **Status:** ✅ Configured — via `runbox.yml`; default: `--all-features`
- **Current State:** `--all-features`
- **Where It Flows:** `runbox.yml cargo_features:` → `--build-arg CARGO_FEATURES` → baked into image `CMD`; also read at runtime by `cmd_list` via `_plugin_list_cmd` in `runbox-run`

### Notes

Applied across two call sites: the baked offline `CMD` and `cmd_list`. `cmd_test_offline` uses the baked `CMD` natively — it does not read `CARGO_FEATURES` at runtime. Rebuild required for the `CMD` change to take effect; `cmd_list` picks up the new value immediately from `runbox.yml` without a rebuild.

### Example

Switching to a specific feature set to avoid conflicting features:
```yaml
cargo_features: --no-default-features -F storage_json
```
`runbox-run` passes `--build-arg CARGO_FEATURES=...` at build time and reads `$CARGO_FEATURES` at runtime for `.list`. Result: offline `CMD` bakes `--no-default-features -F storage_json` after `.build`; `cargo nextest list $CMD_SCOPE --no-default-features -F storage_json` for `.list` immediately.
