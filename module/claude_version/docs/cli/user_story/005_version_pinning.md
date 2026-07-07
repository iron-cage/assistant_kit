# Pin all team members to a shared Claude Code version

**Persona:** team lead
**Goal:** Check what version aliases resolve to, install the pinned team version, and verify alignment — without manual version string coordination.
**Benefit:** Ensures every team member runs the same Claude Code version, preventing behavior drift from silent auto-updates.
**Priority:** High

### Acceptance Criteria

- [ ] `clv .version.list` shows stable, month, and latest aliases with their resolved versions.
- [ ] `clv .version.install version::month dry::1` shows the install plan for the monthly baseline.
- [ ] `clv .version.install version::month` installs and applies 8-layer lock.
- [ ] If currently at the pinned version, install is a no-op (exits 0).
- [ ] `clv .version.show` after install confirms the pinned version is active.
- [ ] `clv .version.guard interval::N` watches for drift and restores the pinned version automatically.

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.help`](../command/root.md#command-1-help) | Provides discovery of available commands |
| 2 | [`.version.show`](../command/version.md#command-3-versionshow) | Verifies the active version post-install |
| 3 | [`.version.install`](../command/version.md#command-4-versioninstall) | Installs the pinned version with 8-layer lock |
| 4 | [`.version.guard`](../command/version.md#command-5-versionguard) | Watches for drift and restores the pinned version |
| 5 | [`.version.list`](../command/version.md#command-6-versionlist) | Shows alias-to-version resolution for selection |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable output for scripting |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/01_output_control.md) | Controls verbosity and output format |
| 2 | [Execution Control](../param_group/02_execution_control.md) | Controls dry-run and force install behavior |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`version::`](../param/01_version.md) | Specifies the version alias or semver to pin |
| 2 | [`dry::`](../param/02_dry.md) | Previews install plan without executing |
| 3 | [`force::`](../param/03_force.md) | Overrides idempotency check to reinstall |
| 4 | [`v::`](../param/04_v.md) | Controls diagnostic detail level |
| 5 | [`format::`](../param/05_format.md) | Selects text or JSON rendering |
| 6 | [`interval::`](../param/08_interval.md) | Sets watch loop interval for continuous drift detection |

### Workflow Steps

**Step 1 — Resolve available version aliases:**

```bash
clv .version.list
# stable   1.2.34
# month    1.2.30
# latest   1.2.34
```

**Step 2 — Preview the team-baseline install:**

```bash
clv .version.install version::month dry::1
# [dry-run] Would install claude-code@1.2.30
# [dry-run] autoUpdates = false   (version lock applied)
```

**Step 3 — Install and lock the pinned version:**

```bash
clv .version.install version::month
# Installing claude-code@1.2.30 ...
# Version lock applied (autoUpdates = false)
# Done.
```

**Step 4 — Start continuous drift detection:**

```bash
clv .version.guard interval::60
# Watching for drift every 60 s ...  (runs until interrupted)
```
