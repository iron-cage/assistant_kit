# Test: `.version.install`

### Scope

- **Purpose**: Integration test cases for the `.version.install` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for version installation.
- **In Scope**: Version aliases, semver validation, dry-run, force, idempotency, 5-layer lock.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.version.install` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

## Test Factor Analysis

### Factor 1: `version::` (String, optional, default "stable")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Defaults to `stable` | Default behavior |
| `stable` | Named alias (pinned) | Valid alias |
| `month` | Named alias (pinned) | Valid alias |
| `latest` | Named alias (dynamic) | Valid alias (special: no lock) |
| `1.2.3` | Valid semver | Valid semver |
| `2.1.50` | Valid semver (older) | Valid semver |
| `0.0.0` | Minimal semver | Boundary: minimum valid |
| `STABLE` | Wrong-case alias | Invalid: exit 1 |
| (empty) | Empty string | Invalid: exit 1 |
| `1.2` | Two-part semver | Invalid: exit 1 |
| `01.02.03` | Leading zeros | Invalid: exit 1 |
| `x` | Unknown alias | Invalid: exit 1 |

Boundary set: `0.0.0`, `latest`, two-part, leading-zeros.

### Factor 2: `dry::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: no dry-run, real install | Default behavior |
| 0 | Explicit: real install | Explicit false |
| 1 | Preview mode; no side effects | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 3: `force::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: idempotency guard active | Default behavior |
| 0 | Explicit: guard active | Explicit false |
| 1 | Bypass idempotency check | Explicit true |

### Factor 4: Interaction: `dry::1` vs `force::1`

| Combination | Behavior | Expected |
|-------------|----------|---------|
| `dry::1 force::1` | dry wins, no install | Preview only |
| `force::1` alone | bypass guard, real install | Real install |

### Factor 5: Lock actions for version type

| Version type | Lock behavior | Description |
|-------------|---------------|-------------|
| pinned alias / semver | autoUpdates=false, DISABLE_AUTOUPDATER=1, chmod 555, purge stale binaries, store preferredVersionSpec/Resolved | 5-layer lock |
| `latest` | autoUpdates=true, remove DISABLE_AUTOUPDATER, chmod 755 | Unlock |

### Factor 6: Preference storage

| Scenario | Expected |
|----------|---------|
| Successful install | `preferredVersionSpec` + `preferredVersionResolved` written |
| `dry::1` | Preference keys NOT written |
| Idempotent skip | Preference still written |

### Factor 7: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

### Factor 8: `verbosity::` / `v::` (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: labeled output | Default behavior |
| 0 | Bare/compact output | Compact |
| 1 | Labeled output | Labeled |

### Factor 9: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: text output | Default behavior |
| `text` | Human-readable text | Valid |
| `json` | Machine-readable JSON | Valid |
| `JSON` | Wrong case | Invalid: exit 1 |

---

## Test Matrix

### Positive Tests (dry-run mode — no network needed)

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | `dry::1` → `[dry-run]` prefix, exit 0 | P | 0 | F2=1 | [mutation_version_install_test.rs] |
| IT-9 | `version::stable dry::1` → preview shows `stable` | P | 0 | F1=stable, F2=1 | [mutation_version_install_test.rs] |
| IT-10 | `version::1.2.3 dry::1` → preview shows exact version | P | 0 | F1=semver, F2=1 | [mutation_version_install_test.rs] |
| IT-2 | `dry::1 force::1` → dry wins | P | 0 | F2=1, F3=1, F4 | [mutation_version_install_test.rs] |
| IT-11 | Absent `version::` with `dry::1` → uses `stable` | P | 0 | F1=absent, F2=1 | [mutation_version_install_test.rs] |
| IT-12 | `version::month dry::1` → resolves to pinned semver (2.1.74) | P | 0 | F1=month, F2=1 | [mutation_version_install_test.rs] |
| IT-13 | `version::latest dry::1` → autoUpdates=true in preview | P | 0 | F1=latest, F2=1, F5=unlock | [mutation_version_install_test.rs] |
| IT-14 | `version::stable dry::1` → autoUpdates=false in preview | P | 0 | F1=stable, F2=1, F5=lock | [mutation_version_install_test.rs] |
| IT-15 | `version::2.1.50 dry::1` → autoUpdates=false in preview | P | 0 | F1=semver, F2=1, F5=lock | [mutation_version_install_test.rs] |
| IT-16 | `version::latest dry::1` → previews unlock actions | P | 0 | F1=latest, F2=1, F5=unlock | [mutation_version_install_test.rs] |
| IT-17 | `version::0.0.0 dry::1` → single-zero parts valid | P | 0 | F1=0.0.0, F2=1 | [mutation_version_install_test.rs] |
| IT-18 | `dry::1` output mentions preferred version storage | P | 0 | F2=1, F6 | [mutation_version_install_test.rs] |
| IT-19 | `dry::1` does NOT write preference keys | P | 0 | F2=1, F6=no-write | [mutation_version_install_test.rs] |
| IT-20 | Idempotent skip still stores preference | P | 0 | F6=idempotent | [mutation_version_install_test.rs] |
| IT-21 | `version::stable dry::1` → output includes purge line | P | 0 | F2=1, F5=layer4 | [mutation_version_install_test.rs] |
| IT-22 | `version::latest dry::1` → output does NOT contain "purge" | P | 0 | F1=latest, F2=1, F5=unlock | [mutation_version_install_test.rs] |
| IT-6 | `dry::1 format::json` → JSON object output, exit 0 | P | 0 | F2=1, F9=json | [mutation_version_install_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-3 | `version::STABLE` → wrong case, exit 1 | N | 1 | F1=STABLE | [mutation_version_install_test.rs] |
| IT-23 | `version::` (empty) → exit 1 | N | 1 | F1=empty | [mutation_version_install_test.rs] |
| IT-24 | `version::1.2` → two-part semver, exit 1 | N | 1 | F1=1.2 | [mutation_version_install_test.rs] |
| IT-25 | `version::x` → unknown alias, exit 1 | N | 1 | F1=x | [mutation_version_install_test.rs] |
| IT-4 | `version::01.02.03` → leading zeros, exit 1 | N | 1 | F1=leading-zeros | [mutation_version_install_test.rs] |
| IT-5 | `bogus::x` → unknown param, exit 1 | N | 1 | F7=present | new |
| IT-7 | `format::JSON` (uppercase) → exit 1 | N | 1 | F9=JSON | [mutation_version_install_test.rs] |
| IT-8 | `dry::2` → out-of-range boolean, exit 1 | N | 1 | F2=2 | [mutation_version_install_test.rs] |

### Summary

- **Total:** 25 tests (17 positive, 8 negative)
- **Negative ratio:** 32.0% — supplemented by cross-cutting `tc242_unknown_format_exits_1`, `tc243_uppercase_format_exits_1`, `tc244_empty_format_exits_1`, `tc261_version_install_format_json_accepted` in `read_status_test.rs` / `cross_cutting_test.rs`
- **Combined with cross-cutting:** 11/26 = 42.3% ✅
- **IT range:** IT-1 to IT-25

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (dry-run or real) | IT-1 through IT-2, IT-6, IT-9 through IT-22 |
| 1 | Invalid arguments | IT-3, IT-4, IT-5, IT-7, IT-8, IT-23, IT-24, IT-25 |
| 2 | Runtime error (install failure) | Real-install tests only (not in automated suite) |

### Dry-Run Parity Requirement (FR-05)

`[dry-run] would install X (Y)` must exactly mirror actual install message.
IT-1 through IT-2 verify dry-run prefix and content consistency.
IT-19 verifies dry-run has zero side effects on settings.

### Version Lock / Unlock Coverage (FR-15)

| Scenario | Lock Status | Test |
|----------|-------------|------|
| pinned semver (2.1.50) | 5-layer lock | IT-15, IT-21 (layers 1–4), IT-18 (layer 5) |
| stable alias | 5-layer lock | IT-14, IT-21 (layers 1–4), IT-18 (layer 5) |
| latest alias | Remove all locks | IT-13, IT-16 |

### Preference Storage Coverage (FR-17)

| Scenario | Written? | Test |
|----------|----------|------|
| dry::1 | No | IT-19 |
| real install (idempotent skip) | Yes | IT-20 |
| dry::1 (preview mentions storage) | Preview only | IT-18 |

---

## Test Case Details

---

### IT-1: `dry::1` → `[dry-run]` prefix

- **Given:** clean environment
- **When:** `clv .version.install dry::1`
- **Then:** exit 0; stdout contains `[dry-run]`
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### IT-2: `dry::1 force::1` → dry wins

- **Given:** clean environment
- **When:** `clv .version.install dry::1 force::1`
- **Then:** exit 0; stdout contains `[dry-run]`; no actual install executed
- **Exit:** 0
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### IT-3: `version::STABLE` → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::STABLE`
- **Then:** exit 1; version alias is case-sensitive; wrong-case alias rejected
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-4: `version::01.02.03` → leading zeros rejected

- **Given:** clean environment
- **When:** `clv .version.install version::01.02.03`
- **Then:** exit 1; leading-zero semver is invalid per VersionSpec rules
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-5: `bogus::x` → unknown parameter, exit 1

- **Given:** clean environment
- **When:** `clv .version.install bogus::x`
- **Then:** exit 1; unknown parameter rejected by adapter
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-6: `dry::1 format::json` → JSON object output

- **Given:** clean environment
- **When:** `clv .version.install dry::1 format::json`
- **Then:** exit 0; stdout starts with `{`; valid JSON object
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### IT-7: `format::JSON` (uppercase) → exit 1

- **Given:** clean environment
- **When:** `clv .version.install format::JSON`
- **Then:** exit 1; format value is case-sensitive; uppercase rejected
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-8: `dry::2` → out-of-range boolean, exit 1

- **Given:** clean environment
- **When:** `clv .version.install dry::2`
- **Then:** exit 1; boolean value out of range (must be 0 or 1)
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-9: `version::stable dry::1` → preview shows `stable`

- **Given:** clean environment
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; stdout contains `[dry-run]` and mentions `stable`
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-10: `version::1.2.3 dry::1` → preview shows exact version

- **Given:** clean environment
- **When:** `clv .version.install version::1.2.3 dry::1`
- **Then:** exit 0; stdout contains `[dry-run]` and `1.2.3`; semver preserved verbatim
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-23: `version::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::`
- **Then:** exit 1; empty version value rejected
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-24: `version::1.2` → two-part semver, exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::1.2`
- **Then:** exit 1; two-part semver is not a valid VersionSpec
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-25: `version::x` → unknown alias, exit 1

- **Given:** clean environment
- **When:** `clv .version.install version::x`
- **Then:** exit 1; unrecognized alias rejected
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-11: Absent `version::` with `dry::1` → defaults to `stable`

- **Given:** clean environment
- **When:** `clv .version.install dry::1` (no `version::` parameter)
- **Then:** exit 0; stdout mentions `stable` as the resolved default version
- **Exit:** 0
- **Source:** [param/readme.md — version:: default: stable](../../../../docs/cli/param/readme.md)

---

### IT-12: `version::month dry::1` → resolves to pinned semver

- **Given:** clean environment
- **When:** `clv .version.install version::month dry::1`
- **Then:** exit 0; stdout contains pinned semver for the `month` alias (e.g., `2.1.74`)
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-13: `version::latest dry::1` → autoUpdates=true in preview

- **Given:** clean environment
- **When:** `clv .version.install version::latest dry::1`
- **Then:** exit 0; preview output indicates unlock action (`autoUpdates=true`)
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-14: `version::stable dry::1` → autoUpdates=false in preview

- **Given:** clean environment
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; preview output indicates lock action (`autoUpdates=false`)
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-15: `version::2.1.50 dry::1` → autoUpdates=false in preview

- **Given:** clean environment
- **When:** `clv .version.install version::2.1.50 dry::1`
- **Then:** exit 0; preview shows lock action; semver triggers 5-layer lock
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-16: `version::latest dry::1` → previews unlock actions

- **Given:** clean environment
- **When:** `clv .version.install version::latest dry::1`
- **Then:** exit 0; stdout describes unlock steps (remove DISABLE_AUTOUPDATER, chmod 755)
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-17: `version::0.0.0 dry::1` → single-zero parts valid

- **Given:** clean environment
- **When:** `clv .version.install version::0.0.0 dry::1`
- **Then:** exit 0; `0.0.0` is a valid semver (zeros in all parts are accepted)
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-18: `dry::1` output mentions preferred version storage

- **Given:** clean environment
- **When:** `clv .version.install dry::1`
- **Then:** exit 0; stdout references preference storage action (preview only, not written)
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### IT-19: `dry::1` does NOT write preference keys

- **Given:** `HOME=<tmp>`; `settings.json` starts empty
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; `settings.json` has no `preferredVersionSpec` key after command
- **Exit:** 0
- **Source:** [feature/004_dry_run.md](../../../../docs/feature/004_dry_run.md)

---

### IT-20: Idempotent skip still stores preference

- **Given:** `HOME=<tmp>`; target version already installed
- **When:** `clv .version.install version::stable`
- **Then:** exit 0; `settings.json` contains `preferredVersionSpec` = `"stable"`
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-21: `version::stable dry::1` → output includes purge line

- **Given:** clean environment
- **When:** `clv .version.install version::stable dry::1`
- **Then:** exit 0; stdout includes a purge/cleanup step (layer 4 of 5-layer lock)
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-22: `version::latest dry::1` → output does NOT contain "purge"

- **Given:** clean environment
- **When:** `clv .version.install version::latest dry::1`
- **Then:** exit 0; stdout does NOT contain "purge" (unlocking does not purge binaries)
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc300_version_install_dry_shows_prefix` | `tests/cli/mutation_version_install_test.rs` |
| `tc301_version_install_dry_stable` | `tests/cli/mutation_version_install_test.rs` |
| `tc302_version_install_dry_exact_semver` | `tests/cli/mutation_version_install_test.rs` |
| `tc303_version_install_dry_wins_over_force` | `tests/cli/mutation_version_install_test.rs` |
| `tc304_version_install_wrong_case_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc305_version_install_empty_version_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc306_version_install_two_part_semver_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc307_version_install_unknown_alias_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc308_version_install_absent_version_defaults_to_stable` | `tests/cli/mutation_version_install_test.rs` |
| `tc309_version_install_dry_month` | `tests/cli/mutation_version_install_test.rs` |
| `tc350_version_install_dry_latest_auto_updates_true` | `tests/cli/mutation_version_install_test.rs` |
| `tc351_version_install_dry_stable_auto_updates_false` | `tests/cli/mutation_version_install_test.rs` |
| `tc352_version_install_dry_semver_auto_updates_false` | `tests/cli/mutation_version_install_test.rs` |
| `tc353_version_install_dry_latest_shows_unlock` | `tests/cli/mutation_version_install_test.rs` |
| `tc354_version_install_leading_zeros_exits_1` | `tests/cli/mutation_version_install_test.rs` |
| `tc355_version_install_zero_parts_valid_dry` | `tests/cli/mutation_version_install_test.rs` |
| `tc356_version_install_dry_mentions_preferred` | `tests/cli/mutation_version_install_test.rs` |
| `tc357_version_install_dry_no_preference_written` | `tests/cli/mutation_version_install_test.rs` |
| `tc358_version_install_idempotent_stores_preference` | `tests/cli/mutation_version_install_test.rs` |
| `tc359_version_install_dry_stable_includes_purge_line` | `tests/cli/mutation_version_install_test.rs` |
| `tc360_version_install_dry_latest_no_purge_line` | `tests/cli/mutation_version_install_test.rs` |
| `tc361_version_install_dry_format_json` | `tests/cli/mutation_version_install_test.rs` |
| `tc362_version_install_format_uppercase_rejected` | `tests/cli/mutation_version_install_test.rs` |
| `tc510_version_install_wrong_case_error` | `tests/cli/error_messages_test.rs` |
