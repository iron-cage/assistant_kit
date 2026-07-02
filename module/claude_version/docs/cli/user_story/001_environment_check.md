# Verify environment state at a glance

**Persona:** developer
**Goal:** Run one command to see installed version, active session count, and active account — without digging into files or running multiple commands.
**Benefit:** Confirms Claude Code is installed and functional in seconds after a system change or on a new machine.
**Priority:** High

### Acceptance Criteria

- [ ] `clv .status` outputs version, session count, and active account in a single view.
- [ ] `clv .status format::json` returns the same fields as a JSON object for scripting.
- [ ] `clv .status v::2` shows additional diagnostic context.
- [ ] Missing HOME exits 2; all other failures also exit 2.

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.help`](../command/root.md#command--1-help) | Provides discovery of available commands |
| 2 | [`.status`](../command/root.md#command--2-status) | Delivers the unified environment view |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [text](../format/01_text.md) | Default human-readable output |
| 2 | [json](../format/02_json.md) | Machine-readable output for scripting |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/01_output_control.md) | Controls verbosity and format of status output |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`v::`](../param/04_v.md) | Controls diagnostic detail level |
| 2 | [`format::`](../param/05_format.md) | Selects text or JSON rendering |

### Workflow Steps

**Step 1 — Check environment state:**

```bash
clv .status
# Version:   2.1.78
# Processes: 2
# Account:   user@example.com
```

**Step 2 — Get machine-readable output for scripting:**

```bash
clv .status format::json
# {"version":"2.1.78","processes":2,"account":"user@example.com","preferred":{"spec":"stable","resolved":"2.1.78"}}
```

**Step 3 — Show additional diagnostic context:**

```bash
clv .status v::2
# Version:   2.1.78
# Processes: 2
# Account:   user@example.com
# Preferred: stable (v2.1.78)  (settings.json → preferredVersionSpec)
```
