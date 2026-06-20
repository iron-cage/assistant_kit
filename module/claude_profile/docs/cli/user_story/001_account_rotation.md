# User Story :: 1. Automatic Account Rotation

**Persona:** SWE managing multiple Claude Max accounts across projects
**Goal:** Switch to a fresh account when the active token expires without manually copying credentials
**Benefit:** Maintains uninterrupted Claude Code sessions without credential management overhead
**Priority:** High

### Acceptance Criteria

- [ ] `clp .usage rotate::1` selects the best account using the active `sort::` strategy (default: `renew` — soonest quota renewal event)
- [ ] The switch is atomic — `~/.claude/.credentials.json` replaced via write-then-rename
- [ ] `dry::1` previews the selected account without switching: `clp .usage rotate::1 dry::1`
- [ ] `clp .account.use name::X` enables manual rotation when the target is already known
- [ ] Exit 1 when no eligible accounts are available; quota table still rendered

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage rotate::1`](../command/006_usage.md) | Primary: strategy-based selection and switch after quota table render |
| 2 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Secondary: manual switch to a known account |
| 3 | [`.accounts`](../command/001_account.md#command--3-accounts) | Supporting: verify account list before rotation |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`rotate::`](../param/059_rotate.md) | Trigger strategy-based rotation after quota table render |
| 2 | [`dry::`](../param/004_dry.md) | Preview rotation target without executing |
| 3 | [`sort::`](../param/025_sort.md) | Row ordering and footer recommendation: `renew` (default), `name`, `renews` |
| 4 | [`force::`](../param/058_force.md) | Bypass G5 ownership gate on rotation eligibility |
| 5 | [`touch::`](../param/034_touch.md) | Activate idle 5h window on target after switch |
| 6 | [`imodel::`](../param/035_imodel.md) | Model for post-switch activation subprocess |
| 7 | [`effort::`](../param/036_effort.md) | Effort for post-switch activation subprocess |
| 8 | [`trace::`](../param/023_trace.md) | Diagnose rotation selection and switch steps |
| 9 | [`refresh::`](../param/019_refresh.md) | Refresh expired token before refusing exit 3 |
| 10 | [`set_model::`](../param/054_set_model.md) | Set session model after switch |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | `touch::`, `refresh::`, `imodel::`, `effort::` govern post-switch activation |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [`text`](../format/001_text.md) | Default confirmation output for rotation commands |
