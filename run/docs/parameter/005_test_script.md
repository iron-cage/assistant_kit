# Parameter: `test_script`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** module-relative path (e.g., `run/test` for workspace; `module/claude_profile/verb/test` for modules)
- **Where It Flows:** `docker run /workspace/$TEST_SCRIPT` — executed after plugin mounts are wired

### Notes

Full-test entrypoint. May invoke bin plugins (e.g., `w3`) and assumes plugin mounts are present. Used by `runbox-run`'s `.test` command path. `_ensure_image()` probes for `test_script`, `lint_script`, and `run_script` in a single container run before executing any command; any missing file triggers an automatic rebuild rather than emitting a cryptic OCI "not found" error.

Use `$SCRIPT_DIR`-relative paths in the script body — inside the container `SCRIPT_DIR` resolves to `$WORKSPACE_DIR/run`, so `$SCRIPT_DIR/..` is `$WORKSPACE_DIR`. This also allows calling the script natively on the host when local dev tools are present.

Module-level runboxes point at `verb/test` (the canonical `do`-protocol test verb) rather than a bespoke `run/test` script. This makes `verb/test` the single source of truth for what "run tests" means for a module.

### Example

Workspace `runbox.yml`:
```yaml
test_script: run/test
```
Module `runbox.yml` (`claude_profile`):
```yaml
test_script: module/claude_profile/verb/test
```
`cmd_test()` builds the full docker invocation:
```bash
docker run --rm \
  --user $(id -u):$(id -g) \
  -v claude_profile_test_plugin_targets:/tmp/will_test_targets \
  -v /usr/local/bin/w3:/usr/local/bin/w3:ro \
  -v /home/user/.claude:/workspace/.claude:rw \
  claude_profile_test \
  /workspace/module/claude_profile/verb/test
```
The script runs inside the container after all plugin mounts are wired, so it can invoke `w3` and read credentials from `/workspace/.claude`. This is the full-test path; `cmd_test_offline()` skips the script entirely and runs the baked CMD directly.
