# Task 007: Show Lock-State Compliance in `.status`

## Execution State

- **Executor Type:** any
- **filed_by:** dev
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ❓ (Unverified)
- **closes:** Q-02
- **unit_type:** module
- **unit:** lib/yrd_core/assistant_kit/claude_version/module/claude_version
- **validated_by:** null
- **validation_date:** null

## Goal

Extend the existing `v::2` tier of `status_routine()` (`module/claude_version/src/commands/status.rs:68-92`) to report actual-vs-expected compliance for every version-pinning mechanism key, so a user can answer "is my pin actually holding?" without manually inspecting `settings.json` and `chmod`. **Motivated:** today `.status` reports only the *preferred* version spec (what the user asked for), never whether the lock mechanisms that are supposed to enforce that preference are actually in place — drift (e.g. `chmod` reset to `755` by an unrelated process, or `autoUpdates` flipped back to `true`) is invisible until an unwanted update happens. **Observable:** `clv .status v::2` against a pinned install prints a `Lock:` section listing each mechanism key (`autoUpdates`, `env.DISABLE_AUTOUPDATER`, plus `autoUpdatesChannel`/`minimumVersion`/`env.DISABLE_UPDATES` once Task 005 lands, and the `chmod` mode of the versions directory) with its actual value next to the value implied by the current pin state, flagging any mismatch. **Scoped:** read-only extension of `.status` output at the existing `v::2` tier and in the JSON arm — no new command, no mutation, no repair action. **Testable:** `verb/test_only status` plus new cases in `claude_version/tests/cli/read_status_test.rs` covering all 6 Test Matrix rows below, and a corresponding Factor + TC update in `tests/docs/cli/command/02_status.md`.

## In Scope

- `module/claude_version/src/commands/status.rs` — extend the `(OutputFormat::Text, v)` arm's `v >= 2` branch (line 81-88) with a `Lock:` section; extend the `(OutputFormat::Json, _)` arm (line 45-59) with a `"lock"` object, unconditionally (matching the existing `"preferred"` object's verbosity-invariant JSON precedent)
- A new read-only helper in `claude_version_core` (e.g. in `version.rs`) that reads the current `chmod` mode of `versions_dir_path()` — pure read, no mutation, so it is NOT one of Task 006's 10 traced mutating functions
- Reading existing lock-mechanism keys via the already-available `get_setting()` (`settings_io.rs:101`) — no new write path, no new settings keys
- Updating `module/claude_version/tests/docs/cli/command/02_status.md` — new Factor (lock-state compliance) plus new TC rows in the same file, as part of this task's own delivery scope (test spec update accompanies the behavior it specs)

## Out of Scope

- A new dedicated command (e.g. `.version.lock_status`) — `.status` already owns "installation state" responsibility (Layer 5 preferred-version reporting already lives here); a second command would duplicate that responsibility, failing the One-Second Test. Per `task/decisions.md` Q-02
- Introducing a new `v::3` verbosity tier — `tests/docs/cli/command/02_status.md` Factor 1 boundary set already documents `v::3` as invalid/out-of-range (`IT-11`, exit 1); extending the valid range would break that existing contract. This task instead extends the already-valid `v::2` tier's content
- Any mutation or repair action (e.g. auto-fixing a detected mismatch by re-applying `chmod` or rewriting settings) — `.status` is documented as read-only by design (FR-01 in `tests/docs/cli/command/02_status.md` § Degradation Semantics: "status is read-only, never fails"); repair belongs to `.version.install`/`.version.guard`, not `.status`
- Changes to `v::0` or `v::1` output — both stay exactly as they are today; only `v::2` and the JSON arm gain new content
- Waiting on Task 005 to land first — this task reads whichever lock keys are actually present via `get_setting()` and reports absence honestly; it is correct whether it executes before or after Task 005

## Null Hypothesis

Do nothing — `.status` continues to report only the preferred version spec, with no visibility into whether the lock mechanisms enforcing it are actually intact.

**Disproof:** The Mechanism Coverage table and this session's Task 005 both establish that multiple independent keys/layers now (or soon will) enforce a pin — a `chmod` reset or a flipped `autoUpdates` boolean silently breaks enforcement while `preferredVersionSpec` keeps reporting the user's original intent unchanged, producing a false sense of security. This is a concrete, already-identified visibility gap, not speculative hardening — the user asked for "a command to see how [pinning] is applied," which this task directly answers by extending the existing status surface rather than inventing a new one. Disproof would require showing lock-mechanism drift cannot occur in practice, which contradicts the very existence of `docs/pitfall/002_symlink_retarget.md` and `docs/pitfall/001_version_lock_chmod.md` (both document real drift scenarios already observed in this project).

## Requirements

- `.status v::2` MUST print a `Lock:` section reporting, for each lock-mechanism key present in `settings.json`, its actual value alongside whether that value matches what the current pin state implies
- `.status v::2` MUST report the versions directory's actual `chmod` mode (`555`/`755`) alongside the mode implied by the current pin state
- `.status format::json` MUST include a `"lock"` object with the same information, regardless of `v::`
- `.status v::0` and `.status v::1` output MUST remain byte-for-byte unchanged (no regression)
- `.status v::3` MUST continue to exit 1 as out-of-range (no regression of the existing boundary)
- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the Work Procedure below.

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Test Matrix populated before any test code
- All Test Matrix cases implemented as failing tests before implementation
- Minimum code to satisfy Test Matrix — no features beyond requirements
- `verb/test` passes with zero failures and zero warnings
- No function exceeds 50 lines; no duplication; public items have `///` doc comments
- Independent validation passes via MAAV (this rulebook's Verification Gate) — never self-verified
- Task state updated to 🎯 on validation pass; file moved from `task/unverified/` to `task/` root
- `task/decisions.md` Q-02 state updated to reflect this task closing it (via the task's own completion, not a separate DECIDE/CONFIRM MAAV cycle — out of this task's scope)

### Work Procedure

1. Read `module/claude_version/src/commands/status.rs` in full (current `v::2`/JSON arms) and `claude_version_core/src/version.rs` `versions_dir_path()`/`lock_version()` (current chmod mode semantics)
2. Write the 6 Test Matrix rows below as failing tests in `claude_version/tests/cli/read_status_test.rs`
3. Add a read-only chmod-mode-read helper in `claude_version_core` (returns locked/unlocked/unknown)
4. Extend `status_routine()`'s `v >= 2` text branch and the JSON arm with the `Lock:`/`"lock"` content, comparing actual vs. pin-implied expected values for each present key
5. Run `verb/test_only status` until all 6 rows pass, and confirm `v::0`/`v::1`/`v::3` cases (existing IT-3/IT-4/IT-11) still pass unchanged
6. Update `module/claude_version/tests/docs/cli/command/02_status.md` — add a new Factor for lock-state compliance and new TC rows covering the 6 scenarios, updating the Test Matrix Summary counts
7. Run full `verb/test` (all crates) and confirm zero failures, zero warnings

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `.status v::2`, pinned install, all keys compliant | versions dir `chmod 555`, all pin keys set correctly | `Lock:` section shows every key as compliant, no mismatch flagged |
| T02 | `.status v::2`, pinned install, `chmod` drifted to `755` | versions dir `chmod 755` while pin keys still set | `Lock:` section flags the chmod layer as mismatched (expected `555`, actual `755`) |
| T03 | `.status v::2`, pinned install, `autoUpdates` flipped back to `true` | settings.json `autoUpdates=true` while other pin keys set | `Lock:` section flags `autoUpdates` as mismatched |
| T04 | `.status v::2`, unpinned (`latest`) install | no pin keys set, versions dir `chmod 755` | `Lock:` section shows all keys as compliant with the unpinned expectation (no mismatch) |
| T05 | `.status v::0` / `.status v::1` | any | output byte-for-byte identical to pre-task behavior (no `Lock:` content, no regression) |
| T06 | `.status v::3` | any | continues to exit 1 as out-of-range (no regression of existing boundary, `IT-11`) |

## Acceptance Criteria

- All 6 Test Matrix rows have a corresponding passing test
- `.status v::2` and `format::json` report lock-mechanism compliance for whichever keys are actually present
- `v::0`, `v::1`, `v::3` behavior is provably unchanged (existing `IT-3`, `IT-4`, `IT-11` still pass)
- `module/claude_version/tests/docs/cli/command/02_status.md` documents the new Factor and TC rows
- `verb/test` (full suite) passes with zero failures and zero warnings

## Validation

**Execution:** The procedure for walking this section is defined in `validation.rulebook.md`. The executor does NOT self-validate — an independent validator performs the walk after EXEC_COMPLETE transition (⚙️ → 🔎).

### Checklist

Desired answer for every question is YES.

**Lock visibility**
- [ ] C1 — Does `.status v::2` show a `Lock:` section reporting chmod mode and every present settings key?
- [ ] C2 — Does a chmod/setting mismatch get flagged distinctly from a compliant state?
- [ ] C3 — Does `format::json` include a `"lock"` object regardless of `v::`?

**No regression**
- [ ] C4 — Are `v::0` and `v::1` outputs byte-for-byte unchanged from before this task?
- [ ] C5 — Does `v::3` still exit 1 as out-of-range?

**Out of Scope confirmation**
- [ ] C6 — Does `.status` perform zero mutation/repair actions (read-only preserved)?

### Measurements

- [ ] M1 — new test count: `grep -c '#\[test\]' module/claude_version/tests/cli/read_status_test.rs` → increased by exactly 6 from this session's baseline

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

- [ ] AF1 — read-only proof: `grep -c 'chmod\|set_setting\|remove_setting\|set_env_var\|remove_env_var' module/claude_version/src/commands/status.rs` → 0 (status.rs must contain zero write/mutation calls)
- [ ] AF2 — doc sync: `tests/docs/cli/command/02_status.md` Test Matrix Summary total test count increased by exactly 6

## Related Documentation

- `module/claude_version/task/decisions.md` — Q-02 (Lock-State Visibility Surface), closed by this task
- `module/claude_version/tests/docs/cli/command/02_status.md` — Test Factor Analysis + Test Matrix, updated by this task
- `module/claude_version/docs/pattern/001_version_lock.md` — Mechanism Coverage table (source of the mechanism list this task reports on)
- `contract/claude_code/docs/settings/003_version_lock.md` — Version Lock Filesystem Operations (the layers this task reads and reports)
- `module/claude_version/docs/pitfall/001_version_lock_chmod.md` — documents the chmod-drift scenario this task makes visible
- `module/claude_version/docs/pitfall/002_symlink_retarget.md` — documents a related drift scenario this task makes visible
- `module/claude_version/src/commands/status.rs` — extended by this task
- `module/claude_version_core/src/version.rs` — new read-only chmod-mode helper added by this task

**Closes:** Q-02

## History

- **[2026-07-05]** `CREATED` — Extend `.status v::2` and JSON output with a `Lock:` section reporting actual-vs-expected compliance for every version-pinning mechanism key, closing the visibility gap between what the user asked to pin and whether the enforcement mechanisms are actually holding.
