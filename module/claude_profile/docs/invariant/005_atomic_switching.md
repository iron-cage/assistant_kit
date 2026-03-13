# Invariant: Atomic Account Switching

### Scope

- **Purpose**: Prevent credential corruption on crash or power loss during account switches.
- **Responsibility**: Documents the write-then-rename atomicity requirement for `switch_account()` (NFR-6).
- **In Scope**: The atomic rename guarantee for `~/.claude/.credentials.json` writes; `_active` marker durability.
- **Out of Scope**: What happens after switching (caller responsibility: terminate old processes, verify new account).

### Invariant Statement

Account switching (FR-9) must use write-then-rename to prevent credential corruption on crash or power loss.

**Measurable threshold:** `switch_account()` implementation always writes to a temp file adjacent to `.credentials.json` and renames — never writes directly to `.credentials.json`.

**Guarantee:** At every point during a switch, `~/.claude/.credentials.json` contains either the complete old credentials or the complete new credentials — never a partial write.

**Formal crash analysis:**
- Crash after temp write, before rename → temp file cleaned up on restart; old credentials intact
- Crash after rename, before `_active` update → new credentials active; `_active` marker stale (advisory only — not enforced by Claude Code)
- Crash during rename → OS guarantees rename is atomic on same filesystem (POSIX rename semantics)

**`_active` marker:** Best-effort metadata. A stale `_active` after a crash is acceptable — `.credentials.json` is the authoritative state.

### Enforcement Mechanism

- Implementation constraint: `switch_account()` must use `std::fs::rename` (not `std::fs::write` directly to target)
- Code review: reject any PR that writes directly to `.credentials.json` without a temp-then-rename pattern
- Test: verify the implementation uses rename (structural test)

### Violation Consequences

- A crash mid-write to `.credentials.json` corrupts the file → user cannot authenticate until they manually restore credentials
- Data loss: original credentials are overwritten before new ones are fully written
- Unrecoverable without manual intervention or backup — `claude_profile` provides no backup/restore

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `switch_account()` — write-then-rename implementation |
| test | `tests/account_tests.rs::switch_account_overwrites_credentials_file` | Verifies atomic overwrite semantics |
| doc | [004_account_switch.md](../feature/004_account_switch.md) | Feature design for account switching |
