# Pitfall Test: Auto-Updater Symlink Retarget

### Scope

- **Purpose**: PF- test cases verifying the symlink retarget bypass is neutralized by Layer 4 binary purge.
- **Responsibility**: Confirm `.version.install` purges cached binaries and `.version.guard` detects/corrects symlink drift.
- **In Scope**: Layer 4 purge in install dry-run preview, guard drift detection, symlink correctness post-install.
- **Out of Scope**: chmod side effects (→ `01_version_lock_chmod.md`), full lock pattern (→ `../pattern/01_version_lock.md`).

Pitfall test surface for symlink retarget. See [pitfall/002_symlink_retarget.md](../../../docs/pitfall/002_symlink_retarget.md) for specification.

## Test Case Index

| PF | Scenario | Source fn |
|----|----------|-----------|
| PF-1 | `.version.install` dry-run preview confirms Layer 4 purge of cached binaries | ✅ |
| PF-2 | `.version.guard dry::1` output confirms drift detection capability | ✅ |
| PF-3 | `.version.guard` reports no drift immediately after a pinned `.version.install` | ✅ |

**Total:** 3 tests

---

### PF-1: install dry-run confirms Layer 4 purge

- **Given:** dry::1 (preview mode)
- **When:** `clv .version.install version::stable dry::1`
- **Then:** stdout references binary purge or cache removal in the preview output; exit 0

---

### PF-2: guard dry-run confirms drift detection

- **Given:** dry::1 (preview mode)
- **When:** `clv .version.guard dry::1`
- **Then:** exit 0; preview output describes version drift detection and recovery steps

---

### PF-3: no drift immediately after pinned install

- **Given:** `.version.install version::stable` completed successfully in a clean environment
- **When:** `clv .version.guard dry::1`
- **Then:** exit 0; preview output indicates no drift detected (version matches preference)

---

### Source Functions

| Function | File |
|----------|------|
| `pf01_002_purge_in_install_preview` | `tests/cli/pitfall_surface_test.rs` |
| `pf02_002_guard_dry_detects_drift` | `tests/cli/pitfall_surface_test.rs` |
| `pf03_002_no_drift_after_install` | `tests/cli/pitfall_surface_test.rs` |
