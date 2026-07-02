# Pitfall Test: Version Lock chmod Side Effects

### Scope

- **Purpose**: PF- test cases verifying the chmod 555 side-effect trap is avoided by `.version.install` and `.version.guard`.
- **Responsibility**: Confirm the implementation handles chmod transitions automatically; no manual chmod needed.
- **In Scope**: Automatic chmod handling in install, chmod restoration in guard, dry-run preview accuracy.
- **Out of Scope**: Symlink retarget bypass (→ `02_symlink_retarget.md`), full lock pattern (→ `../pattern/01_version_lock.md`).

Pitfall test surface for chmod side effects. See [pitfall/001_version_lock_chmod.md](../../../docs/pitfall/001_version_lock_chmod.md) for specification.

## Test Case Index

| PF | Scenario | Source fn |
|----|----------|-----------|
| PF-1 | `.version.install` with pinned version completes without requiring manual chmod | ✅ |
| PF-2 | `.version.install dry::1` preview references chmod in output | ✅ |
| PF-3 | `.version.guard dry::1` output indicates chmod restoration capability | ✅ |

**Total:** 3 tests

---

### PF-1: install handles chmod automatically

- **Given:** no manual `chmod` operations performed
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; no manual chmod step required from the caller; command manages permissions internally

---

### PF-2: install dry-run previews chmod operation

- **Given:** dry::1 (preview mode)
- **When:** `clv .version.install version::stable dry::1`
- **Then:** stdout references chmod or permissions in the preview output; exit 0

---

### PF-3: guard dry-run shows chmod restoration

- **Given:** dry::1 (preview mode)
- **When:** `clv .version.guard dry::1`
- **Then:** stdout indicates the guard can restore version lock (not just detect drift); exit 0

---

### Source Functions

| Function | File |
|----------|------|
| `pf01_001_chmod_auto_handled` | `tests/cli/pitfall_surface_test.rs` |
| `pf02_001_chmod_dry_shows_chmod` | `tests/cli/pitfall_surface_test.rs` |
| `pf03_001_guard_shows_restore` | `tests/cli/pitfall_surface_test.rs` |
