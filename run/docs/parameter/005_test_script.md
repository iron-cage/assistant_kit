# Parameter: `test_script`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** module-relative path (e.g., `run/test` for workspace; `module/claude_profile/verb/test` for modules)
- **Where It Flows:** `docker run /workspace/$TEST_SCRIPT` — executed after plugin mounts are wired

### Notes

Full-test entrypoint. May invoke bin plugins (e.g., `w3`) and assumes plugin mounts are present. Used by `docker-run`'s `.test` command path. `_ensure_image()` probes for the script inside the image before running; a missing probe triggers an automatic rebuild rather than emitting a cryptic OCI "not found" error.

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
