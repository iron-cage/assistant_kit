# Test: `.version.guard`

Integration test planning for the `.version.guard` command. See [commands.md](../../../../../docs/cli/commands.md#command--15-versionguard) for specification.

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
| EC-1 | HOME not set → defaults to stable, exit 0 | Empty State |
| EC-2 | `interval::0` behaves as one-shot (default) | One-Shot |
| EC-3 | Preference `latest` → verifies auto-update config | Latest Handling |
| EC-4 | `dry::1` with `latest` → preview message | Dry Run |
| EC-5 | Malformed settings → graceful degradation | Error Handling |
| EC-6 | `preferredVersionResolved` absent but spec present → graceful | Error Handling |
| TC-410 | Stale `preferredVersionResolved` → guard re-resolves alias | Bug Fix |
| TC-411 | `version::9.9.9 dry::1` → override shown in output, exit 0 | Version Override |
| TC-412 | `bogus::x` → unknown parameter, exit 1 | Error Handling |
| TC-413 | `version::9.9.9 force::1 dry::1` → dry wins, override shown | Version Override |
| TC-414 | `version::` (empty value) → exit 1, stderr mentions version | Error Handling |
| TC-415 | watch loop continues after install error, not terminated | Watch Resilience |
| TC-416 | `version::latest dry::1` override → "no version pin to guard" | Version Override |
| TC-417 | `dry::1 v::0` → output shorter than `v::1` | Output Control |
| TC-418 | `format::json dry::1` → JSON object output, exit 0 | Format |

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

**Goal:** When no preferred version has been stored, the guard defaults to `stable`.
**Setup:** `~/.claude/settings.json` exists but contains no `preferredVersionSpec` key.
**Command:** `cm .version.guard`
**Expected Output:** output contains "stable"
**Verification:**
- exit code 0
- output contains "stable"
**Pass Criteria:** Exit 0; guard defaults to stable channel.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### IT-2: `dry::1` no preference → defaults to stable, exit 0

**Goal:** Dry-run with no preference stored defaults to stable like non-dry.
**Setup:** Empty settings.
**Command:** `cm .version.guard dry::1`
**Expected Output:** output contains "stable"
**Verification:**
- exit code 0
- output contains "stable"
**Pass Criteria:** Exit 0; defaults to stable regardless of dry flag.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### IT-3: Installed version matches preferred → "matches", exit 0

**Goal:** When the installed version equals the preferred resolved version, no action is taken.
**Setup:** `claude` installed at version X; `preferredVersionResolved = "X"` in settings.
**Command:** `cm .version.guard`
**Expected Output:** `version X matches preferred ...`
**Verification:**
- exit code 0
- output contains "matches"
- no install occurs
**Pass Criteria:** Exit 0; match message present.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### IT-4: Drift detected → reinstalls preferred version, exit 0

**Goal:** When installed version differs from preferred, the guard restores it.
**Setup:** `claude` at version A; `preferredVersionResolved = "B"` in settings; network available.
**Command:** `cm .version.guard`
**Expected Output:** Drift message followed by restoration confirmation.
**Verification:**
- exit code 0
- output contains "drift" or "restored"
- installed version changes to B
**Pass Criteria:** Exit 0; preferred version restored.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### IT-5: `dry::1` drift → preview without reinstall

**Goal:** Dry-run shows drift detection without executing install.
**Setup:** Installed version differs from preferred.
**Command:** `cm .version.guard dry::1`
**Expected Output:** `[dry-run] drift detected: ...` and `[dry-run] would reinstall ...`
**Verification:**
- exit code 0
- output contains `[dry-run]` prefix
- installed version unchanged
**Pass Criteria:** Exit 0; dry-run markers present; no side effects.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### IT-6: `force::1` → reinstalls even when version matches

**Goal:** Force bypasses the match check and reinstalls unconditionally.
**Setup:** Installed version matches preferred; network available.
**Command:** `cm .version.guard force::1`
**Expected Output:** Install proceeds despite version match.
**Verification:**
- exit code 0
- output does not contain "matches" skip message
**Pass Criteria:** Exit 0; install proceeds.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### IT-7: `dry::1 force::1` → dry wins, no install

**Goal:** `dry::` takes precedence over `force::`.
**Setup:** Preference set with pinned version.
**Command:** `cm .version.guard dry::1 force::1`
**Expected Output:** Dry-run preview; no install.
**Verification:**
- exit code 0
- output contains `[dry-run]`
- installed version unchanged
**Pass Criteria:** Exit 0; dry-run output only.
**Source:** [parameter_interactions.md — dry+force precedence](../../../../../docs/cli/parameter_interactions.md)

---

### IT-8: Exit 0 on successful guard check

**Goal:** Successful guard operation always exits 0.
**Setup:** Any valid state (preference set or absent).
**Command:** `cm .version.guard; echo $?`
**Expected Output:** `0` on the last line.
**Verification:**
- `$?` equals `0`
**Pass Criteria:** Exit code 0.
**Source:** [commands.md — Exit: 0](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### EC-1: HOME not set → defaults to stable, exit 0

**Goal:** Missing HOME degrades gracefully, defaults to stable.
**Setup:** HOME unset or empty.
**Command:** `cm .version.guard`
**Expected Output:** output contains "stable"
**Verification:**
- exit code 0
- no panic or crash
**Pass Criteria:** Exit 0; graceful degradation with stable default.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### EC-2: `interval::0` behaves as one-shot (default)

**Goal:** Explicit `interval::0` is identical to omitting the parameter.
**Setup:** Empty settings.
**Command:** `cm .version.guard interval::0`
**Expected Output:** Same as one-shot mode.
**Verification:**
- exit code 0
- process exits immediately (does not loop)
**Pass Criteria:** Exit 0; one-shot behavior.
**Source:** [params.md — interval::](../../../../../docs/cli/params.md#parameter--9-interval)

---

### EC-3: Preference `latest` → verifies auto-update config

**Goal:** For `latest` preference, the guard checks auto-update settings instead of version comparison.
**Setup:** `preferredVersionSpec = "latest"`, `preferredVersionResolved = null` in settings.
**Command:** `cm .version.guard`
**Expected Output:** Message about latest preference and auto-update status.
**Verification:**
- exit code 0
- output mentions "latest"
**Pass Criteria:** Exit 0; latest-specific behavior.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### EC-4: `dry::1` with `latest` → preview message

**Goal:** Dry-run with latest preference shows informational message.
**Setup:** `preferredVersionSpec = "latest"` in settings.
**Command:** `cm .version.guard dry::1`
**Expected Output:** Message about latest preference, no side effects.
**Verification:**
- exit code 0
- output mentions "latest"
- no config changes made
**Pass Criteria:** Exit 0; informational output only.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### EC-5: Malformed settings → graceful degradation

**Goal:** Corrupt or unparseable settings file does not crash the guard.
**Setup:** `settings.json` contains invalid JSON.
**Command:** `cm .version.guard`
**Expected Output:** "no preferred version set" or error message; no crash.
**Verification:**
- exit code 0 or 2
- no panic or stack trace
**Pass Criteria:** No crash; clean exit.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### EC-6: `preferredVersionResolved` absent but spec present → graceful

**Goal:** Incomplete preference data (spec without resolved) is handled gracefully.
**Setup:** `preferredVersionSpec = "stable"` but no `preferredVersionResolved` key.
**Command:** `cm .version.guard`
**Expected Output:** Guard treats resolved as absent and handles accordingly.
**Verification:**
- no crash
- exit code 0 or 2
**Pass Criteria:** Graceful handling; no panic.
**Source:** [commands.md — .version.guard](../../../../../docs/cli/commands.md#command--15-versionguard)

---

### TC-410: Stale `preferredVersionResolved` → guard re-resolves alias

**Goal:** After an alias bump, stored `preferredVersionResolved` becomes stale. Guard must re-resolve through current compile-time table.
**Setup:** `preferredVersionSpec = "month"`, `preferredVersionResolved = "2.1.50"` (stale — current month is 2.1.74).
**Command:** `cm .version.guard dry::1`
**Expected Output:** Output references `2.1.74` (current alias value), not `2.1.50` (stale stored value).
**Verification:**
- exit code 0
- output contains `2.1.74`
- output does NOT contain `2.1.50`
**Pass Criteria:** Guard uses re-resolved alias value, ignoring stale stored resolution.
**Source:** [commands.rs — guard_once() re-resolution fix](../../../../../src/commands.rs)

---

### TC-411: `version::9.9.9 dry::1` → override shown in output, exit 0

**Goal:** `version::` parameter overrides the stored preference for this invocation without modifying `settings.json`.
**Setup:** Empty settings (no stored preference).
**Command:** `cm .version.guard version::9.9.9 dry::1`
**Expected Output:** Output mentions `9.9.9`.
**Verification:**
- exit code 0
- output contains `9.9.9`
- `settings.json` unchanged
**Pass Criteria:** Exit 0; override version appears in output.
**Source:** [feature/001_version_management.md](../../../../../docs/feature/001_version_management.md), [commands.md — Command :: 5](../../../../../docs/cli/commands.md#command--5-versionguard)

---

### TC-412: `bogus::x` → unknown parameter, exit 1

**Goal:** Unknown parameters to `.version.guard` are rejected with exit 1 (SemanticAnalyzer enforcement).
**Setup:** None.
**Command:** `cm .version.guard bogus::x`
**Expected Output:** stderr contains "unknown parameter"
**Verification:**
- exit code 1
- stderr contains error message
**Pass Criteria:** Exit 1; unknown-param rejection.
**Source:** [feature/005_cli_design.md](../../../../../docs/feature/005_cli_design.md)

---

### TC-413: `version::9.9.9 force::1 dry::1` → dry wins, override shown

**Goal:** `dry::1` takes precedence over `force::1` even when `version::` override is present.
**Setup:** Empty settings.
**Command:** `cm .version.guard version::9.9.9 force::1 dry::1`
**Expected Output:** `[dry-run]` prefix in output; `9.9.9` mentioned.
**Verification:**
- exit code 0
- output contains `9.9.9`
- output contains `[dry-run]`
- no install occurs
**Pass Criteria:** Exit 0; dry-run wins; override version shown.
**Source:** [parameter_interactions.md — dry+force precedence](../../../../../docs/cli/parameter_interactions.md)

---

### TC-414: `version::` (empty value) → exit 1, stderr mentions version

**Goal:** Empty `version::` value is rejected before any guard logic runs.
**Setup:** None.
**Command:** `cm .version.guard version::`
**Expected Output:** stderr mentions "version"; exit 1.
**Verification:**
- exit code 1
- stderr contains "version" (case-insensitive)
**Pass Criteria:** Exit 1; validation error with version context.
**Source:** [feature/001_version_management.md](../../../../../docs/feature/001_version_management.md)

---

### TC-415: watch loop continues after install error

**Goal:** A transient install failure during watch mode must not terminate the guard daemon; the loop must log the error, sleep the interval, and retry.
**Setup:** `~/.claude/settings.json` contains `preferredVersionSpec="9.9.9"` / `preferredVersionResolved="9.9.9"`; no claude binary installed; empty `PATH` (forces install failure).
**Command:** `timeout 2 cm .version.guard interval::1`
**Expected Output:** Process runs until killed by `timeout` (2 seconds); stderr contains `#1` and `#2` iteration headers.
**Verification:**
- exit code 124 (killed by timeout, not self-exited on error)
- stderr contains `#2` (second iteration was reached)
- no premature exit with code 2
**Pass Criteria:** Process survives first install error; daemon continues watching.
**Bug:** `return result` in watch loop error branch — fixed by continuing the loop instead.
**Source:** [commands.rs — version_guard_routine watch loop](../../../../../src/commands.rs), [feature/001_version_management.md](../../../../../docs/feature/001_version_management.md)

---

### TC-416: `version::latest dry::1` override → "no version pin to guard"

**Goal:** `version::` override with `latest` bypasses settings lookup and delegates to `guard_once_latest`, which outputs "no version pin to guard".
**Setup:** Empty settings (no `preferredVersionSpec` — override must not read from settings per FR-21).
**Command:** `cm .version.guard version::latest dry::1`
**Expected Output:** stdout contains "no version pin to guard"; exit 0.
**Verification:**
- exit code 0
- stdout contains "no version pin to guard"
**Pass Criteria:** Exit 0; latest-override message present.
**Note:** Distinct from TC-403 (settings-driven `preferredVersionSpec = "latest"`); this test exercises the `version::` override dispatch path exclusively.
**Source:** [commands.rs — version_guard_routine override dispatch](../../../../../src/commands.rs)

---

### TC-417: `dry::1 v::0` → output shorter than `v::1`

**Goal:** Verbosity 0 produces compact output; verbosity 1 produces labeled output. `.version.guard` honours the `v::` parameter after TSK-099 output-control wiring.
**Setup:** Empty settings (no preference → defaults to stable).
**Command:** `cm .version.guard dry::1 v::0` vs `cm .version.guard dry::1 v::1`
**Expected:** `v::0` stdout char count < `v::1` stdout char count.
**Verification:** `s0.len() < s1.len()`.
**Pass Criteria:** v::0 output shorter than v::1 for same guard invocation.
**Source:** [commands.rs — version_guard_routine opts.verbosity](../../../../../src/commands.rs)
