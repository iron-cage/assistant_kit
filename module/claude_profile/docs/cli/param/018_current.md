# Parameter: 18. `current::`

**Removed.** `current::` is fully removed — `.accounts` (the only command that ever accepted it) now rejects it with `parameter 'current' removed — use 'cols::-current' instead` (see `REMOVED_TOGGLES` in `src/commands/accounts.rs`; [Feature 037](../../feature/037_accounts_usage_param_unification.md)). Unlike sibling field-presence params (`sub::`, `tier::`, etc.), `current::` has no other host command — `.credentials.status` never supported it — so there is no working fallback for this parameter.

Historically, `current::` controlled whether the current (live) account line appeared in `.accounts` output for each account entry. The current account is the saved account whose `accessToken` matches the live `~/.claude/.credentials.json` file — distinct from the active account (per-machine active marker). See [feature/016_current_account_awareness.md](../../feature/016_current_account_awareness.md).

- **Default:** `1` (shown)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; the line is always suppressed when `~/.claude/.credentials.json` is absent or unreadable regardless of the toggle value
- **Purpose:** Indicates which saved account corresponds to the credentials currently loaded by Claude Code. When current ≠ active (divergence), both `Active:  yes` and `Current: no` appear on the active account row, and `Active:  no` / `Current: yes` appear on the current account row.

**Examples:**

```text
current::1   → Current: yes  (default; or "no" for accounts not matching live token)
current::0   → line omitted
```

**Notes:**
- When `~/.claude/.credentials.json` is unreadable, the `Current:` line is suppressed for all accounts (equivalent to `current::0`). This prevents misleading `Current: no` output when the live token cannot be determined.
- `format::json` always includes `is_current` per account object regardless of this toggle.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

Not a member of any parameter group. [Field Presence](../param_group/002_field_presence.md) explicitly excludes `current` from its 16-member list (`.accounts` uses `cols::` for this field instead).

### Referenced Commands

None. `.accounts` rejects `current::` (see removal note above); no other command ever supported it.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Identify which account matches live credentials |
| 2 | [Account Onboarding](../user_story/002_onboarding.md) | Current vs active account visibility during management |
