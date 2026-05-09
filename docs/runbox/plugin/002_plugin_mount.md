# Plugin: `plugin_mount`

- **Status:** ⚠️ Partial — configurable in `runbox.yml`; capacity hardcoded at one instance
- **Controls:** Host data directory mounted read-write into container
- **Mechanism:** Presence check → `-v host_dir:container_dir:type`; required+rw for `.test`, optional+ro for `.shell`

### Notes

Configured via `runbox.yml` key `plugin_mount: host_path:container_path:type`. Current use: `~/.claude:/workspace/.claude:directory`. A second mount slot requires code changes to `docker-run`.

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
