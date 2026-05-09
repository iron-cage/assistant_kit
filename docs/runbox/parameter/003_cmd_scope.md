# Parameter: `cmd_scope`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** `--workspace` (workspace runner) or `-p crate` (module runner)
- **Where It Flows:** `ARG CMD_SCOPE` in cook stage → `cargo nextest run $CMD_SCOPE`

### Notes

Single source of truth for both the dependency pre-compilation scope (cook stage) and the test execution scope (run stage). Changing one changes both atomically.
