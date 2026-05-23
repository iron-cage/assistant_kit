# Feature Test: Version Management

### Scope

- **Purpose**: FT- test cases for version install, guard, history, and alias resolution.
- **Responsibility**: Acceptance criteria verifying version alias resolution, idempotency, guard defaults, and preferred version persistence.
- **In Scope**: `.version.install`, `.version.guard`, alias resolution (stable/month/latest), idempotency, preference persistence.
- **Out of Scope**: 5-layer version lock (-> `../../pattern/001_version_lock.md`), dry-run semantics (-> `004_dry_run.md`).

Feature test surface for version management. See [feature/001_version_management.md](../../../../docs/feature/001_version_management.md) for specification.

## Behavioral Divergence Pair

Two valid version aliases produce distinct output:

- **Input A:** `cm .version.install version::stable dry::1` → output contains `"2.1.78"` (pinned stable semver)
- **Input B:** `cm .version.install version::month dry::1` → output contains `"2.1.74"` (pinned month semver)

Both are valid invocations; the resolved semver strings differ.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | `version::stable dry::1` → output contains pinned semver `2.1.78` | Alias Resolution |
| FT-2 | `version::month dry::1` → output contains pinned semver `2.1.74` | Alias Resolution |
| FT-3 | Guard with no preference stored → defaults to `stable` | Guard Default |
| FT-4 | Guard with `version::latest` preference → skips pin, shows "no version pin" | Guard Latest |
| FT-5 | `dry::1` does not write `preferredVersionSpec` to settings | Preference Isolation |

## Test Coverage Summary

- Alias Resolution: 2 tests (FT-1, FT-2)
- Guard Default: 1 test (FT-3)
- Guard Latest: 1 test (FT-4)
- Preference Isolation: 1 test (FT-5)

**Total:** 5 tests

---

### FT-1: `version::stable dry::1` → output contains pinned semver `2.1.78`

- **Given:** clean environment, no settings file
- **When:** `cm .version.install version::stable dry::1`
- **Then:** stdout contains `"2.1.78"`; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Version aliases](../../../../docs/feature/001_version_management.md)

---

### FT-2: `version::month dry::1` → output contains pinned semver `2.1.74`

- **Given:** clean environment, no settings file
- **When:** `cm .version.install version::month dry::1`
- **Then:** stdout contains `"2.1.74"`; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Version aliases](../../../../docs/feature/001_version_management.md)

---

### FT-3: Guard with no preference stored → defaults to `stable`

- **Given:** isolated HOME with no `settings.json` (no `preferredVersionSpec` key)
- **When:** `cm .version.guard dry::1`
- **Then:** stdout contains `"stable"`; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Version guard](../../../../docs/feature/001_version_management.md)

---

### FT-4: Guard with `version::latest` preference → skips pin, shows "no version pin"

- **Given:** isolated HOME with `settings.json` containing `preferredVersionSpec = "latest"`
- **When:** `cm .version.guard dry::1`
- **Then:** stdout contains text indicating no version pin to guard; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Version guard](../../../../docs/feature/001_version_management.md)

---

### FT-5: `dry::1` does not write `preferredVersionSpec` to settings

- **Given:** isolated HOME with empty `settings.json`
- **When:** `cm .version.install version::stable dry::1`
- **Then:** `settings.json` does not contain `"preferredVersionSpec"`; exit 0
- **Exit:** 0
- **Source:** [feature/001_version_management.md — Preferred version persistence](../../../../docs/feature/001_version_management.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc301_version_install_dry_stable` | `integration/mutation_commands_test.rs` |
| `tc309_version_install_dry_month` | `integration/mutation_commands_test.rs` |
| `tc400_guard_no_preference_defaults_stable` | `integration/mutation_commands_test.rs` |
| `tc403_guard_preference_latest_dry` | `integration/mutation_commands_test.rs` |
| `tc357_version_install_dry_does_not_write_preference` | `integration/mutation_commands_test.rs` |
