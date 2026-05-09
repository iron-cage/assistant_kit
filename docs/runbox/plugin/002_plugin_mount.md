# Plugin: `plugin_mount`

- **Status:** ⚠️ Partial — configurable in `runbox.yml`; capacity hardcoded at one instance
- **Controls:** Host data directory mounted read-write into container
- **Mechanism:** Presence check → `-v host_dir:container_dir:type`; required+rw for `.test`, optional+ro for `.shell`

### Notes

Configured via `runbox.yml` key `plugin_mount: host_path:container_path:type`. Current use: `~/.claude:/workspace/.claude:directory`. A second mount slot requires code changes to `docker-run`.
