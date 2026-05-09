# Plugin: Offline runner

- **Status:** 🔒 Hardcoded — baked into image `CMD` via `runbox.dockerfile` `ARG`
- **Controls:** What command the image's default `CMD` executes when run without `test_script`
- **Mechanism:** `cargo nextest run $CMD_SCOPE --filter-expr $CMD_FILTER` baked at image build time via dockerfile `ARG`

### Notes

Always nextest. Baked at build time — changing the offline runner requires rebuilding the image. The filter expression is also baked in at this point via `CMD_FILTER`.

### Example

For `workspace_test` built with `cmd_scope: --workspace` and `cmd_filter: "!test(lim_it) & !binary(behavior)"`, the dockerfile bakes:
```dockerfile
ARG CMD_SCOPE=--workspace
ARG CMD_FILTER=!test(lim_it) & !binary(behavior)
CMD cargo nextest run --workspace --filter-expr "!test(lim_it) & !binary(behavior)"
```
`docker run workspace_test` (no overriding command) executes this. `cmd_test_offline()` also invokes it explicitly with the build-cache volume mounted:
```bash
docker run --rm -v workspace_test_target:/workspace/target workspace_test \
  cargo nextest run --workspace --filter-expr "!test(lim_it) & !binary(behavior)"
```
Changing the offline runner to plain `cargo test` requires editing the CMD line in `runbox.dockerfile` and rebuilding — the filter expression syntax (`--filter-expr`) is nextest-specific and would also need updating.
