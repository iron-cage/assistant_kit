# Parameter: `cmd_filter`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** filter expression (e.g., `!binary(claude_direct_test)`)
- **Where It Flows:** Baked into image `CMD` as `--filter-expr $CMD_FILTER` at build time via dockerfile `ARG`

### Notes

Offline-safe default. Excludes tests that require live plugins (binary injection, host mounts). Tests requiring plugins run via `test_script` after plugin wiring.
