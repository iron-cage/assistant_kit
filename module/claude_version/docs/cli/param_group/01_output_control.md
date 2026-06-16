# Group :: 1. Output Control

-- **Summary:** Parameters that control how command output appears.
-- **Parameters:** `v::`, `format::`, `count::`
-- **Coherence Test:** "Does this parameter control output appearance?"

Control how command output appears. These parameters affect display without
changing behavior.

**Parameters:**

| Parameter | Type | Purpose |
|-----------|------|---------|
| [`v::`](../param/04_v.md) | [`VerbosityLevel`](../type/01_verbosity_level.md) | Detail level (0=minimal, 1=normal, 2=verbose) |
| [`format::`](../param/05_format.md) | [`OutputFormat`](../type/02_output_format.md) | Display encoding (text or json) |
| [`count::`](../param/09_count.md) | u64 | Entry limit (`.version.history` only; default 10) |

**Why NOT in this group:**
- `dry::`: controls execution, not display
- `key::`: identifies what to read, not how to display

**Typical usage:**

```sh
clv .status v::0 format::json
clv .processes format::json v::2
```

### Referenced Commands

| # | Command | Membership |
|---|---------|-----------|
| 1 | [`.status`](../command/root.md#command--2-status) | Partial (`v::`, `format::`) |
| 2 | [`.version.show`](../command/version.md#command--3-versionshow) | Partial (`v::`, `format::`) |
| 3 | [`.version.install`](../command/version.md#command--4-versioninstall) | Partial (`v::`, `format::`) |
| 4 | [`.version.guard`](../command/version.md#command--5-versionguard) | Partial (`v::`, `format::`) |
| 5 | [`.version.list`](../command/version.md#command--6-versionlist) | Partial (`v::`, `format::`) |
| 6 | [`.version.history`](../command/version.md#command--12-versionhistory) | Full (`v::`, `format::`, `count::`) |
| 7 | [`.processes`](../command/processes.md#command--7-processes) | Partial (`v::`, `format::`) |
| 8 | [`.processes.kill`](../command/processes.md#command--8-processeskill) | Partial (`v::`, `format::`) |
| 9 | [`.settings.show`](../command/settings.md#command--9-settingsshow) | Partial (`v::`, `format::`) |
| 10 | [`.settings.get`](../command/settings.md#command--10-settingsget) | Partial (`v::`, `format::`) |
| 11 | [`.config`](../command/config.md#command--13-config) | Partial (`v::`, `format::`) |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [001 Environment Check](../user_story/001_environment_check.md) | Developer (new machine setup) |
| 2 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 3 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 4 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 5 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
| 6 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
