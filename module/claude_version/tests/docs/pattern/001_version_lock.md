# Pattern Test: Version Lock

### Scope

- **Purpose**: PT- test cases for the 5-layer version lock strategy applied after pinned installs.
- **Responsibility**: Verify that dry-run previews reflect the correct lock/unlock behavior for pinned vs latest versions.
- **In Scope**: Layer 1 (`autoUpdates`), Layer 2 (`DISABLE_AUTOUPDATER`), Layer 4 (purge preview), lock inversion for `latest`.
- **Out of Scope**: Actual file writes and chmod operations (-> source integration tests), guard recovery behavior (-> `../feature/001_version_management.md`).

Pattern test surface for version lock. See [pattern/001_version_lock.md](../../../../docs/pattern/001_version_lock.md) for specification.

## Behavioral Divergence Pair

Two valid version installs produce opposite lock behavior in dry-run output:

- **Input A:** `cm .version.install version::stable dry::1` → output contains `"autoUpdates = false"` (lock applied)
- **Input B:** `cm .version.install version::latest dry::1` → output contains `"autoUpdates = true"` (lock inverted)

Both are valid invocations; the `autoUpdates` value in the preview is opposite.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PT-1 | Pinned version dry-run preview shows `autoUpdates = false` (Layer 1 lock) | Lock Layer 1 |
| PT-2 | `latest` version dry-run preview shows `autoUpdates = true` (Layer 1 inversion) | Lock Inversion |
| PT-3 | Pinned version dry-run preview includes Layer 4 purge line | Lock Layer 4 |

## Test Coverage Summary

- Lock Layer 1: 1 test (PT-1)
- Lock Inversion: 1 test (PT-2)
- Lock Layer 4: 1 test (PT-3)

**Total:** 3 tests

---

### PT-1: Pinned version dry-run preview shows `autoUpdates = false` (Layer 1 lock)

- **Given:** clean environment
- **When:** `cm .version.install version::stable dry::1`
- **Then:** stdout contains `"autoUpdates"` and `"false"`; exit 0
- **Exit:** 0
- **Source:** [pattern/001_version_lock.md — Layer 1](../../../../docs/pattern/001_version_lock.md)

---

### PT-2: `latest` version dry-run preview shows `autoUpdates = true` (Layer 1 inversion)

- **Given:** clean environment
- **When:** `cm .version.install version::latest dry::1`
- **Then:** stdout contains `"autoUpdates"` and `"true"`; exit 0
- **Exit:** 0
- **Source:** [pattern/001_version_lock.md — Lock inversion for latest](../../../../docs/pattern/001_version_lock.md)

---

### PT-3: Pinned version dry-run preview includes Layer 4 purge line

- **Given:** clean environment
- **When:** `cm .version.install version::stable dry::1`
- **Then:** stdout contains text indicating cached binary purge; exit 0
- **Exit:** 0
- **Source:** [pattern/001_version_lock.md — Layer 4](../../../../docs/pattern/001_version_lock.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc351_version_install_dry_stable_auto_updates_false` | `integration/mutation_commands_test.rs` |
| `tc350_version_install_dry_latest_auto_updates_true` | `integration/mutation_commands_test.rs` |
| `tc359_version_install_dry_stable_includes_purge` | `integration/mutation_commands_test.rs` |
