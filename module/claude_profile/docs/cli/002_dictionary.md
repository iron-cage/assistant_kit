# Dictionary

### Core Terms

**Account** — Email-identified credential set stored as `{credential_store}/{email}.credentials.json`. Each account is a snapshot of Claude OAuth credentials identified by the account's email address and can be restored by switching.

**Account Store** — The credential store directory (`$PRO/.persistent/claude/credential/` or `$HOME/.persistent/claude/credential/`) containing all named credential snapshots and the active marker file. Created automatically on first `save` operation.

**Active Account** — Account whose credentials are currently at `~/.claude/.credentials.json`, identified by the active marker. Only one account can be active at a time.

**Active Marker** — Per-machine text file at `{credential_store}/_active_{hostname}_{user}` (filename from `active_marker_filename()`) containing the name of the currently active account on this machine. Each machine writes its own marker, so switching on one machine never affects another. Updated during `switch` and `save` operations.

**Live Credentials** — The credential data currently used by Claude Code, residing at `~/.claude/.credentials.json`. This file exists on any authenticated machine regardless of whether account management has been initialized (credential store, active marker). Reading live credentials requires no account store setup and is the direct source for `.credentials.status`.

**Token Status** — Classification of the active OAuth access token: `Valid` (more than threshold remaining), `ExpiringSoon` (within warning threshold), or `Expired` (past `expiresAt`).

### Quota Dimensions

**5h quota** — The 5-hour sliding session usage window. Column header: `5h Left`. Exhaustion threshold: `≤ 15%`. Resets on a short cycle (hours). Canonical adjective for below-threshold: **h-exhausted**.

**7d quota** — The 7-day rolling weekly usage quota. Column header: `7d Left`. Exhaustion threshold: `≤ 5%`. Resets on a long cycle (days). Canonical adjective for below-threshold: **weekly-exhausted**.

### Quota Status

**available** — A quota dimension above its exhaustion threshold (usable).

**exhausted** — A quota dimension at or below its exhaustion threshold (unusable until reset).

**h-exhausted** — Account with `5h Left ≤ 15%`; the 5h session window is at or below threshold. Status group 2 when 7d is still available; status group 4 when both dimensions are exhausted.

**weekly-exhausted** — Account with `7d Left ≤ 5%`; the 7d weekly quota is at or below threshold. Status group 3 when 5h is still available; status group 4 when both dimensions are exhausted.

### Status Groups

**status group** — One of four fixed partitions applied to all accounts before any sort strategy runs. Group order is fixed and never reversed by `desc::`. Sorting reorders rows within each group only. Replaces the former "three-tier" terminology.

| Group | Emoji | Name | 5h | 7d | Recovery |
|-------|-------|------|----|----|----------|
| 1 | 🟢 | Green | available | available | — |
| 2 | 🟡 | h-exhausted | exhausted | available | Short-cycle (5h reset) |
| 3 | 🟡 | weekly-exhausted | available | exhausted | Long-cycle (7d reset) |
| 4 | 🔴 | Red | — | — | Fully exhausted or error |

Group 2 ranks above group 3 because 5h exhaustion recovers in hours; 7d exhaustion takes days. See [sort::](param/025_sort.md).

### Technical Terms

**Atomic Switch** — Write-then-rename pattern used during account switching. Credentials are written to a `.json.tmp` file adjacent to the target, then renamed into place, ensuring the credential file is never partially written.

**Credential File** — JSON file containing the `claudeAiOauth` object with fields: `accessToken`, `refreshToken`, `expiresAt`, `scopes`, `subscriptionType`, `rateLimitTier`.

**Token Expiry** — `expiresAt` field in a credential file — Unix epoch milliseconds after which the OAuth access token is invalid. Reflects OAuth lifecycle, not the server-side subscription usage window.

**Warning Threshold** — Seconds before token expiry at which the status transitions from `Valid` to `ExpiringSoon`. Default: `3600` (60 minutes). Configurable via `threshold::` parameter.
