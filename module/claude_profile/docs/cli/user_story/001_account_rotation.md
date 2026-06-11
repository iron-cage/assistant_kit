# User Story :: 1. Automatic Account Rotation

**Persona:** SWE managing multiple Claude Max accounts across projects
**Goal:** Switch to a fresh account when the active token expires without manually copying credentials
**Benefit:** Maintains uninterrupted Claude Code sessions without credential management overhead
**Priority:** High

### Acceptance Criteria

- [ ] `clp .account.rotate` automatically selects the inactive account with the highest remaining expiry
- [ ] The switch is atomic — `~/.claude/.credentials.json` replaced via write-then-rename
- [ ] `dry::1` previews the selected account without switching
- [ ] `clp .account.use name::X` enables manual rotation when the target is already known
- [ ] Exit 2 when no inactive accounts are available

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.rotate`](../command/001_account.md#command--13-accountrotate) | Primary: automatic selection and atomic switch |
| 2 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Secondary: manual switch to a known account |
| 3 | [`.accounts`](../command/001_account.md#command--3-accounts) | Supporting: verify account list before rotation |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`dry::`](../param/004_dry.md) | Preview rotation target without executing |
| 2 | [`touch::`](../param/034_touch.md) | Activate idle 5h window on target after switch |
| 3 | [`imodel::`](../param/035_imodel.md) | Model for post-switch activation subprocess |
| 4 | [`effort::`](../param/036_effort.md) | Effort for post-switch activation subprocess |
| 5 | [`trace::`](../param/023_trace.md) | Diagnose rotation selection and switch steps |
| 6 | [`refresh::`](../param/019_refresh.md) | Refresh expired token before refusing exit 3 |
| 7 | [`set_model::`](../param/054_set_model.md) | Set session model after switch |
| 8 | [`active::`](../param/013_active.md) | Show active/inactive status in account list |
| 9 | [`current::`](../param/018_current.md) | Show which account is currently live |
| 10 | [`expires::`](../param/009_expires.md) | Show token expiry to decide rotation timing |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `touch::`, `refresh::`, `imodel::`, `effort::` govern post-switch activation |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [`text`](../format/001_text.md) | Default confirmation output for rotation commands |
