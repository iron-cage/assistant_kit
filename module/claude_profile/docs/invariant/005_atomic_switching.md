# Invariant: Atomic Account Switching

### Scope

- **Purpose**: Prevent credential corruption on crash or power loss during account switches.
- **Responsibility**: Documents the write-then-rename atomicity requirement for `switch_account()` (NFR-6).
- **In Scope**: The atomic rename guarantee for `~/.claude/.credentials.json` writes; active marker durability.
- **Out of Scope**: What happens after switching (caller responsibility: terminate old processes, verify new account).

### Invariant Statement

Account switching (FR-9) must use write-then-rename to prevent credential corruption on crash or power loss.

**Measurable threshold:** `switch_account()` implementation always writes to a temp file adjacent to `.credentials.json` and renames — never writes directly to `.credentials.json`.

**Guarantee:** At every point during a switch, `~/.claude/.credentials.json` contains either the complete old credentials or the complete new credentials — never a partial write.

**Formal crash analysis:**
- Crash after temp write, before rename → temp file cleaned up on restart; old credentials intact
- Crash after rename, before `_active_{hostname}_{user}` update → new credentials active; active marker stale (advisory only — not enforced by Claude Code)
- Crash during rename → OS guarantees rename is atomic on same filesystem (POSIX rename semantics)

**Active marker:** Best-effort metadata. A stale `_active_{hostname}_{user}` after a crash is acceptable — `.credentials.json` is the authoritative state.

**Scope boundary (BUG-341):** The staleness tolerance above covers only the transient crash-recovery window in the Formal crash analysis — a marker that briefly lags a completed rename until the next switch. It does NOT excuse indefinite staleness after a completed, non-crashed `delete()`: `delete()` clears every `_active_*` marker — the calling machine's own and every other machine's — naming the deleted account (see [feature/025_per_machine_active_marker.md](../feature/025_per_machine_active_marker.md) AC-06), and `.usage`'s Sessions table flags any marker still naming an account absent from the credential store with `(stale)` as a defense-in-depth backstop (see [feature/009_token_usage.md](../feature/009_token_usage.md) AC-33).

### Enforcement Mechanism

- Implementation constraint: `switch_account()` must use `std::fs::rename` (not `std::fs::write` directly to target)
- Code review: reject any PR that writes directly to `.credentials.json` without a temp-then-rename pattern
- Test: verify the implementation uses rename (structural test)

### Violation Consequences

- A crash mid-write to `.credentials.json` corrupts the file → user cannot authenticate until they manually restore credentials
- Data loss: original credentials are overwritten before new ones are fully written
- Unrecoverable without manual intervention or backup — `claude_profile` provides no backup/restore

### Sources

| File | Relationship |
|------|-------------|
| `claude_profile_core/src/account.rs` | `switch_account()` — write-then-rename implementation (`src/account.rs` in this crate only re-exports it) |

### Features

| File | Relationship |
|------|-------------|
| [004_account_use.md](../feature/004_account_use.md) | Feature design for account switching |
| [025_per_machine_active_marker.md](../feature/025_per_machine_active_marker.md) | AC-06: `delete()` clears own and foreign markers naming the deleted account — the permanent-staleness case this invariant's tolerance clause does not cover |
| [009_token_usage.md](../feature/009_token_usage.md) | AC-33: Sessions table `(stale)` flag — defense-in-depth backstop for any marker that still escapes AC-06's clearing |

### Tests

| File | Relationship |
|------|-------------|
| `tests/account_tests.rs::switch_account_overwrites_credentials_file` | Verifies atomic overwrite semantics |
