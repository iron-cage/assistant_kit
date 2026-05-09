# Parameter: `cmd_filter`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** filter expression (e.g., `!binary(claude_direct_test)`)
- **Where It Flows:** Baked into image `CMD` as `--filter-expr $CMD_FILTER` at build time via dockerfile `ARG`

### Notes

Offline-safe default. Excludes tests that require live plugins (binary injection, host mounts). Tests requiring plugins run via `test_script` after plugin wiring.

### Example

Workspace `runbox.yml`:
```yaml
cmd_filter: "!test(lim_it) & !binary(behavior)"
```
Passed as `--build-arg CMD_FILTER=...` and baked into the image CMD:
```dockerfile
ARG CMD_FILTER=!test(lim_it) & !binary(behavior)
CMD cargo nextest run --workspace --filter-expr "!test(lim_it) & !binary(behavior)"
```
`docker run workspace_test` (no override) executes this offline-safe subset. Tests matching `binary(behavior)` require the `bin_plugin` (`w3`) and `plugin_mount` (`~/.claude`); they are excluded here and run via `test_script` in `cmd_test()` after plugin wiring. `cmd_test_offline()` also uses this same baked filter explicitly.
