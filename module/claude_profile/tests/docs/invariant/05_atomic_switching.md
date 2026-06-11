# Test: Invariant 005 — Atomic Account Switching

Property assertion cases for `docs/invariant/005_atomic_switching.md`. Verifies that
`switch_account()` uses a write-then-rename pattern and that `.credentials.json` always
contains complete credentials at any observable point during a switch.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | switch_account uses temp file + std::fs::rename, not direct write | Invariant holds (normal) |
| IN-2 | Credentials file contains complete credentials before and after switch | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: switch_account uses temp file + std::fs::rename, not direct write

- **Given:** The `src/` implementation of `switch_account()` (or equivalent account-switching fn)
- **When:** The implementation is inspected for the use of `std::fs::rename` as the final step
  of writing `.credentials.json`
- **Then:** `std::fs::rename` is called as the final write step; `std::fs::write` or equivalent
  is never called directly on `.credentials.json` as the target path — the rename is always from
  a temporary adjacent file
- **Source:** [docs/invariant/005_atomic_switching.md](../../../docs/invariant/005_atomic_switching.md)

---

### IN-2: Credentials file contains complete credentials before and after switch

- **Given:** A valid `.credentials.json` exists with account A's credentials; a switch to account
  B is initiated
- **When:** The switch operation completes (or is interrupted before the rename step)
- **Then:** At every observable moment, `.credentials.json` is either the complete old credentials
  (A) or the complete new credentials (B); partial/truncated content never appears at the
  `.credentials.json` path
- **Source:** [docs/invariant/005_atomic_switching.md](../../../docs/invariant/005_atomic_switching.md)
