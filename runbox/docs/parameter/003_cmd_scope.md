# Parameter: `cmd_scope`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** `--workspace` (workspace runner) or `-p crate` (module runner)
- **Where It Flows:** `ARG CMD_SCOPE` in cook stage → `cargo nextest run $CMD_SCOPE`

### Notes

Single source of truth for both the dependency pre-compilation scope (cook stage) and the test execution scope (run stage). Changing one changes both atomically.

### Example

Workspace runner:
```yaml
cmd_scope: --workspace
```
Module runner (`claude_profile`):
```yaml
cmd_scope: -p claude_profile
```
Both propagate as `--build-arg CMD_SCOPE=...` to `_build()`. In the dockerfile, the same `$CMD_SCOPE` arg drives two stages:
```dockerfile
# cook stage — pre-compiles exactly the deps needed by the crate under test
RUN cargo chef cook --recipe-path recipe.json $CMD_SCOPE --tests

# final CMD baked at build time — executes the same scope at runtime
ARG CMD_SCOPE=--workspace
CMD cargo nextest run $CMD_SCOPE --filter-expr "$CMD_FILTER"
```
Changing `cmd_scope: -p claude_profile` → `cmd_scope: --workspace` widens both the dep pre-compilation and the test execution scope atomically with a single `runbox.yml` edit followed by `.build`.
