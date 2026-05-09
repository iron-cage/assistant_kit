# Parameter: `test_script`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** module-relative path (e.g., `run/test`, `module/dream/run/test`)
- **Where It Flows:** `docker run /workspace/$TEST_SCRIPT` — executed after plugin mounts are wired

### Notes

Full-test entrypoint. May invoke bin plugins (e.g., `w3`) and assumes plugin mounts are present. Used by `docker-run`'s `.test` command path.

### Example

Workspace `runbox.yml`:
```yaml
test_script: run/test
```
Module `runbox.yml` (`claude_storage`):
```yaml
test_script: module/claude_storage/run/test
```
`cmd_test()` builds the full docker invocation:
```bash
docker run --rm \
  --user $(id -u):$(id -g) \
  -v workspace_test_plugin_targets:/tmp/will_test_targets \
  -v /usr/local/bin/w3:/usr/local/bin/w3:ro \
  -v /home/user/.claude:/workspace/.claude:rw \
  workspace_test \
  /workspace/run/test
```
The `run/test` script runs inside the container after all plugin mounts are wired, so it can invoke `w3` and read credentials from `/workspace/.claude`. This is the full-test path; `cmd_test_offline()` skips the script entirely and runs the baked CMD directly.
