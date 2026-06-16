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
| US-2 | Dry-run preview for monthly baseline | Acceptance: preview |
| US-3 | Install monthly baseline applies 5-layer lock | Acceptance: install |
| US-4 | Already-at-pinned-version is no-op | Acceptance: idempotency |
| US-5 | `.version.show` confirms pinned version active | Acceptance: verification |
| US-6 | `.version.guard interval::N` watches for drift | Acceptance: drift watch |

## Test Coverage Summary

- Alias listing: 1 test (US-1)
- Dry-run preview: 1 test (US-2)
- Install with lock: 1 test (US-3)
- Idempotency: 1 test (US-4)
- Post-install verification: 1 test (US-5)
- Drift watch: 1 test (US-6)

**Total:** 6 tests

---

### US-1: `.version.list` shows aliases with resolved versions

- **Given:** network available
- **When:** `clv .version.list`
- **Then:** exit 0; output contains stable, month, and latest aliases with resolved semver versions
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 1](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-2: Dry-run preview for monthly baseline

- **Given:** Claude Code installed; network available
- **When:** `clv .version.install version::month dry::1`
- **Then:** exit 0; stdout shows install plan for monthly baseline; no files modified
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 2](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-3: Install monthly baseline applies 5-layer lock

- **Given:** Claude Code installed; monthly baseline version available
- **When:** `clv .version.install version::month`
- **Then:** exit 0; monthly version installed; 5-layer version lock applied
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 3](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-4: Already-at-pinned-version is no-op

- **Given:** Claude Code already at monthly baseline version
- **When:** `clv .version.install version::month`
- **Then:** exit 0; no install action taken
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 4](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-5: `.version.show` confirms pinned version active

- **Given:** monthly baseline just installed
- **When:** `clv .version.show`
- **Then:** exit 0; output confirms monthly baseline version is active
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 5](../../../../docs/cli/user_story/005_version_pinning.md)

---

### US-6: `.version.guard interval::N` watches for drift

- **Given:** version locked to monthly baseline
- **When:** `clv .version.guard interval::5`
- **Then:** process runs; watches for drift; restores pinned version if drift detected
- **Exit:** 0
- **Source:** [user_story/005 -- AC bullet 6](../../../../docs/cli/user_story/005_version_pinning.md)

---

### Source Functions

| Function | File | Status |
|----------|------|--------|
| `us01_005_version_list_shows_aliases` | `integration/user_story_test.rs` | ✅ |
| `us02_005_version_install_month_dry` | `integration/user_story_test.rs` | ✅ |
| `us03_005_version_install_month_accepted` | `integration/user_story_test.rs` | ✅ |
| `us04_005_version_install_idempotent` | `integration/user_story_test.rs` | ✅ |
| `us05_005_version_show_confirms_active` | `integration/user_story_test.rs` | ✅ |
| `us06_005_version_guard_drift_watch` | `integration/user_story_test.rs` | ✅ |
