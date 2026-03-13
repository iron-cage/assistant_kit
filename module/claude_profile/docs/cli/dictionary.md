# Dictionary

## Core Terms

**Account** — Named credential set stored as `~/.claude/accounts/{name}.credentials.json`. Each account is a snapshot of Claude OAuth credentials that can be restored by switching.

**Account Store** — `~/.claude/accounts/` directory containing all named credential snapshots and the active marker file. Created automatically on first `save` operation.

**Active Account** — Account whose credentials are currently at `~/.claude/.credentials.json`, identified by the active marker. Only one account can be active at a time.

**Active Marker** — `~/.claude/accounts/_active` text file containing the name of the currently active account. Updated atomically during `switch` operations.

**Live Credentials** — The credential data currently used by Claude Code, residing at `~/.claude/.credentials.json`. This file exists on any authenticated machine regardless of whether account management has been initialized (`accounts/` directory, `_active` marker). Reading live credentials requires no account store setup and is the direct source for `.credentials.status`.

**Token Status** — Classification of the active OAuth access token: `Valid` (more than threshold remaining), `ExpiringSoon` (within warning threshold), or `Expired` (past `expiresAt`).

## Technical Terms

**Atomic Switch** — Write-then-rename pattern used during account switching. Credentials are written to a `.json.tmp` file adjacent to the target, then renamed into place, ensuring the credential file is never partially written.

**Credential File** — JSON file containing the `claudeAiOauth` object with fields: `accessToken`, `refreshToken`, `expiresAt`, `scopes`, `subscriptionType`, `rateLimitTier`.

**Token Expiry** — `expiresAt` field in a credential file — Unix epoch milliseconds after which the OAuth access token is invalid. Reflects OAuth lifecycle, not the server-side subscription usage window.

**Warning Threshold** — Seconds before token expiry at which the status transitions from `Valid` to `ExpiringSoon`. Default: `3600` (60 minutes). Configurable via `threshold::` parameter.
