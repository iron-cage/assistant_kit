# User Story :: 002. Version Upgrade

### Scope

- **Purpose**: Safely upgrade Claude Code to a target version.
- **Responsibility**: Persona, goal, and acceptance criteria for the version upgrade workflow.

### Persona

Developer who wants to upgrade (or downgrade) Claude Code to a specific version without risking an unintended install or breaking auto-update settings.

### Goal

Preview exactly what will happen, then install the target version with version locking applied — and verify the result.

### Acceptance Criteria

- `cm .version.install version::X dry::1` shows the full install plan without executing.
- `cm .version.install version::X` installs, applies 5-layer version lock, and exits 0.
- Already-at-target is a no-op (exits 0) unless `force::1` is set.
- `cm .version.show` after install prints the newly installed version.
- `cm .version.history` shows recent releases with summaries to aid version selection.
- `cm .version.guard` after install detects drift and restores preferred version if needed.

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.version.show`](../command/version.md#command--3-versionshow) |
| 2 | [`.version.install`](../command/version.md#command--4-versioninstall) |
| 3 | [`.version.guard`](../command/version.md#command--5-versionguard) |
| 4 | [`.version.history`](../command/version.md#command--12-versionhistory) |
| 5 | [`.help`](../command/root.md#command--1-help) |

### Referenced Formats

| # | Format |
|---|--------|
| 1 | [text](../format/01_text.md) |
| 2 | [json](../format/02_json.md) |

### Referenced Parameter Groups

| # | Group |
|---|-------|
| 1 | [Execution Control](../param_group/02_execution_control.md) |
| 2 | [Output Control](../param_group/01_output_control.md) |

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 1 | [`version::`](../param/01_version.md) |
| 2 | [`dry::`](../param/02_dry.md) |
| 3 | [`force::`](../param/03_force.md) |
| 4 | [`v::`](../param/04_v.md) |
| 5 | [`format::`](../param/05_format.md) |
| 6 | [`count::`](../param/09_count.md) |
