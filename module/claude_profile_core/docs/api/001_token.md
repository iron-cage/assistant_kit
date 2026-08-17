# API: Token

### Scope

- **Purpose**: Document the programmatic interface of the claude_profile_core `token` module.
- **Responsibility**: Specify the OAuth access-token expiry classification contract.
- **In Scope**: `TokenStatus`, `WARNING_THRESHOLD_SECS`, `status`, `status_with_threshold`, `classify_ms`, `parse_expires_at`.
- **Out of Scope**: Token refresh (→ `account::refresh_account_token`, `002_account.md`), credential file writes (→ `account`, `002_account.md`).

### Abstract

`claude_profile_core::token` reads the active OAuth access token from `~/.claude/.credentials.json` and classifies it against a warning threshold. Pure read-side: it never mutates the credential file. The classification primitive (`classify_ms`) is separated from the file read (`status*`) so callers holding an already-parsed `expiresAt` (e.g. per-account status listings over the credential store) can classify without touching disk.

### Operations

#### `TokenStatus`

Enum classifying the active token: `Valid { expires_in }` (more than the warning threshold remaining), `ExpiringSoon { expires_in }` (within the threshold), `Expired`, `Static` (redirect-backend API-key account, Feature 071 — never expires).

#### `WARNING_THRESHOLD_SECS`

Default `ExpiringSoon` threshold: `3600` (60 minutes).

#### `status() -> Result<TokenStatus, io::Error>`

Reads the active token from `~/.claude/.credentials.json` and classifies it using `WARNING_THRESHOLD_SECS`. Errors if `HOME` is unset, the file is missing/unreadable, or `expiresAt` cannot be parsed.

#### `status_with_threshold(warning_secs: u64) -> Result<TokenStatus, io::Error>`

Same read as `status`, with a caller-supplied warning threshold in seconds.

#### `classify_ms(expires_at_ms: u64, warning_secs: u64) -> TokenStatus`

Pure classification of a millisecond epoch expiry against the current time and a threshold — no I/O.

#### `parse_expires_at(json: &str) -> Option<u64>`

Extracts the numeric `expiresAt` field (millisecond epoch) from raw credential JSON. The single authority for this parse — CLI-side callers delegate here rather than needle-parsing the JSON themselves (Fix(audit-inline-expiry-parse)).

### Error Handling

`status`/`status_with_threshold` return `std::io::Error` (`NotFound` for missing `HOME`/file, `InvalidData` for an unparseable `expiresAt`). `classify_ms` and `parse_expires_at` are infallible/`Option`-returning.

### Sources

| File | Relationship |
|------|--------------|
| `../../src/token.rs` | `TokenStatus`, all operations |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/token_test.rs` | Expiry parsing and Valid / ExpiringSoon / Expired classification coverage |
