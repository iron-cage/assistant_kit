# Task 005: Adopt Unused Version-Pinning Mechanisms in Lock Layer

## Execution State

- **Executor Type:** any
- **filed_by:** dev
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ❓ (Unverified)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_core/assistant_kit/claude_version/module/claude_version_core
- **validated_by:** null
- **validation_date:** null

## Goal

Extend `lock_version()` in `claude_version_core/src/version.rs` so a pinned install also sets `autoUpdatesChannel` (`"stable"`), `minimumVersion` (the resolved pinned semver), and `env.DISABLE_UPDATES` (`"1"`) — the three self-service mechanisms the Mechanism Coverage table in `module/claude_version/docs/pattern/001_version_lock.md` marks ❌/⚠️ against the 8 official upstream mechanisms. **Motivated:** today `lock_version()` writes only `autoUpdates: false` and `env.DISABLE_AUTOUPDATER`, both of which stop only the *background* updater — a user or script that flips `autoUpdates` back to `true`, or runs `claude update` manually, is not stopped at all by the current lock; the 3 additional keys close those specific, documented bypass vectors. **Observable:** `.version.install` against a pinned target writes all 5 keys to `settings.json`, and `.config show` reports all 5 with `source: user`; unpinning (`target::"latest"`) symmetrically removes/resets all 5. **Scoped:** exactly these 3 additional self-service keys via the existing `set_setting()`/`set_env_var()` primitives already used by `lock_version()` — no new write path, no new dependency, no CLI-facing flag. **Testable:** `verb/test_only lock_version` plus new assertions in `claude_version_core/tests/version_test.rs` covering all 5 Test Matrix rows below.

## In Scope

- `module/claude_version_core/src/version.rs` — `lock_version()` (or a small helper it calls)
- Setting 3 new keys during pinned install: `autoUpdatesChannel="stable"`, `minimumVersion=<resolved pinned semver>`, `env.DISABLE_UPDATES="1"`
- Symmetric removal/reset of the same 3 keys on unpin (`target::"latest"`), mirroring the existing `autoUpdates`/`env.DISABLE_AUTOUPDATER` removal behavior already in `lock_version()`
- Updating `contract/claude_code/docs/settings/003_version_lock.md` (Version Lock Filesystem Operations table + Install Sequence) to describe the extended 5-key/6-layer lock — this task's own code change makes the current 3-layer description stale, so the update is delivered in the same task, not deferred
- Updating `module/claude_version/docs/pattern/001_version_lock.md`'s Mechanism Coverage table rows 1 (Channel selection), 2 (Soft update floor), 4 (Update suppression) to ✅ Used, for the same reason
- New/updated tests in `claude_version_core/tests/version_test.rs`

## Out of Scope

- `requiredMinimumVersion` / `requiredMaximumVersion` — managed-settings.json only; no self-service write path exists in this repo, and no concrete need for MDM-style org deployment has been identified (confirmed via `contract/claude_code/docs/pattern/001_version_pinning.md` § Solution item 3)
- `installMethod` — managed-settings.json only, explicitly "no self-service equivalent" per `contract/claude_code/docs/pattern/001_version_pinning.md` § Solution item 5 and § Applicability
- `manifest.json` / codesign integrity verification — a different concern (tamper detection, not version pinning); no concrete committed need
- Any new CLI-facing flag to control the 3 keys individually — they apply unconditionally as part of the existing pin/unpin flow, matching how `autoUpdates`/`DISABLE_AUTOUPDATER` are applied today with no equivalent flag
- Changes to `claude_version` (CLI crate) command wiring — `.version.install`/`.version.guard` already call `perform_install()`/`lock_version()` with no new parameters needed; this task is confined to `claude_version_core`

## Null Hypothesis

Do nothing — leave `lock_version()` at 2 of 8 official mechanisms, relying on `autoUpdates: false` + `env.DISABLE_AUTOUPDATER` alone.

**Disproof:** The Mechanism Coverage table (already committed in `module/claude_version/docs/pattern/001_version_lock.md`) enumerates 3 further self-service mechanisms that close concrete, documented bypass vectors: manual `claude update` (blocked only by `DISABLE_UPDATES`, not `DISABLE_AUTOUPDATER`), channel drift if `autoUpdates` is ever re-enabled (mitigated by `autoUpdatesChannel`), and floor-less downgrade (mitigated by `minimumVersion`). This is a committed, already-known gap — the user explicitly asked for "a comprehensive plan to use all [the mechanisms]," not speculative hardening. Disproof would require showing the 3 additional keys don't change effective pinning behavior; they do, per `param/119_disable_updates.md`, `param/121_auto_updates_channel.md`, and `param/122_minimum_version.md` (all read and confirmed this session).

## Requirements

- Pinned installs (`target::<specific version>`) MUST write exactly 5 keys: `autoUpdates=false`, `env.DISABLE_AUTOUPDATER="1"`, `env.DISABLE_UPDATES="1"`, `autoUpdatesChannel="stable"`, `minimumVersion=<resolved semver>`
- Unpinning (`target::"latest"`) MUST remove/reset the same 5 keys symmetrically (matching current removal behavior for the existing 2)
- Re-pinning to a different version while already pinned MUST update `minimumVersion` to the new resolved semver (never leave it stale at a prior floor)
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

### Work Procedure

1. Read `module/claude_version_core/src/version.rs` `lock_version()` in full (current 2-key behavior) and `settings_io.rs` (`set_setting`, `set_env_var`, `remove_setting`, `remove_env_var` signatures)
2. Write the 5 Test Matrix rows below as failing tests in `claude_version_core/tests/version_test.rs`
3. Extend `lock_version()` to additionally write/remove `autoUpdatesChannel`, `minimumVersion`, `env.DISABLE_UPDATES` symmetrically with the existing 2 keys
4. Run `verb/test_only lock_version` until all 5 rows pass
5. Update `contract/claude_code/docs/settings/003_version_lock.md` — extend the Version Lock Filesystem Operations table to 5 keys and the Install Sequence step 4 to name all 5
6. Update `module/claude_version/docs/pattern/001_version_lock.md` Mechanism Coverage table rows 1, 2, 4 to ✅ Used with a short note on how each is applied
7. Run full `verb/test` (all crates) and confirm zero failures, zero warnings

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `.version.install target::"2.1.78"` (pin) | fresh settings.json | `autoUpdatesChannel="stable"`, `minimumVersion="2.1.78"`, `env.DISABLE_UPDATES="1"` all written alongside existing `autoUpdates=false`/`DISABLE_AUTOUPDATER="1"` |
| T02 | `.version.install target::"latest"` (unpin) | settings.json with all 5 pin keys already set | all 5 keys removed/reset (`autoUpdates`→`true`, `DISABLE_AUTOUPDATER`/`DISABLE_UPDATES` removed, `autoUpdatesChannel`/`minimumVersion` removed) |
| T03 | `.version.install target::"2.1.90"` while already pinned to `"2.1.78"` | settings.json with prior pin keys | `minimumVersion` updates to `"2.1.90"` (not left stale at `"2.1.78"`) |
| T04 | `.config show` after a pinned install | settings.json with 5 lock keys | all 5 keys resolve with `source: user` |
| T05 | existing Layer 1/2 tests (`autoUpdates`, `DISABLE_AUTOUPDATER`) | — | continue passing unchanged (no regression) |

## Acceptance Criteria

- All 5 Test Matrix rows have a corresponding passing test
- `contract/claude_code/docs/settings/003_version_lock.md` describes the 5-key lock (not 3)
- `module/claude_version/docs/pattern/001_version_lock.md` Mechanism Coverage table rows 1, 2, 4 read ✅ Used
- `verb/test` (full suite) passes with zero failures and zero warnings

## Validation

**Execution:** The procedure for walking this section is defined in `validation.rulebook.md`. The executor does NOT self-validate — an independent validator performs the walk after EXEC_COMPLETE transition (⚙️ → 🔎).

### Checklist

Desired answer for every question is YES.

**Lock-key application**
- [ ] C1 — Does a pinned install write all 5 keys (2 existing + 3 new) to `settings.json`?
- [ ] C2 — Does an unpin remove/reset all 5 keys symmetrically?
- [ ] C3 — Does re-pinning to a new version update `minimumVersion` rather than leaving it stale?

**Out of Scope confirmation**
- [ ] C4 — Is `requiredMinimumVersion`/`requiredMaximumVersion` absent from any write path added by this task?
- [ ] C5 — Is `installMethod` absent from any write path added by this task?

### Measurements

- [ ] M1 — key count: `grep -c 'set_setting\|set_env_var' module/claude_version_core/src/version.rs` in `lock_version()` → 5 write call sites (was: 2)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

- [ ] AF1 — settings content: after `.version.install target::"2.1.78"`, `cat ~/.claude/settings.json` → contains literal strings `"autoUpdatesChannel"`, `"minimumVersion"`, `"DISABLE_UPDATES"`
- [ ] AF2 — doc sync: `grep -c '✅ Used' module/claude_version/docs/pattern/001_version_lock.md` → increased by exactly 3 from this session's baseline

## Related Documentation

- `module/claude_version/docs/pattern/001_version_lock.md` — Mechanism Coverage table (rows 1, 2, 4 updated by this task)
- `contract/claude_code/docs/settings/003_version_lock.md` — Version Lock Filesystem Operations + Install Sequence (updated by this task)
- `contract/claude_code/docs/settings/001_global_settings.md` — atomic write protocol, type inference used by `set_setting()`
- `contract/claude_code/docs/settings/readme.md` — Settings Config Parameter Table (scope column for all 5 keys)
- `contract/claude_code/docs/param/121_auto_updates_channel.md` — `autoUpdatesChannel` semantics
- `contract/claude_code/docs/param/122_minimum_version.md` — `minimumVersion` semantics
- `contract/claude_code/docs/param/119_disable_updates.md` — `DISABLE_UPDATES` semantics
- `contract/claude_code/docs/param/099_disable_autoupdater.md` — existing `DISABLE_AUTOUPDATER` (for contrast)
- `contract/claude_code/docs/pattern/001_version_pinning.md` — official mechanism landscape; § Applicability documents why `requiredMinimumVersion`/`requiredMaximumVersion`/`installMethod` are Out of Scope (managed-settings-only)

**Closes:** null

## History

- **[2026-07-05]** `CREATED` — Adopt `autoUpdatesChannel`, `minimumVersion`, `env.DISABLE_UPDATES` into `lock_version()`, closing 3 of the 6 previously-unused official version-pinning mechanisms identified in the Mechanism Coverage table.
