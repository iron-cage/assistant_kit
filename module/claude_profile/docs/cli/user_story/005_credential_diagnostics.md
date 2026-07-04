# User Story: 5. Credential Diagnostics

**Persona:** Developer troubleshooting authentication failures or verifying account configuration
**Goal:** Inspect live credential and environment state without modifying the account store
**Benefit:** Rapid root-cause analysis for authentication errors without manual file inspection
**Priority:** Medium

### Acceptance Criteria

- [ ] `clp .credentials.status` shows subscription, tier, token validity, and expiry without account store lookup
- [ ] `clp .token.status` classifies the active token as Valid / ExpiringSoon / Expired with exact duration
- [ ] `clp .paths` resolves all canonical `~/.claude/` file paths on the current machine
- [ ] `clp .account.inspect trace::1` shows live endpoint responses and membership selection priority

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Primary: live credential metadata without store dependency |
| 2 | [`.token.status`](../command/005_token.md#command-7-tokenstatus) | Token expiry classification with configurable threshold |
| 3 | [`.paths`](../command/004_paths.md#command-8-paths) | Canonical file path resolution |
| 4 | [`.account.inspect`](../command/001_account.md#command-15-accountinspect) | Deep live diagnostic with endpoint trace |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`trace::`](../param/023_trace.md) | Verbose diagnostic trace to stderr |
| 2 | [`format::`](../param/002_format.md) | JSON for structured diagnostic comparison |
| 3 | [`field::`](../param/024_field.md) | Target `.account.inspect` to a specific path subtree |
| 4 | [`threshold::`](../param/003_threshold.md) | Tune Valid / ExpiringSoon classification boundary |
| 5 | [`refresh::`](../param/019_refresh.md) | Attempt token refresh before inspect endpoint calls |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `trace::`, `refresh::` for diagnostic depth |
| 2 | [Output Control](../param_group/001_output_control.md) | `format::json` for structured diagnostic output |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [`text`](../format/001_text.md) | Default human-readable diagnostic output |
| 2 | [`json`](../format/002_json.md) | Structured output for comparing diagnostic state |
