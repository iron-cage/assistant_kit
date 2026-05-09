# Parameter: `cargo_features`

- **Status:** 🔒 Hardcoded — in `docker-run`
- **Current State:** `--all-features`
- **Where It Flows:** `cargo nextest list $CMD_SCOPE --all-features` in `cmd_list` function

### Notes

Hardcoded in `docker-run`. Projects with conflicting feature combinations need `--no-default-features -F specific_feature` instead.
