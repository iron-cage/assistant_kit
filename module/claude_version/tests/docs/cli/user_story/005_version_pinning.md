# Test: Version Pinning

Acceptance tests for User Story 005. See [user_story/005_version_pinning.md](../../../../docs/cli/user_story/005_version_pinning.md) for specification.

### Scope

- **Purpose**: Verify team-wide version pinning workflow.
- **Responsibility**: Acceptance criteria coverage for the version pinning scenario.
- **Commands:** `.version.list`, `.version.install`, `.version.show`, `.version.guard`
- **In Scope**: Alias resolution, install with lock, idempotency, post-install verification, drift watch.
- **Out of Scope**: Process management (-> `03_process_lifecycle.md`), settings (-> `04_settings_management.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.version.list` shows aliases with resolved versions | Acceptance: alias listing |
| US-2 | Dry-run preview for stable baseline | Acceptance: preview |
| US-4 | Already-at-pinned-version is no-op | Acceptance: idempotency |
| US-5 | `.version.show` confirms pinned version active | Acceptance: verification |
| US-6 | `.version.guard interval::N` watches for drift | Acceptance: drift watch |

## Test Coverage Summary

- Alias listing: 1 test (US-1)
- Dry-run preview: 1 test (US-2)
- Idempotency: 1 test (US-4)
- Post-install verification: 1 test (US-5)
- Drift watch: 1 test (US-6)

**Total:** 5 tests

---

### US-1: `.version.list` shows aliases with resolved versions

- **Given:** network available
- **When:** `clv .version.list`
- **Then:** exit 0; output contains stable, latest, and any custom aliases with resolved semver versions
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 1](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-2: Dry-run preview for stable baseline

- **Given:** Claude Code installed; network available
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; stdout shows install plan for stable baseline; no files modified
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 2](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-4: Already-at-pinned-version is no-op

- **Given:** Claude Code already at stable version
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; no install action taken
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 4](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-5: `.version.show` confirms pinned version active

- **Given:** stable version just installed
- **When:** `clv .version.show`
- **Then:** exit 0; output confirms stable version is active
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 5](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-6: `.version.guard interval::N` watches for drift

- **Given:** version locked to stable
- **When:** `clv .version.guard interval::5`
- **Then:** process runs; watches for drift; restores pinned version if drift detected
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 6](../../../../docs/cli/user_story/005_version_pinning.md)

---

### Source Functions

| Function | File | Status |
|----------|------|--------|
| `us01_005_version_list_shows_aliases` | `tests/cli/user_story_test.rs` | ✅ |
| `us02_005_version_install_stable_dry` | `tests/cli/user_story_test.rs` | ✅ |
| `us04_005_version_install_idempotent` | `tests/cli/user_story_test.rs` | ✅ |
| `us05_005_version_show_confirms_active` | `tests/cli/user_story_test.rs` | ✅ |
| `us06_005_version_guard_drift_watch` | `tests/cli/user_story_test.rs` | ✅ |
