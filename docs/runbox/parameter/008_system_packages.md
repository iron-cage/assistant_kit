# Parameter: `system_packages`

- **Status:** 🔒 Hardcoded — in `runbox.dockerfile`
- **Current State:** `curl procps`
- **Where It Flows:** `apt-get install -y curl procps` in test stage

### Notes

Project-specific: `curl` for version history fetching; `procps` for `kill`-based process management tests. Other workspaces differ (e.g., `willbe` also needs `git`).

### Example

```dockerfile
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl procps \
 && rm -rf /var/lib/apt/lists/*
```
`procps` provides `/bin/kill` used by `send_sigterm`/`send_sigkill` in `claude_core::process` — tests fail with `ENOENT` without it. `curl` is used by version history commands to fetch GitHub release data. Other workspaces differ: `willbe` also installs `git` for repository operations. Adding a package requires editing this line directly; there is no `runbox.yml` key.
