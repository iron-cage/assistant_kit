# Group :: 2. Execution Control

-- **Summary:** Parameters that control whether and how mutation commands execute.
-- **Parameters:** `dry::`, `force::`, `record_only::`
-- **Coherence Test:** "Does this parameter control mutation execution mode?"

All three parameters modify the execution mode of destructive operations.

**Parameters:**

| Parameter | Type | Purpose |
|-----------|------|---------|
| [`dry::`](../param/02_dry.md) | bool | Preview without executing |
| [`force::`](../param/03_force.md) | bool | Bypass safety guards |
| [`record_only::`](../param/15_record_only.md) | bool | Persist preference without installing |

**Partial implementors:** `.settings.set`, `.config`, and `.version.mark` implement `dry::` only (no `force::` or `record_only::`). `.version.guard` and `.ps.kill` implement `dry::`/`force::` but not `record_only::` (install-only concept — see `15_record_only.md`).

**Why NOT in this group:**
- `version::`: specifies *what* to install, not *whether* to install
- `v::`: controls display, not execution
- `interval::`: controls guard *frequency*, not execution mode

**Typical usage:**

```sh
clv .version.install dry::1          # preview
clv .version.install force::1        # bypass idempotency
clv .version.install record_only::1  # persist preference, no install
clv .version.guard dry::1 force::1   # preview forced guard
clv .ps.kill dry::1 force::1  # preview forced kill
```

### Referenced Commands

| # | Command | Membership | Excluded Params |
|---|---------|-----------|----------------|
| 1 | [`.version.install`](../command/version.md#command-4-versioninstall) | Full | — |
| 2 | [`.version.guard`](../command/version.md#command-5-versionguard) | Partial | `record_only::` |
| 3 | [`.ps.kill`](../command/ps.md#command-8-pskill) | Partial | `record_only::` |
| 4 | [`.settings.set`](../command/settings.md#command-11-settingsset) | Partial | `force::`, `record_only::` |
| 5 | [`.config`](../command/config.md#command-13-config) | Partial | `force::`, `record_only::` |
| 6 | [`.version.mark`](../command/version.md#command-17-versionmark) | Partial | `force::`, `record_only::` |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 1 | [`dry::`](../param/02_dry.md) | bool | false | Preview without executing |
| 2 | [`force::`](../param/03_force.md) | bool | false | Bypass safety guards |
| 3 | [`record_only::`](../param/15_record_only.md) | bool | false | Persist preference without installing |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [002 Version Upgrade](../user_story/002_version_upgrade.md) | Developer (version upgrade) |
| 2 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
| 3 | [004 Settings Management](../user_story/004_settings_management.md) | Developer (settings management) |
| 4 | [005 Version Pinning](../user_story/005_version_pinning.md) | Team lead (version pinning) |
| 5 | [006 Config Management](../user_story/006_config_management.md) | Developer (config management) |
