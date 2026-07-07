# Install a target Claude Code version safely

**Persona:** developer
**Goal:** Preview exactly what will happen, then install the target version with version locking applied — and verify the result.
**Benefit:** Changes Claude Code version safely without unintended installs or broken auto-update settings.
**Priority:** High

### Acceptance Criteria

- [ ] `clv .version.install version::X dry::1` shows the full install plan without executing.
- [ ] `clv .version.install version::X` installs, applies 8-layer version lock, and exits 0.
- [ ] Already-at-target is a no-op (exits 0) unless `force::1` is set.
- [ ] `clv .version.show` after install prints the newly installed version.
- [ ] `clv .version.history` shows recent releases with summaries to aid version selection.
- [ ] `clv .version.guard` after install detects drift and restores preferred version if needed.

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.help`](../command/root.md#command-1-help) | Provides discovery of available commands |
| 2 | [`.version.show`](../command/version.md#command-3-versionshow) | Confirms installed version before and after install |
| 3 | [`.version.install`](../command/version.md#command-4-versioninstall) | Installs target version with 8-layer lock |
| 4 | [`.version.guard`](../command/version.md#command-5-versionguard) | Detects and restores preferred version on drift |
| 5 | [`.version.history`](../command/version.md#command-12-versionhistory) | Lists recent releases for version selection |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable output for scripting |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/01_output_control.md) | Controls verbosity and output format |
| 2 | [Execution Control](../param_group/02_execution_control.md) | Controls dry-run and force behavior |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`version::`](../param/01_version.md) | Specifies the target version or alias to install |
| 2 | [`dry::`](../param/02_dry.md) | Previews install plan without executing |
| 3 | [`force::`](../param/03_force.md) | Overrides idempotency check to reinstall |
| 4 | [`v::`](../param/04_v.md) | Controls diagnostic detail level |
| 5 | [`format::`](../param/05_format.md) | Selects text or JSON rendering |
| 6 | [`count::`](../param/09_count.md) | Limits number of history entries shown |
| 7 | [`.help`](../param/10_help.md) | Universal help override for any command |

### Workflow Steps

**Step 1 — Browse recent releases to select a target:**

```bash
clv .version.history count::5
# 1.2.34  2024-01-15  Fixes tool-call streaming edge case
# 1.2.33  2024-01-08  Improves context window utilization
# 1.2.32  2024-01-01  Adds prompt caching support
```

**Step 2 — Preview the install plan without executing:**

```bash
clv .version.install version::stable dry::1
# [dry-run] Would install claude-code@1.2.34
# [dry-run] autoUpdates = false   (version lock applied)
# [dry-run] purge cached binary at ~/.npm/_npx/.../claude
```

**Step 3 — Execute the install:**

```bash
clv .version.install version::stable
# Installing claude-code@1.2.34 ...
# Version lock applied (autoUpdates = false)
# Done.
```

**Step 4 — Confirm the new version is active:**

```bash
clv .version.show
# 1.2.34
```
