# Workflows

### All Workflows (5 total)

| # | Workflow | Commands | Use Case |
|---|----------|----------|----------|
| 1 | Environment check | `.status` | Verify installation and account state |
| 2 | Version upgrade | `.version.show`, `.version.install` | Upgrade Claude Code |
| 3 | Process management | `.processes`, `.processes.kill` | Inspect and terminate processes |
| 4 | Settings configuration | `.settings.show`, `.settings.get`, `.settings.set` | Read and modify settings |
| 5 | Monthly baseline refresh | `.version.list`, `.version.install`, `.version.show` | Sync to pinned monthly version |

---

### Workflow :: 1. Environment Check

Verify installation state, running sessions, and active account in one
command.

```sh
cm .status
cm .status format::json       # machine-readable
cm .status v::2               # verbose diagnostics
```

---

### Workflow :: 2. Version Upgrade

Check current version, preview the upgrade, then install.

```bash
# 1. Preview exactly what would run
cm .version.install version::2.1.78 dry::1
# [dry-run] would install stable (v2.1.78)
# [dry-run] would set autoUpdates = false
# [dry-run] would set env.DISABLE_AUTOUPDATER = 1
# [dry-run] would chmod 555 versions dir (hard lock)
# [dry-run] would purge stale cached binaries (keep v2.1.78)
# [dry-run] would store preferred version = 2.1.78 (v2.1.78)

# 2. Preview what would happen
cm .version.install dry::1

# 3. Install stable (default)
cm .version.install

# 4. Install specific version
cm .version.install version::1.2.3

# 5. Force reinstall even if already current
cm .version.install force::1
```

---

### Workflow :: 3. Process Management

Inspect running Claude Code processes and terminate them if needed.

```sh
# List active processes
cm .processes
cm .processes format::json

# 2. Preview the kill — confirm these are the right processes
cm .processes.kill dry::1
# [dry-run] would send SIGTERM to PID 4821
# [dry-run] would send SIGTERM to PID 4900

# 3. Kill all processes gracefully (SIGTERM → wait 2s → SIGKILL if needed)
cm .processes.kill
# killed 2 process(es)

# 4. Verify nothing remains
cm .processes
# (empty output — no active processes)
```

---

### Workflow :: 4. Settings Configuration

View all settings, read a specific key, or write a new value.

```sh
# View all settings
cm .settings.show
cm .settings.show format::json

# Read a specific setting
cm .settings.get key::theme

# Preview a write
cm .settings.set key::theme value::dark dry::1

# Write settings (type-inferred)
cm .settings.set key::theme value::dark          # string
cm .settings.set key::timeout value::30          # number
cm .settings.set key::autoUpdate value::true     # bool
```

---

### Workflow :: 5. Monthly Baseline Refresh

**Trigger:** Start of a new month; team syncs to the pinned `MONTH_VERSION` for reproducibility.

```bash
# 1. Check what month is pinned to
cm .version.list
# stable   2.1.78
# month    2.1.74
# latest   (resolves at install time)

# 2. Check what is currently installed
cm .version.show
# 2.1.78

# 3. Preview the downgrade (month is behind stable in this example)
cm .version.install version::month dry::1
# [dry-run] would install month (v2.1.74)
# [dry-run] would set autoUpdates = false
# [dry-run] would set env.DISABLE_AUTOUPDATER = 1
# [dry-run] would chmod 555 versions dir (hard lock)
# [dry-run] would purge stale cached binaries (keep v2.1.74)
# [dry-run] would store preferred version = month (v2.1.74)

# 4. Install the monthly baseline (force needed since 2.1.78 != 2.1.74)
cm .version.install version::month
# Installing Claude Code 2.1.74...
# Done.

# 5. Verify
cm .version.show
# 2.1.74
```

**Key points:**
- The monthly baseline exists for reproducibility: every team member runs the same `MONTH_VERSION`.
- Since `month` (2.1.74) differs from `stable` (2.1.78), the install proceeds without `force`.
- At month end, team returns to `stable` via `cm .version.install version::stable`.
