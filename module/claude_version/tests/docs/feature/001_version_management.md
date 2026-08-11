# Feature Test: Version Management

### Scope

- **Purpose**: FT- test cases for version install, guard, history, and alias resolution.
- **Responsibility**: Acceptance criteria verifying version alias resolution, idempotency, guard defaults, and preferred version persistence.
- **In Scope**: `.version.install`, `.version.guard`, alias resolution (stable/latest), idempotency, preference persistence.
- **Out of Scope**: 8-layer version lock (-> `../../pattern/01_version_lock.md`), dry-run semantics (-> `04_dry_run.md`).

Feature test surface for version management. See [feature/001_version_management.md](../../../docs/feature/001_version_management.md) for specification.

## Behavioral Divergence Pair

Two valid version aliases produce distinct output:

- **Input A:** `clv .version.install version::stable dry::1` → output contains `"2.1.220"` (pinned stable semver)
- **Input B:** `clv .version.install version::latest dry::1` → output enables auto-updates (no version pin)

Both are valid invocations; the lock behavior differs.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `version::stable dry::1` → output contains pinned semver `2.1.220` | Alias Resolution |
| FT-3 | Guard with no preference stored → defaults to `stable` | Guard Default |
| FT-4 | Guard with `version::latest` preference → skips pin, shows "no version pin" | Guard Latest |
| FT-5 | `dry::1` does not write `preferredVersionSpec` to settings | Preference Isolation |
| FT-6 | Guard with stale `preferredVersionResolved` re-resolves alias and uses current semver | Guard Alias Re-resolution |

## Test Coverage Summary

- Alias Resolution: 1 test (FT-1)
- Guard Default: 1 test (FT-3)
- Guard Latest: 1 test (FT-4)
- Preference Isolation: 1 test (FT-5)
- Guard Alias Re-resolution: 1 test (FT-6)

**Total:** 5 tests

---

### FT-1: `version::stable dry::1` → output contains pinned semver `2.1.220`

- **Given:** clean environment, no settings file
- **When:** `clv .version.install version::stable dry::1`
- **Then:** stdout contains `"2.1.220"`; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Version aliases](../../../docs/feature/001_version_management.md)

---

### FT-3: Guard with no preference stored → defaults to `stable`

- **Given:** isolated HOME with no `settings.json` (no `preferredVersionSpec` key)
- **When:** `clv .version.guard dry::1`
- **Then:** stdout contains `"stable"`; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Version guard](../../../docs/feature/001_version_management.md)

---

### FT-4: Guard with `version::latest` preference → skips pin, shows "no version pin"

- **Given:** isolated HOME with `settings.json` containing `preferredVersionSpec = "latest"`
- **When:** `clv .version.guard dry::1`
- **Then:** stdout contains text indicating no version pin to guard; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Version guard](../../../docs/feature/001_version_management.md)

---

### FT-5: `dry::1` does not write `preferredVersionSpec` to settings

- **Given:** isolated HOME with empty `settings.json`
- **When:** `clv .version.install version::stable dry::1`
- **Then:** `settings.json` does not contain `"preferredVersionSpec"`; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Preferred version persistence](../../../docs/feature/001_version_management.md)

---

### FT-6: Guard with stale `preferredVersionResolved` re-resolves alias

- **Given:** isolated HOME with `settings.json` containing `preferredVersionSpec = "stable"` and `preferredVersionResolved = "2.1.0"` (stale — alias has since been bumped to 2.1.220)
- **When:** `clv .version.guard dry::1`
- **Then:** stdout contains `"2.1.220"` (not `"2.1.0"`); exit 0; stale stored value ignored
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Version guard (alias re-resolution)](../../../docs/feature/001_version_management.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc301_version_install_dry_stable` | `tests/cli/mutation_version_install_test.rs` |
| `tc400_guard_no_preference` | `tests/cli/mutation_version_guard_test.rs` |
| `tc403_guard_latest_dry` | `tests/cli/mutation_version_guard_test.rs` |
| `tc357_version_install_dry_no_preference_written` | `tests/cli/mutation_version_install_test.rs` |
| `tc410_guard_reresoves_stale_alias` | `tests/cli/mutation_version_guard_test.rs` |
