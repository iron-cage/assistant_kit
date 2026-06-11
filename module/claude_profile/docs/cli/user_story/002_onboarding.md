# User Story :: 2. Account Onboarding and Lifecycle Management

**Persona:** Developer setting up Claude Code or managing saved account profiles
**Goal:** Persist, update, and remove named credential profiles without touching raw credential files
**Benefit:** Enables multi-account workflows with safe, traceable credential management
**Priority:** High

### Acceptance Criteria

- [ ] `clp .account.save` captures `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json`
- [ ] Name auto-inferred from `oauthAccount.emailAddress` when omitted; falls back to active marker; exits 1 if neither present
- [ ] `host::` and `role::` metadata captured in `{name}.json`; `dry::1` previews without writing
- [ ] `clp .account.delete` removes `.credentials.json` + `.json` + legacy satellite files
- [ ] `clp .account.relogin` spawns `claude` with TTY; propagates fresh credentials to store on exit
- [ ] `clp .account.renewal` sets `_renewal_at` in `{name}.json`; accepted by single name, list, or `all`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command--4-accountsave) | Primary: capture credentials as named profile |
| 2 | [`.accounts`](../command/001_account.md#command--3-accounts) | Verify: confirm profile appears in store |
| 3 | [`.account.delete`](../command/001_account.md#command--6-accountdelete) | Cleanup: remove stale or incorrect profiles |
| 4 | [`.account.relogin`](../command/001_account.md#command--12-accountrelogin) | Recovery: browser re-auth when refresh token is dead |
| 5 | [`.account.renewal`](../command/001_account.md#command--14-accountrenewal) | Management: set billing renewal timestamp override |
| 6 | [`.account.assign`](../command/001_account.md#command--16-accountassign) | Cross-machine: write active-account marker for another host |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`name::`](../param/001_name.md) | Identifies the target account profile |
| 2 | [`host::`](../param/048_host.md) | Machine label stored in `{name}.json` at save time |
| 3 | [`role::`](../param/052_role.md) | User-defined role label stored in `{name}.json` |
| 4 | [`dry::`](../param/004_dry.md) | Preview save/delete/renewal without side effects |
| 5 | [`at::`](../param/049_at.md) | Absolute renewal timestamp for `.account.renewal` |
| 6 | [`from_now::`](../param/050_from_now.md) | Delta-from-now renewal timestamp |
| 7 | [`clear::`](../param/051_clear.md) | Remove `_renewal_at` override from `{name}.json` |
| 8 | [`for::`](../param/053_for.md) | Target `USER@MACHINE` for `.account.assign` |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Account Targeting](../param_group/006_account_targeting.md) | `host::` and `role::` metadata at save time |

### Referenced Formats

| # | Format | Role |
|---|--------|------|
| 1 | [`text`](../format/001_text.md) | Default confirmation output for all lifecycle commands |
