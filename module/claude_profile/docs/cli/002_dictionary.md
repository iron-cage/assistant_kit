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

**h-exhausted** — Account with `5h Left ≤ 15%` and `7d Left > 5%`; the 5h session window is at or below threshold but the weekly quota is still available. Status group 2.

**weekly-exhausted** — Account with `7d Left ≤ 5%`; the 7d weekly quota is at or below threshold. Status group 3. Covers any `7d Left ≤ 5%` account, including accounts where both quotas are exhausted — the 7d constraint is binding in both cases. Fix(BUG-321): the former code classified the both-exhausted case (`5h ≤ 15%` AND `7d ≤ 5%`) as Dead (🔴); the correct classification is weekly-exhausted (🟡) since both-exhausted accounts recover without external action.

### Status Groups

**status group** — One of four fixed partitions applied to all accounts before any sort strategy runs. Group order is fixed and never reversed by `desc::`. Sorting reorders rows within each group only. Replaces the former "three-tier" terminology.

| Group | Emoji | Name | 5h | 7d | Recovery |
|-------|-------|------|----|----|----------|
| 1 | 🟢 | Green | available | available | — |
| 2 | 🟡 | h-exhausted | exhausted | available | Short-cycle (5h reset) |
| 3 | 🟡 | weekly-exhausted | any | exhausted | Long-cycle (7d reset) — includes both-exhausted |
| 4 | 🔴 | Dead | — | — | Error or cancelled (`billing_type="none"`) — not recoverable without external action |

Group 2 ranks above group 3 because 5h exhaustion recovers in hours; 7d exhaustion takes days. Group 3 (weekly-exhausted) ranks above group 4 (Dead) because it WILL recover — no external action needed. See [sort::](param/025_sort.md).

### Ownership and Assignment

**Owner** — The `USER@MACHINE` identity that holds persistent ownership of an account, stored in the `owner` field of `{name}.json`. Ownership is managed via `owner::USER@MACHINE` (set) and `owner::0` (release). The ownership gate (G8) blocks credential operations by non-owners. See [feature/036_account_ownership.md](../feature/036_account_ownership.md).

**Assignee** — The `USER@MACHINE` identity that holds a specific account as currently active on its machine, recorded in the `_active_{machine}_{user}` marker file. Assignment is managed via `assignee::USER@MACHINE name::X` (set), `assignee::0 name::X` (set for current machine), and `assignee::USER@MACHINE` / `assignee::0` without `name::` (clear). Assignment is marker-only and does not affect the `owner` field. See [feature/065_assignee_param_redesign.md](../feature/065_assignee_param_redesign.md).

### Operational Modes

**solo mode** — Token conservation mode activated by `solo::1` on `.usage`. Restricts all credential-consuming operations (HTTP quota fetch, account metadata fetch, refresh subprocess, touch subprocess) to the account that is both current AND owned. All other accounts display approximated historical data via the dedicated `approximate_quota()` function — no direct cache file reads permitted. Controls token consumption only; display filters (`only_active::`, `count::`, `offset::`, etc.) remain fully independent. Mutually exclusive with `rotate::1`. See [param/060_solo.md](param/060_solo.md).

### Technical Terms

**Atomic Switch** — Write-then-rename pattern used during account switching. Credentials are written to a `.json.tmp` file adjacent to the target, then renamed into place, ensuring the credential file is never partially written.

**Credential File** — JSON file containing the `claudeAiOauth` object with fields: `accessToken`, `refreshToken`, `expiresAt`, `scopes`, `subscriptionType`, `rateLimitTier`.

**Token Expiry** — `expiresAt` field in a credential file — Unix epoch milliseconds after which the OAuth access token is invalid. Reflects OAuth lifecycle, not the server-side subscription usage window.

**Warning Threshold** — Seconds before token expiry at which the status transitions from `Valid` to `ExpiringSoon`. Default: `3600` (60 minutes). Configurable via `threshold::` parameter.
