# Parameter: `image`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** per-module tag (e.g., `claude_tools_test`, `dream_test`)
- **Where It Flows:** `docker build -t $IMAGE $WORKSPACE` → `docker run $IMAGE`

### Notes

Unique per module. Prevents image tag collisions between workspace-level and crate-level builds.

### Example

Workspace `runbox.yml`:
```yaml
image: workspace_test
```
Module `runbox.yml` (`claude_profile`):
```yaml
image: claude_profile_test
```
`_build()` invokes `docker build -f run/runbox.dockerfile -t workspace_test .`. `cmd_test()` runs `docker run ... workspace_test /workspace/run/test`. Build-cache volumes are also derived from the tag: `workspace_test_target`, `workspace_test_plugin_targets` — so both images coexist on the same host with fully isolated caches.
