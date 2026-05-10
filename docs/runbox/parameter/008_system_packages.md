# Parameter: `system_packages`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** `curl procps`
- **Where It Flows:** `runbox.yml system_packages:` → `--build-arg SYSTEM_PACKAGES` → `apt-get install -y --no-install-recommends $SYSTEM_PACKAGES` in test stage; empty value skips the install block entirely

### Notes

Workspace-specific: `curl` for version history fetching; `procps` for `/bin/kill`-based process management tests. `willbe` also needs `git`. Empty string (or absent key) disables the apt-get block entirely — safe for workspaces with no system dependencies.

### Example

Adding `git` for a crate that shells out to git:
```yaml
system_packages: curl procps git
```
`docker-run` passes `--build-arg SYSTEM_PACKAGES=curl procps git` → dockerfile runs `apt-get install -y --no-install-recommends curl procps git`. To use no system packages at all, set the key to an empty string or remove it — the install block is skipped when `$SYSTEM_PACKAGES` is empty.
