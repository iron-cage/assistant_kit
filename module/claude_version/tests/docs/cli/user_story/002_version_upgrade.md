# Test: Version Upgrade

Acceptance tests for User Story 002. See [user_story/002_version_upgrade.md](../../../../docs/cli/user_story/002_version_upgrade.md) for specification.

### Scope

- **Purpose**: Verify the version upgrade workflow from preview through install and verification.
- **Responsibility**: Acceptance criteria coverage for the version upgrade scenario.
- **Commands:** `.version.show`, `.version.install`, `.version.guard`, `.version.history`
- **In Scope**: Dry-run preview, install execution, version lock, post-install verification, history check.
- **Out of Scope**: Process management (-> `003_process_lifecycle.md`), settings (-> `004_settings_management.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | Dry-run preview shows install plan without executing | Acceptance: preview |
| US-2 | Install applies version lock and exits 0 | Acceptance: install |
| US-3 | Already-at-target is no-op unless force::1 | Acceptance: idempotency |
| US-4 | `.version.show` after install prints new version | Acceptance: verification |
| US-5 | `.version.history` shows recent releases | Acceptance: history |
| US-6 | `.version.guard` detects drift after install | Acceptance: guard |

## Test Coverage Summary

- Dry-run preview: 1 test (US-1)
- Install execution: 1 test (US-2)
- Idempotency: 1 test (US-3)
- Post-install verification: 1 test (US-4)
- History inspection: 1 test (US-5)
- Drift detection: 1 test (US-6)

**Total:** 6 tests

---

### US-1: Dry-run preview shows install plan without executing

- **Given:** Claude Code is installed at version X
- **When:** `cm .version.install version::Y dry::1`
- **Then:** exit 0; stdout shows install plan; no files modified
- **Exit:** 0
- **Source:** [user_story/002 -- AC bullet 1](../../../../docs/cli/user_story/002_version_upgrade.md)

---

### US-2: Install applies version lock and exits 0

- **Given:** Claude Code is installed at version X; target version Y is available
- **When:** `cm .version.install version::Y`
- **Then:** exit 0; version Y installed; 5-layer version lock applied
- **Exit:** 0
- **Source:** [user_story/002 -- AC bullet 2](../../../../docs/cli/user_story/002_version_upgrade.md)

---

### US-3: Already-at-target is no-op unless force::1

- **Given:** Claude Code is already at version Y
- **When:** `cm .version.install version::Y`
- **Then:** exit 0; no install action taken (no-op)
- **Exit:** 0
- **Source:** [user_story/002 -- AC bullet 3](../../../../docs/cli/user_story/002_version_upgrade.md)

---

### US-4: `.version.show` after install prints new version

- **Given:** just installed version Y via `.version.install`
- **When:** `cm .version.show`
- **Then:** exit 0; output contains version Y
- **Exit:** 0
- **Source:** [user_story/002 -- AC bullet 4](../../../../docs/cli/user_story/002_version_upgrade.md)

---

### US-5: `.version.history` shows recent releases

- **Given:** Claude Code installed; network available
- **When:** `cm .version.history`
- **Then:** exit 0; output lists recent releases with summaries
- **Exit:** 0
- **Source:** [user_story/002 -- AC bullet 5](../../../../docs/cli/user_story/002_version_upgrade.md)

---

### US-6: `.version.guard` detects drift after install

- **Given:** version Y installed and locked
- **When:** `cm .version.guard`
- **Then:** exit 0; confirms version matches preferred or restores if drifted
- **Exit:** 0
- **Source:** [user_story/002 -- AC bullet 6](../../../../docs/cli/user_story/002_version_upgrade.md)

---

### Source Functions

| Function | File | Status |
|----------|------|--------|
| `us01_002_version_install_dry_preview` | `integration/user_story_test.rs` | ✅ |
| `us02_002_version_install_plan_accepted` | `integration/user_story_test.rs` | ✅ |
| `us03_002_version_install_idempotent` | `integration/user_story_test.rs` | ✅ |
| `us04_002_version_show_exits_0` | `integration/user_story_test.rs` | ✅ |
| `us05_002_version_history_exits_0` | `integration/user_story_test.rs` | ✅ |
| `us06_002_version_guard_exits_0` | `integration/user_story_test.rs` | ✅ |
