# Parameter: `cargo_features`

- **Status:** ✅ Configured — via `runbox.yml`; default: `--all-features`
- **Current State:** `--all-features`
- **Where It Flows:** `runbox.yml cargo_features:` → `--build-arg CARGO_FEATURES` → baked into image `CMD`; also used at runtime by `cmd_test_offline` and `cmd_list` in `runbox-run`

### Notes

Applied consistently across all three call sites: the baked offline `CMD`, `cmd_test_offline`, and `cmd_list`. A workspace with mutually exclusive features can set `--no-default-features -F specific_feature` to avoid compilation conflicts at all three sites simultaneously. Rebuild required for the `CMD` change to take effect; `cmd_test_offline` and `cmd_list` pick up the new value immediately from `runbox.yml`.

### Example

Switching to a specific feature set to avoid conflicting features:
```yaml
cargo_features: --no-default-features -F storage_json
```
`runbox-run` passes `--build-arg CARGO_FEATURES=...` and uses `$CARGO_FEATURES` at runtime. Result: `cargo nextest run $CMD_SCOPE --no-default-features -F storage_json --filter-expr "..."` in offline runs, and `cargo nextest list $CMD_SCOPE --no-default-features -F storage_json` for `.list`. The offline `CMD` gets the same flags baked in after `.build`.
