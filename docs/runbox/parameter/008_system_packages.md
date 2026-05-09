# Parameter: `system_packages`

- **Status:** 🔒 Hardcoded — in `runbox.dockerfile`
- **Current State:** `curl procps`
- **Where It Flows:** `apt-get install -y curl procps` in test stage

### Notes

Project-specific: `curl` for version history fetching; `procps` for `kill`-based process management tests. Other workspaces differ (e.g., `willbe` also needs `git`).
