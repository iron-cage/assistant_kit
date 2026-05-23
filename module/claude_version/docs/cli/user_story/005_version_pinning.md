# User Story :: 005. Version Pinning

### Scope

- **Purpose**: Align a team to a shared, reproducible Claude Code version.
- **Responsibility**: Persona, goal, and acceptance criteria for team-wide version pinning.

### Persona

Team lead who needs every team member running the same Claude Code version for reproducibility, preventing behavior drift from auto-updates.

### Goal

Check what version aliases resolve to, install the pinned team version, and verify alignment — without manual version string coordination.

### Acceptance Criteria

- `cm .version.list` shows stable, month, and latest aliases with their resolved versions.
- `cm .version.install version::month dry::1` shows the install plan for the monthly baseline.
- `cm .version.install version::month` installs and applies 5-layer lock.
- If currently at the pinned version, install is a no-op (exits 0).
- `cm .version.show` after install confirms the pinned version is active.
- `cm .version.guard interval::N` can watch for drift and restore the pinned version automatically.

### Referenced Commands

| # | Command |
|---|---------|
| 1 | [`.version.list`](../command/version.md#command--6-versionlist) |
| 2 | [`.version.install`](../command/version.md#command--4-versioninstall) |
| 3 | [`.version.show`](../command/version.md#command--3-versionshow) |
| 4 | [`.version.guard`](../command/version.md#command--5-versionguard) |
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
| 4 | [`interval::`](../param/08_interval.md) |
| 5 | [`v::`](../param/04_v.md) |
| 6 | [`format::`](../param/05_format.md) |
