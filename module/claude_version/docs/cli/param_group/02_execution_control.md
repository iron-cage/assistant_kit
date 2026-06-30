# Group :: 2. Execution Control

-- **Summary:** Parameters that control whether and how mutation commands execute.
-- **Parameters:** `dry::`, `force::`
-- **Coherence Test:** "Does this parameter control mutation execution mode?"

Both parameters modify the execution mode of destructive operations.

**Parameters:**

| Parameter | Type | Purpose |
|-----------|------|---------|
| [`dry::`](../param/02_dry.md) | bool | Preview without executing |
| [`force::`](../param/03_force.md) | bool | Bypass safety guards |

**Partial implementors:** `.settings.set` and `.config` implement `dry::` only (no `force::`).

**Why NOT in this group:**
- `version::`: specifies *what* to install, not *whether* to install
- `v::`: controls display, not execution
- `interval::`: controls guard *frequency*, not execution mode

**Typical usage:**

```sh
clv .version.install dry::1          # preview
clv .version.install force::1        # bypass idempotency
clv .version.guard dry::1 force::1   # preview forced guard
clv .processes.kill dry::1 force::1  # preview forced kill
```

### Referenced Commands

| # | Command | Membership | Excluded Params |
|---|---------|-----------|----------------|
| 1 | [`.version.install`](../command/version.md#command--4-versioninstall) | Full | — |
| 2 | [`.version.guard`](../command/version.md#command--5-versionguard) | Full | — |
| 3 | [`.processes.kill`](../command/processes.md#command--8-processeskill) | Full | — |
| 4 | [`.settings.set`](../command/settings.md#command--11-settingsset) | Partial | `force::` |
| 5 | [`.config`](../command/config.md#command--13-config) | Partial | `force::` |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 1 | [`dry::`](../param/02_dry.md) | bool | false | Preview without executing |
| 2 | [`force::`](../param/03_force.md) | bool | false | Bypass safety guards |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 3 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 4 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
| 5 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
