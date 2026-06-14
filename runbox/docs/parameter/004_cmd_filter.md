# Parameter: `cmd_filter`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** filter expression (e.g., `all()`)
- **Where It Flows:** Baked into image `CMD` as `--filter-expr $CMD_FILTER` at build time via dockerfile `ARG`

### Notes

Offline-safe default. Runs all workspace tests. `lim_it` tests run by default — they skip automatically via `live_active_token()` guard when no credentials are mounted. Tests requiring plugins run via `test_script` after plugin wiring (`cmd_test()`).

Note: `contract/claude_code` (which owns a `behavior` integration test) is excluded from the workspace via `Cargo.toml` `exclude`; using `!binary(behavior)` with `--workspace` produces "no binary names matched" in nextest. Use `all()` to match all workspace tests without triggering this error.

### Example

Workspace `runbox.yml`:
```yaml
cmd_filter: "all()"
```
Passed as `--build-arg CMD_FILTER=...` and baked into the image CMD:
```dockerfile
ARG CMD_FILTER=all()
CMD cargo nextest run --workspace --all-features --filter-expr "all()"
```
`docker run workspace_test` (no override) executes all workspace tests. `cmd_test_offline()` also uses this same baked filter. `lim_it` tests self-skip when `~/.claude/.credentials.json` is absent.
