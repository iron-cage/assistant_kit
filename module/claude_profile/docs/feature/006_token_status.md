# Feature: Token Status

### Scope

- **Purpose**: Classify the active OAuth token's validity to enable proactive account rotation before operations fail.
- **Responsibility**: Documents the `token::status()` API and `.token.status` CLI command (FR-11).
- **In Scope**: Token expiry classification (Valid/ExpiringSoon/Expired), custom threshold, output formats.
- **Out of Scope**: OAuth refresh (forbidden — NFR-5), account rotation logic (→ 008_auto_rotate.md).

### Design

`claude_profile` must read `expiresAt` from `~/.claude/.credentials.json` and return one of:

| Status | Condition |
|--------|-----------|
| `Valid` | `expiresAt` is in the future and more than `threshold` seconds away |
| `ExpiringSoon` | `expiresAt` is in the future but within `threshold` seconds |
| `Expired` | `expiresAt` is in the past (now ≥ expiresAt) |

**Default threshold:** 3600 seconds (60 minutes), matching `token::WARNING_THRESHOLD_SECS`.

**Custom threshold:** `status_with_threshold(threshold_secs: u64)` accepts caller-specified seconds. CLI exposes this via `threshold::` parameter.

**Important:** `expiresAt` reflects the **OAuth access token** expiry — typically auto-refreshed by Claude Code. It does NOT reflect the server-side 5-hour subscription usage window, which is not locally observable.

**Exit codes:**
- 0: success
- 2: credentials file unreadable or `expiresAt` field missing/unparseable

### Acceptance Criteria

- **AC-01**: Token with `expiresAt` > now + 3600s → `Valid`; token with `expiresAt` < now → `Expired`.
- **AC-02**: Token with `expiresAt` within threshold → `ExpiringSoon`.
- **AC-03**: `threshold::1800` changes the classification boundary to 30 minutes.
- **AC-04**: `format::json` returns `{"status":"valid","expires_in_secs":N}`.

### Commands

| File | Relationship |
|------|--------------|
| [command/005_token.md](../cli/command/005_token.md#command-7-tokenstatus) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [008_auto_rotate.md](008_auto_rotate.md) | Consumes token status to detect when rotation is needed |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.token.status`](../cli/command/005_token.md#command-7-tokenstatus) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/token.rs` | `status()`, `status_with_threshold()`, `TokenStatus` enum |
| `src/commands/token_paths.rs` | `token_status_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| `tests/token_tests.rs` | Valid/ExpiringSoon/Expired classification tests |
