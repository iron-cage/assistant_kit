# Plugin: Offline runner

- **Status:** 🔒 Hardcoded — baked into image `CMD` via `run/runbox.dockerfile` `ARG`
- **Controls:** What command the image's default `CMD` executes when run without `test_script`
- **Mechanism:** `cargo nextest run $CMD_SCOPE $CARGO_FEATURES --filter-expr $CMD_FILTER` baked at image build time; scope, features, and filter are all configurable via their respective parameters

### Notes

Always nextest. The command structure is hardcoded; `CMD_SCOPE`, `CARGO_FEATURES`, and `CMD_FILTER` are all baked in at build time from `runbox.yml` values. Changing the offline runner to a different test tool requires editing the `CMD` line in `run/runbox.dockerfile` and rebuilding.

### Example

For `workspace_test` built with `cmd_scope: --workspace`, `cargo_features: --all-features`, and `cmd_filter: "!test(lim_it) & !binary(behavior)"`, the dockerfile bakes:
```dockerfile
ARG CMD_SCOPE=--workspace
ARG CMD_FILTER=!test(lim_it) & !binary(behavior)
CMD cargo nextest run --workspace --all-features --filter-expr "!test(lim_it) & !binary(behavior)"
```
`docker run workspace_test` (no overriding command) executes this. `cmd_test_offline()` also invokes it explicitly with the build-cache volume mounted:
```bash
docker run --rm -v workspace_test_target:/workspace/target workspace_test \
  cargo nextest run --workspace --all-features --filter-expr "!test(lim_it) & !binary(behavior)"
```
Setting `cargo_features: --no-default-features -F core_only` in `runbox.yml` and rebuilding bakes `--no-default-features -F core_only` into the CMD instead.
