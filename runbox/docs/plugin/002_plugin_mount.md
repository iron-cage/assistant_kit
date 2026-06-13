# Plugin: `plugin_mount`

- **Status:** ✅ Configured — defined in `runbox/plugins.sh`; configured via `runbox.yml`
- **Controls:** Host data directory mounted read-write into container
- **Mechanism:** `_resolve_mount()` in `runbox/plugins.sh` — presence check → `-v host_dir:container_dir:type`; required+rw for `.test`, optional+ro for `.shell`

### Notes

Configured via `runbox.yml` key `plugin_mount: host_path:container_path:type`. Current use: `~/.claude:/workspace/.claude:directory`. Plugin logic lives entirely in `runbox/plugins.sh` — core `runbox-run` has no plugin knowledge. A second mount slot requires additions to `plugins.sh` only; `runbox-run` is unchanged.

**Host-side prerequisite for `directory` mounts under `$WORKSPACE_DIR`:** runc applies bind-mounts in parent-before-child order.  After the workspace `:ro` bind-mount is applied, `/workspace` in the container reflects the HOST `$WORKSPACE_ROOT` directory — image layers at that path are hidden.  runc then processes the child mount `/workspace/.claude:rw`; if `$WORKSPACE_ROOT/.claude/` is absent on the HOST, runc calls `mkdirat()` which fails with EROFS because `/workspace` is `:ro`.

`cmd_test()` in `runbox-run` handles this automatically: the `mkdir -p` loop before `podman run` pre-creates every `mount_args` container path that falls under `$WORKSPACE_DIR` in `$WORKSPACE_ROOT`.  No manual action required.

**Dockerfile `RUN mkdir $WORKSPACE_DIR/.claude`** is kept as belt-and-suspenders for containers run without the workspace `:ro` overlay (e.g., custom `docker run` invocations).  It is not the primary fix for the workspace `:ro` scenario.  See BUG-001 in `agent_kit/task/runbox/bug/` for full root cause analysis.

### Example

```yaml
plugin_mount: ~/.claude:/workspace/.claude:directory
```
`_resolve_mount()` expands `~` → `/home/user/.claude`, checks `[[ -d "/home/user/.claude" ]]`.

`.test` path — required + rw:
```bash
plugin_mount=$(_resolve_mount "$PLUGIN_MOUNT" true rw)
# → /home/user/.claude:/workspace/.claude:rw
docker run -v /home/user/.claude:/workspace/.claude:rw ...
```
Tests that read Claude credentials find them at `/workspace/.claude/credentials.json` because `ENV HOME=/workspace` makes the container home `/workspace`. If `~/.claude` is absent, `_resolve_mount` prints an error and exits.

`.shell` path — optional + ro:
```bash
shell_mount=$(_resolve_mount "$PLUGIN_MOUNT" false ro)
# → /home/user/.claude:/workspace/.claude:ro  (or empty string if absent)
```
Missing directory silently skips the mount — you get a usable shell without credentials.
