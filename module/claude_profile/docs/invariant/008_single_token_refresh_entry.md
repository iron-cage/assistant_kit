# Invariant: Single Token Refresh Entry Point

### Scope

- **Purpose**: Guarantee that all token refresh operations go through `refresh_account_token()`, which wraps `run_isolated()` with RT rotation (expiresAt manipulation) and current-account live credential sync.
- **Responsibility**: Documents the single-entry-point constraint for credential refresh across `claude_profile` and `claude_profile_core`.
- **In Scope**: All call sites that produce new OAuth credentials via `run_isolated()` or any other subprocess-based mechanism.
- **Out of Scope**: `run_isolated()` internal implementation (-> `claude_runner_core`); non-credential subprocess uses (if any future use arises).

### Invariant Statement

All token refresh operations across `claude_profile` and `claude_profile_core` MUST call `claude_profile_core::account::refresh_account_token()`. Direct calls to `claude_runner_core::run_isolated()` for credential refresh are forbidden.

**Measurable threshold:** Zero direct `run_isolated()` calls outside `refresh_account_token()` in `claude_profile` and `claude_profile_core` source trees, detected by automated grep test.

**Rationale:** `refresh_account_token()` wraps `run_isolated()` with two critical behaviors:

1. **RT rotation (Change A):** Sets `expiresAt=1` in the credential JSON before passing to `run_isolated`, forcing Claude CLI to treat the access token as expired. The CLI then uses the refresh token to obtain a fresh AT+RT pair, rotating the RT on every call. Without this, `run_isolated` with a valid AT returns `credentials=None` (CLI uses AT as-is, no RT rotation), and the RT ages silently until it expires server-side — making the account irrecoverable without manual re-authentication.

2. **Current-account live sync (Change B):** When the account being refreshed is the current account, checks `~/.claude/.credentials.json` (live session credentials) against the stored credentials. If different (the live Claude Code session already refreshed), syncs live->store without spawning a subprocess. After `run_isolated` returns `credentials=None`, re-checks live credentials as race recovery.

Bypassing `refresh_account_token()` loses both behaviors: RT silently ages (no rotation), and current-account credentials diverge between live and store (no sync).

### Enforcement Mechanism

- Automated test: grep-based invariant test scans all non-test `.rs` files in `claude_profile/src/` for `run_isolated(` call sites; asserts zero matches. `refresh_account_token()` in `claude_profile_core/src/account.rs` is the sole authorized caller.
- Doc comment warnings: `run_isolated()` function doc in `claude_runner_core/src/isolated.rs` and the `pub use` re-export in `claude_runner_core/src/lib.rs` carry warnings directing callers to `refresh_account_token()`.
- Code review: immediate rejection of any direct `run_isolated()` call outside `account.rs`.

### Violation Consequences

- **RT decay:** Direct `run_isolated()` calls with valid ATs return `credentials=None` — no RT rotation occurs. The RT ages silently across multiple AT cycles until it expires server-side, making the account permanently irrecoverable (requires browser re-authentication).
- **Credential divergence:** For the current account, direct `run_isolated()` calls may race with the live Claude Code session. Without live credential sync, the store retains a stale RT while the live session holds a fresh one — the next `apply_refresh` cycle uses the stale RT and fails.
- **Lost write-back:** `apply_post_switch_touch` previously called `run_isolated()` directly (fire-and-forget) — any RT rotation from that subprocess was silently discarded because no credential write-back occurred.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `claude_profile_core/src/account.rs` | `refresh_account_token()` — sole authorized `run_isolated()` caller |
| source | `claude_runner_core/src/isolated.rs` | `run_isolated()` — doc comment warning directing to `refresh_account_token()` |
| feature | [017_token_refresh.md](../feature/017_token_refresh.md) | Token refresh feature — AC-32/AC-33/AC-34 implement this invariant |
| feature | [024_session_touch.md](../feature/024_session_touch.md) | Session touch — calls `refresh_account_token()` per this invariant |
| feature | [027_account_use_post_switch_touch.md](../feature/027_account_use_post_switch_touch.md) | Post-switch touch — Change C routes through `refresh_account_token()` |
| invariant | [004_no_process_execution.md](004_no_process_execution.md) | Related boundary: `claude_profile` has zero `std::process` — all execution delegated |
| test | (invariant grep test) | Automated test enforcing zero `run_isolated()` calls outside `account.rs` |
