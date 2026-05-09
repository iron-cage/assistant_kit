# Plugin: Test lister

- **Status:** 🔒 Hardcoded — in `docker-run`
- **Controls:** What command enumerates available tests for the `.list` sub-command
- **Mechanism:** `cargo nextest list $CMD_SCOPE --all-features` hardcoded in `cmd_list` function

### Notes

Tied to nextest. Changing the test runner would break `.list` and require updates to `docker-run`'s `cmd_list` function.
