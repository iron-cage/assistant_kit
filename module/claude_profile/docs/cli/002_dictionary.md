# Dictionary

### Core Terms

**Account** — Email-identified credential set stored as `{credential_store}/{email}.credentials.json`. Each account is a snapshot of Claude OAuth credentials identified by the account's email address and can be restored by switching.

**Account Store** — The credential store directory (`$PRO/.persistent/claude/credential/` or `$HOME/.persistent/claude/credential/`) containing all named credential snapshots and the active marker file. Created automatically on first `save` operation.

**Active Account** — Account whose credentials are currently at `~/.claude/.credentials.json`, identified by the active marker. Only one account can be active at a time.

**Active Marker** — Per-machine text file at `{credential_store}/_active_{hostname}_{user}` (filename from `active_marker_filename()`) containing the name of the currently active account on this machine. Each machine writes its own marker, so switching on one machine never affects another. Updated during `switch` and `save` operations.

**Live Credentials** — The credential data currently used by Claude Code, residing at `~/.claude/.credentials.json`. This file exists on any authenticated machine regardless of whether account management has been initialized (credential store, active marker). Reading live credentials requires no account store setup and is the direct source for `.credentials.status`.

**Token Status** — Classification of the active OAuth access token: `Valid` (more than threshold remaining), `ExpiringSoon` (within warning threshold), or `Expired` (past `expiresAt`).

### Quota State Terms

**h-exhausted** — Account with `5h Left ≤ 15%`; the current 5-hour session window is nearly or fully consumed. May still have weekly quota. Within the 🟡 tier, h-exhausted accounts appear before weekly-exhausted accounts.

**weekly-exhausted** — Account with `7d Left ≤ 5%`; the rolling 7-day quota is nearly or fully consumed. May still have session (5h) quota available.

### Technical Terms

**Atomic Switch** — Write-then-rename pattern used during account switching. Credentials are written to a `.json.tmp` file adjacent to the target, then renamed into place, ensuring the credential file is never partially written.

**Credential File** — JSON file containing the `claudeAiOauth` object with fields: `accessToken`, `refreshToken`, `expiresAt`, `scopes`, `subscriptionType`, `rateLimitTier`.

**Token Expiry** — `expiresAt` field in a credential file — Unix epoch milliseconds after which the OAuth access token is invalid. Reflects OAuth lifecycle, not the server-side subscription usage window.

**Warning Threshold** — Seconds before token expiry at which the status transitions from `Valid` to `ExpiringSoon`. Default: `3600` (60 minutes). Configurable via `threshold::` parameter.
