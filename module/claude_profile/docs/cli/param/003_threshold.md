# Parameter :: 3. `threshold::`

Overrides the default 60-minute warning window for token expiry classification. Tokens expiring within `threshold::` seconds are classified as `ExpiringSoon` instead of `Valid`.

- **Default:** `3600` (60 minutes, matching `token::WARNING_THRESHOLD_SECS`)
- **Constraints:** Non-negative integer (seconds)
- **Purpose:** Allows callers to tune the early-warning sensitivity — automation scripts may want `threshold::7200` (2 hours) for proactive rotation, while interactive users may prefer the default 60 minutes.

**Examples:**

```text
threshold::3600   → classify as ExpiringSoon when <=60 minutes remain (default)
threshold::1800   → classify as ExpiringSoon when <=30 minutes remain
threshold::7200   → classify as ExpiringSoon when <=2 hours remain
threshold::0      → never classify as ExpiringSoon (only Valid or Expired)
```

### Referenced Type

| # | Type | Role |
|---|------|------|
| 1 | [WarningThreshold](../type/003_warning_threshold.md) | Value type |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.token.status`](../command/005_token.md#command--7-tokenstatus) | Applies warning threshold for expiry classification |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Tunable expiry warning window for diagnostics |
