# Test: `.version.guard`

### Scope

- **Purpose**: Integration test cases for the `.version.guard` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for version drift detection.
- **In Scope**: One-shot mode, watch mode, preference override, dry-run, force, drift restoration.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.version.guard` command. See [command/readme.md](../../../../docs/cli/command/readme.md#command--15-versionguard) for specification.

## Parameter Analysis

### `verbosity::` / `v::` (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: labeled output | Default behavior |
| 0 | Bare/compact output | Compact |
| 1 | Labeled output | Labeled |

### `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: text output | Default behavior |
| `text` | Human-readable text | Valid |
| `json` | Machine-readable JSON | Valid |

---

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No preference set → defaults to stable, exit 0 | Empty State |
| IT-2 | `dry::1` no preference → defaults to stable, exit 0 | Dry Run |
| IT-3 | Installed version matches preferred → "matches", exit 0 | Happy Path |
| IT-4 | Drift detected → reinstalls preferred version, exit 0 | Happy Path |
| IT-5 | `dry::1` drift → preview without reinstall | Dry Run |
| IT-6 | `force::1` → reinstalls even when version matches | Force Behavior |
| IT-7 | `dry::1 force::1` → dry wins, no install | Dry Run |
| IT-8 | Exit 0 on successful guard check | Exit Codes |
| IT-9 | HOME not set → defaults to stable, exit 0 | Empty State |
| IT-10 | `interval::0` behaves as one-shot (default) | One-Shot |
| IT-11 | Preference `latest` → verifies auto-update config | Latest Handling |
| IT-12 | `dry::1` with `latest` → preview message | Dry Run |
| IT-13 | Malformed settings → graceful degradation | Error Handling |
| IT-14 | `preferredVersionResolved` absent but spec present → graceful | Error Handling |
| IT-15 | Stale `preferredVersionResolved` → guard re-resolves alias | Bug Fix |
| IT-16 | `version::9.9.9 dry::1` → override shown in output, exit 0 | Version Override |
| IT-17 | `bogus::x` → unknown parameter, exit 1 | Error Handling |
| IT-18 | `version::9.9.9 force::1 dry::1` → dry wins, override shown | Version Override |
| IT-19 | `version::` (empty value) → exit 1, stderr mentions version | Error Handling |
| IT-20 | watch loop continues after install error, not terminated | Watch Resilience |
| IT-21 | `version::latest dry::1` override → "no version pin to guard" | Version Override |
| IT-22 | `dry::1 v::0` → output shorter than `v::1` | Output Control |
| IT-23 | `format::json dry::1` → JSON object output, exit 0 | Format |

## Test Coverage Summary

- Happy Path: 2 tests
- Dry Run: 4 tests
- Empty State: 2 tests
- Force Behavior: 1 test
- Exit Codes: 1 test
- One-Shot: 1 test
- Latest Handling: 1 test
- Error Handling: 3 tests
- Bug Fix: 1 test
- Version Override: 3 tests
- Watch Resilience: 1 test
- Output Control: 1 test
- Format: 1 test

**Total:** 23 tests (8 integration, 6 edge cases, 1 bug fix, 5 version override/error, 1 watch resilience, 1 output control, 1 format)

---

### IT-1: No preference set → defaults to stable, exit 0

- **Given:** `~/.claude/settings.json` exists but contains no `preferredVersionSpec` key.
- **When:** `clv .version.guard`
- **Then:** output contains "stable"; guard defaults to stable channel
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-2: `dry::1` no preference → defaults to stable, exit 0

- **Given:** Empty settings.
- **When:** `clv .version.guard dry::1`
- **Then:** output contains "stable"; defaults to stable regardless of dry flag
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-3: Installed version matches preferred → "matches", exit 0

- **Given:** `claude` installed at version X; `preferredVersionResolved = "X"` in settings.
- **When:** `clv .version.guard`
- **Then:** `version X matches preferred ...`; match message present
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-4: Drift detected → reinstalls preferred version, exit 0

- **Given:** `claude` at version A; `preferredVersionResolved = "B"` in settings; network available.
- **When:** `clv .version.guard`
- **Then:** Drift message followed by restoration confirmation.; preferred version restored
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-5: `dry::1` drift → preview without reinstall

- **Given:** Installed version differs from preferred.
- **When:** `clv .version.guard dry::1`
- **Then:** `[dry-run] drift detected: ...` and `[dry-run] would reinstall ...`; dry-run markers present; no side effects
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-6: `force::1` → reinstalls even when version matches

- **Given:** Installed version matches preferred; network available.
- **When:** `clv .version.guard force::1`
- **Then:** Install proceeds despite version match.
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-7: `dry::1 force::1` → dry wins, no install

- **Given:** Preference set with pinned version.
- **When:** `clv .version.guard dry::1 force::1`
- **Then:** Dry-run preview; no install.; dry-run output only
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force precedence](../../../../docs/cli/004_parameter_interactions.md)

---

### IT-8: Exit 0 on successful guard check

- **Given:** Any valid state (preference set or absent).
- **When:** `clv .version.guard; echo $?`
- **Then:** `0` on the last line.; Exit code 0
- **Exit:** 0
- **Source:** [command/readme.md — Exit: 0](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-9: HOME not set → defaults to stable, exit 0

- **Given:** HOME unset or empty.
- **When:** `clv .version.guard`
- **Then:** output contains "stable"; graceful degradation with stable default
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-10: `interval::0` behaves as one-shot (default)

- **Given:** Empty settings.
- **When:** `clv .version.guard interval::0`
- **Then:** Same as one-shot mode.; one-shot behavior
- **Exit:** 0
- **Source:** [param/readme.md — interval::](../../../../docs/cli/param/readme.md#parameter--9-interval)

---

### IT-11: Preference `latest` → verifies auto-update config

- **Given:** `preferredVersionSpec = "latest"`, `preferredVersionResolved = null` in settings.
- **When:** `clv .version.guard`
- **Then:** Message about latest preference and auto-update status.; latest-specific behavior
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-12: `dry::1` with `latest` → preview message

- **Given:** `preferredVersionSpec = "latest"` in settings.
- **When:** `clv .version.guard dry::1`
- **Then:** Message about latest preference, no side effects.; informational output only
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-13: Malformed settings → graceful degradation

- **Given:** `settings.json` contains invalid JSON.
- **When:** `clv .version.guard`
- **Then:** "no preferred version set" or error message; no crash.; No crash; clean exit
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-14: `preferredVersionResolved` absent but spec present → graceful

- **Given:** `preferredVersionSpec = "stable"` but no `preferredVersionResolved` key.
- **When:** `clv .version.guard`
- **Then:** Guard treats resolved as absent and handles accordingly.; Graceful handling; no panic
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### IT-15: Stale `preferredVersionResolved` → guard re-resolves alias

- **Given:** `preferredVersionSpec = "month"`, `preferredVersionResolved = "2.1.50"` (stale — current month is 2.1.74).
- **When:** `clv .version.guard dry::1`
- **Then:** Output references `2.1.74` (current alias value), not `2.1.50` (stale stored value).; Guard uses re-resolved alias value, ignoring stale stored resolution
- **Exit:** 0
- **Source:** [commands/version.rs — guard_once() re-resolution fix](../../../../src/commands/version.rs)

---

### IT-16: `version::9.9.9 dry::1` → override shown in output, exit 0

- **Given:** Empty settings (no stored preference).
- **When:** `clv .version.guard version::9.9.9 dry::1`
- **Then:** Output mentions `9.9.9`.; override version appears in output
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md), [command/readme.md — Command :: 5](../../../../docs/cli/command/readme.md#command--5-versionguard)

---

### IT-17: `bogus::x` → unknown parameter, exit 1

- **Given:** clean environment
- **When:** `clv .version.guard bogus::x`
- **Then:** stderr contains "unknown parameter"; unknown-param rejection
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-18: `version::9.9.9 force::1 dry::1` → dry wins, override shown

- **Given:** Empty settings.
- **When:** `clv .version.guard version::9.9.9 force::1 dry::1`
- **Then:** `[dry-run]` prefix in output; `9.9.9` mentioned.; dry-run wins; override version shown
- **Exit:** 0
- **Source:** [004_parameter_interactions.md — dry+force precedence](../../../../docs/cli/004_parameter_interactions.md)

---

### IT-19: `version::` (empty value) → exit 1, stderr mentions version

- **Given:** clean environment
- **When:** `clv .version.guard version::`
- **Then:** stderr mentions "version"; exit 1.; validation error with version context
- **Exit:** 1
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-20: watch loop continues after install error

- **Given:** `~/.claude/settings.json` contains `preferredVersionSpec="9.9.9"` / `preferredVersionResolved="9.9.9"`; no claude binary installed; empty `PATH` (forces install failure).
- **When:** `timeout 2 cm .version.guard interval::1`
- **Then:** Process runs until killed by `timeout` (2 seconds); stderr contains `#1` and `#2` iteration headers.; Process survives first install error; daemon continues watching.
- **Bug:** `return result` in watch loop error branch — fixed by continuing the loop instead
- **Exit:** 0
- **Source:** [commands/version.rs — version_guard_routine watch loop](../../../../src/commands/version.rs), [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### IT-21: `version::latest dry::1` override → "no version pin to guard"

- **Given:** Empty settings (no `preferredVersionSpec` — override must not read from settings per FR-21).
- **When:** `clv .version.guard version::latest dry::1`
- **Then:** stdout contains "no version pin to guard"; exit 0.; latest-override message present.
- **Note:** Distinct from the settings-driven `preferredVersionSpec = "latest"` case; this test exercises the `version::` override dispatch path exclusively
- **Exit:** 0
- **Source:** [commands/version.rs — version_guard_routine override dispatch](../../../../src/commands/version.rs)

---

### IT-22: `dry::1 v::0` → output shorter than `v::1`

- **Given:** Empty settings (no preference → defaults to stable).
- **When:** `clv .version.guard dry::1 v::0` vs `clv .version.guard dry::1 v::1`
- **Expected:** `v::0` stdout char count < `v::1` stdout char count.
- **Then:** v::0 output shorter than v::1 for same guard invocation
- **Exit:** 0
- **Source:** [commands/version.rs — version_guard_routine opts.verbosity](../../../../src/commands/version.rs)

---

### IT-23: `format::json dry::1` → JSON object output, exit 0

- **Given:** clean environment (no stored preference)
- **When:** `clv .version.guard format::json dry::1`
- **Then:** exit 0; stdout starts with `{`; valid JSON object output
- **Exit:** 0
- **Source:** [command/readme.md — .version.guard](../../../../docs/cli/command/readme.md#command--15-versionguard)

---

### Source Functions

| Function | File |
|----------|------|
| `tc400_guard_no_preference` | `integration/mutation_commands_test.rs` |
| `tc401_guard_dry_no_preference` | `integration/mutation_commands_test.rs` |
| `tc402_guard_no_home` | `integration/mutation_commands_test.rs` |
| `tc403_guard_latest_dry` | `integration/mutation_commands_test.rs` |
| `tc406_guard_dry_force_no_install` | `integration/mutation_commands_test.rs` |
| `tc409_guard_interval_zero_oneshot` | `integration/mutation_commands_test.rs` |
| `tc410_guard_reresoves_stale_alias` | `integration/mutation_commands_test.rs` |
| `tc411_guard_version_override_dry` | `integration/mutation_commands_test.rs` |
| `tc412_guard_unknown_param_rejected` | `integration/mutation_commands_test.rs` |
| `tc413_guard_version_override_dry_wins_over_force` | `integration/mutation_commands_test.rs` |
| `tc414_guard_version_empty_rejected` | `integration/mutation_commands_test.rs` |
| `tc415_watch_loop_continues_after_install_error` | `integration/mutation_commands_test.rs` |
| `tc416_guard_version_latest_override_dry` | `integration/mutation_commands_test.rs` |
| `tc417_guard_v0_shorter_than_v1` | `integration/mutation_commands_test.rs` |
| `tc418_guard_format_json_dry` | `integration/mutation_commands_test.rs` |
