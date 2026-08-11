# Feature: Token Status

### Scope

- **Purpose**: Classify the active OAuth token's validity to enable proactive account rotation before operations fail.
- **Responsibility**: Documents the `token::status()` API, exposed via `.credentials.status`'s `token`/`expires` fields (FR-11); the `static` classification for redirect-backend accounts (see [071_redirect_backend_accounts.md](071_redirect_backend_accounts.md)).
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

**Custom threshold:** `status_with_threshold(threshold_secs: u64)` accepts caller-specified seconds. CLI exposes this via `.credentials.status`'s `threshold::` parameter.

**Important:** `expiresAt` reflects the **OAuth access token** expiry — typically auto-refreshed by Claude Code. It does NOT reflect the server-side 5-hour subscription usage window, which is not locally observable.

**Error handling:** `status()`/`status_with_threshold()` return `Result<TokenStatus, io::Error>` — `Err` when `expiresAt` is missing or unparseable. `.credentials.status` (the only CLI surface) does not propagate this as a command failure: `derive_token_state()` degrades gracefully, rendering `Token: unknown` / `Expires: (unavailable)` while every other field (subscription, tier, email, etc.) still renders normally and the command exits 0. Only a missing `~/.claude/.credentials.json` file itself causes `.credentials.status` to exit 2 (see [command/002_credentials.md](../cli/command/002_credentials.md#command-10-credentialsstatus)).

**`static` classification (redirect-backend accounts):** The CLI-layer caller (`credentials_status_routine()` in `src/commands/credentials.rs`) checks the active account's `backend` — equivalently, the genuine absence of `expiresAt` in its credentials file — *before* calling `status_with_threshold()` at all. A `backend: redirect` account never reaches `status()`/`status_with_threshold()`: the caller constructs `TokenStatus::Static` directly and skips the threshold call entirely, so no expiry probe runs since there is no OAuth session to expire. `Static` is terminal-stable: no transition moves an account into or out of it except re-running `.account.save`. See [071_redirect_backend_accounts.md](071_redirect_backend_accounts.md).

### Acceptance Criteria

- **AC-01**: Token with `expiresAt` > now + 3600s → `Valid`; token with `expiresAt` < now → `Expired`.
- **AC-02**: Token with `expiresAt` within threshold → `ExpiringSoon`.
- **AC-03**: `.credentials.status threshold::1800` changes the classification boundary to 30 minutes.
- **AC-04**: `.credentials.status format::json` includes `"token"` (status label) and `"expires_in_secs"` (integer) fields reflecting this classification.
- **AC-05**: A `backend: redirect` account (equivalently, any credentials file with `expiresAt` genuinely absent) → `Static` classification, checked before Valid/ExpiringSoon/Expired threshold math; `.credentials.status` renders `Token: static`.

### Commands

| File | Relationship |
|------|--------------|
| [command/002_credentials.md](../cli/command/002_credentials.md#command-10-credentialsstatus) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [008_auto_rotate.md](008_auto_rotate.md) | Consumes token status to detect when rotation is needed |
| [071_redirect_backend_accounts.md](071_redirect_backend_accounts.md) | Adds the `Static` classification for redirect-backend accounts |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../cli/command/002_credentials.md#command-10-credentialsstatus) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/token.rs` | `status()`, `status_with_threshold()`, `TokenStatus` enum — new `Static` variant, constructed by the CLI-layer caller (not by `status()`/`status_with_threshold()` themselves) before any threshold math runs |
| `src/commands/credentials.rs` | `credentials_status_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| `tests/token_tests.rs` | Valid/ExpiringSoon/Expired classification tests |
