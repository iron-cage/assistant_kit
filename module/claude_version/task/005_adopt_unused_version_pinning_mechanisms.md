# Task 005: Adopt Unused Version-Pinning Mechanisms in Lock Layer

## Execution State

- **Executor Type:** any
- **filed_by:** dev
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_core/assistant_kit/claude_version/module/claude_version_core
- **validated_by:** null
- **validation_date:** null

## Goal

Extend `lock_version()` in `claude_version_core/src/version.rs` so a pinned install also sets `autoUpdatesChannel` (`"stable"`), `minimumVersion` (the resolved pinned semver), and `env.DISABLE_UPDATES` (`"1"`) — the three self-service mechanisms the Mechanism Coverage table in `module/claude_version/docs/pattern/001_version_lock.md` marks ❌/⚠️ against the 8 official upstream mechanisms. **Motivated:** today `lock_version()` writes only `autoUpdates: false` and `env.DISABLE_AUTOUPDATER`, both of which stop only the *background* updater — a user or script that flips `autoUpdates` back to `true`, or runs `claude update` manually, is not stopped at all by the current lock; the 3 additional keys close those specific, documented bypass vectors. **Observable:** `.version.install` against a pinned target writes all 5 keys to `settings.json`, and `.config show` reports all 5 with `source: user`; unpinning (`target::"latest"`) symmetrically removes/resets all 5. **Scoped:** exactly these 3 additional self-service keys via the existing `set_setting()`/`set_env_var()` primitives already used by `lock_version()` — no new write path, no new dependency, no CLI-facing flag. **Testable:** `verb/test_only lock_version` plus new assertions in `claude_version_core/tests/version_test.rs` (T01-T05) and `claude_version_core/tests/config_resolve_test.rs` (T06) covering all 6 Test Matrix rows below — T06's new test function name MUST include the substring `lock_version` so this single filtered run selects it too, since `config_resolve_test.rs`'s existing 6 tests all follow an unrelated `at0N_002_*` naming convention with no `lock_version` substring.

## In Scope

- `module/claude_version_core/src/version.rs` — `lock_version()` (or a small helper it calls)
- Setting 3 new keys during pinned install: `autoUpdatesChannel="stable"`, `minimumVersion=<resolved pinned semver>`, `env.DISABLE_UPDATES="1"`
- `module/claude_version_core/src/config_catalog.rs` — register the 3 new keys in the static catalog consumed by `resolve_all()`, so `.config show` surfaces them with `source: user` (required for the Goal's `.config show` claim and Test Matrix T04 — without this registration, the 3 keys land in `settings.json` but stay invisible to `.config show`)
- `lock_version()`'s signature — add a `resolved : &str` parameter to carry the pinned semver into `minimumVersion` writes; its sole call site in `perform_install()` (same file, already in scope) is updated to pass through the `resolved` parameter `perform_install()` already receives — no CLI-crate change, contradicting the (now-corrected) Out of Scope claim below
- `module/claude_version_core/src/config_resolve.rs`'s `resolve()` Step 3 (User config check) — fix the flat top-level key match so dotted `env.*` catalog keys (`env.DISABLE_AUTOUPDATER`, `env.DISABLE_UPDATES`) resolve by looking inside the nested `"env"` sub-object instead of failing to match and falling through to `Absent`; required for Requirements bullet 4 / Test Matrix T04 and T06 — 2 of the 5 lock keys live inside the nested `env` object, not as flat top-level keys. Also fixes the pre-existing `env.DISABLE_AUTOUPDATER` catalog entry (registered since before this task, never previously reachable via `.config show`)
- Symmetric removal/reset of the same 3 keys on unpin (`target::"latest"`), mirroring the existing `autoUpdates`/`env.DISABLE_AUTOUPDATER` removal behavior already in `lock_version()`
- Updating `contract/claude_code/docs/settings/003_version_lock.md` (Version Lock Filesystem Operations table + Install Sequence) to describe the extended 5-key/6-layer lock — this task's own code change makes the current 3-layer description stale, so the update is delivered in the same task, not deferred
- Updating `module/claude_version/docs/pattern/001_version_lock.md`'s Mechanism Coverage table rows 1 (Channel selection), 2 (Soft update floor), 4 (Update suppression) to ✅ Used, for the same reason
- New/updated tests in `claude_version_core/tests/version_test.rs`

## Out of Scope

- `requiredMinimumVersion` / `requiredMaximumVersion` — managed-settings.json only; no self-service write path exists in this repo, and no concrete need for MDM-style org deployment has been identified (confirmed via `contract/claude_code/docs/pattern/001_version_pinning.md` § Solution item 3)
- `installMethod` — managed-settings.json only, explicitly "no self-service equivalent" per `contract/claude_code/docs/pattern/001_version_pinning.md` § Solution item 5 and § Applicability
- `manifest.json` / codesign integrity verification — a different concern (tamper detection, not version pinning); no concrete committed need
- Any new CLI-facing flag to control the 3 keys individually — they apply unconditionally as part of the existing pin/unpin flow, matching how `autoUpdates`/`DISABLE_AUTOUPDATER` are applied today with no equivalent flag
- Changes to `claude_version` (CLI crate) command wiring — `.version.install`/`.version.guard` already call `perform_install(resolved, is_latest)` with no change to that call's own arguments; this task is confined to `claude_version_core`. (`lock_version()`'s own signature does gain a `resolved` parameter — see In Scope — but this is an internal `claude_version_core` change, not CLI-crate wiring: `perform_install` already holds `resolved` and only threads it one level deeper to a function it already calls)

## Null Hypothesis

Do nothing — leave `lock_version()` at 2 of 8 official mechanisms, relying on `autoUpdates: false` + `env.DISABLE_AUTOUPDATER` alone.

**Disproof:** The Mechanism Coverage table (already committed in `module/claude_version/docs/pattern/001_version_lock.md`) enumerates 3 further self-service mechanisms that close concrete, documented bypass vectors: manual `claude update` (blocked only by `DISABLE_UPDATES`, not `DISABLE_AUTOUPDATER`), channel drift if `autoUpdates` is ever re-enabled (mitigated by `autoUpdatesChannel`), and floor-less downgrade (mitigated by `minimumVersion`). This is a committed, already-known gap — the user explicitly asked for "a comprehensive plan to use all [the mechanisms]," not speculative hardening. Disproof would require showing the 3 additional keys don't change effective pinning behavior; they do, per `param/119_disable_updates.md`, `param/121_auto_updates_channel.md`, and `param/122_minimum_version.md` (all read and confirmed this session).

## Requirements

- Pinned installs (`target::<specific version>`) MUST write exactly 5 keys: `autoUpdates=false`, `env.DISABLE_AUTOUPDATER="1"`, `env.DISABLE_UPDATES="1"`, `autoUpdatesChannel="stable"`, `minimumVersion=<resolved semver>`
- Unpinning (`target::"latest"`) MUST remove/reset the same 5 keys symmetrically (matching current removal behavior for the existing 2)
- Re-pinning to a different version while already pinned MUST update `minimumVersion` to the new resolved semver (never leave it stale at a prior floor)
- `.config show` MUST resolve all 5 lock keys with `source: user` after a pinned install — requires (a) registering the 3 new keys in `config_catalog.rs`'s static catalog, and (b) fixing `config_resolve.rs`'s `resolve()` Step 3 to match dotted `env.*` keys against the nested `env` sub-object (both `env.DISABLE_AUTOUPDATER` and `env.DISABLE_UPDATES` are unreachable via the current flat-key match alone)
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

1. Read `module/claude_version_core/src/version.rs` `lock_version()`/`perform_install()` in full (current 2-key behavior, current call site), `settings_io.rs` (`set_setting`, `set_env_var`, `remove_setting`, `remove_env_var`, `json_parse_flat_object` signatures), and `config_resolve.rs` (`resolve()` Step 3)
2. Write the 6 Test Matrix rows below as failing tests in `claude_version_core/tests/version_test.rs` and `config_resolve_test.rs`; name T06's test function with the substring `lock_version` (e.g. `at07_002_lock_version_nested_env_resolves`, extending the file's existing `at0N_002_*` prefix convention rather than replacing it) so step 8's single filtered run selects it — the file's existing 6 tests use that same prefix convention with no `lock_version` substring and would not otherwise be selected
3. Change `json_parse_flat_object` in `settings_io.rs` from private to `pub(crate)` so `config_resolve.rs` can reuse it to parse the nested `env` sub-object
4. Fix `config_resolve.rs`'s `resolve()` Step 3: when `key` has an `env.` prefix, look up the remainder inside the nested `env` sub-object (parsed via `json_parse_flat_object`) instead of flat-matching the whole dotted key against top-level pairs
5. Add a `resolved : &str` parameter to `lock_version()`; update its sole call site in `perform_install()` to pass through the `resolved` parameter `perform_install` already receives
6. Extend `lock_version()` to additionally write/remove `autoUpdatesChannel`, `minimumVersion=resolved`, `env.DISABLE_UPDATES` symmetrically with the existing 2 keys
7. Register the 3 new keys in `config_catalog.rs`'s static catalog (mirroring the existing `autoUpdates`/`env.DISABLE_AUTOUPDATER` entries) so `resolve_all()` surfaces them for `.config show`
8. Run `verb/test_only lock_version` until all 6 rows pass
9. Update `contract/claude_code/docs/settings/003_version_lock.md` — extend the Version Lock Filesystem Operations table to 5 keys and the Install Sequence step 4 to name all 5
10. Update `module/claude_version/docs/pattern/001_version_lock.md` Mechanism Coverage table rows 1, 2, 4 to ✅ Used with a short note on how each is applied
11. Run full `verb/test` (all crates) and confirm zero failures, zero warnings

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `.version.install target::"2.1.78"` (pin) | fresh settings.json | `autoUpdatesChannel="stable"`, `minimumVersion="2.1.78"`, `env.DISABLE_UPDATES="1"` all written alongside existing `autoUpdates=false`/`DISABLE_AUTOUPDATER="1"` |
| T02 | `.version.install target::"latest"` (unpin) | settings.json with all 5 pin keys already set | all 5 keys removed/reset (`autoUpdates`→`true`, `DISABLE_AUTOUPDATER`/`DISABLE_UPDATES` removed, `autoUpdatesChannel`/`minimumVersion` removed) |
| T03 | `.version.install target::"2.1.90"` while already pinned to `"2.1.78"` | settings.json with prior pin keys | `minimumVersion` updates to `"2.1.90"` (not left stale at `"2.1.78"`) |
| T04 | `.config show` after a pinned install | settings.json with 5 lock keys | all 5 keys resolve with `source: user` |
| T05 | existing Layer 1/2 tests (`autoUpdates`, `DISABLE_AUTOUPDATER`) | — | continue passing unchanged (no regression) |
| T06 | `config_resolve::resolve("env.DISABLE_AUTOUPDATER", ...)` and `resolve("env.DISABLE_UPDATES", ...)` directly (unit-level, no CLI) | settings.json with `"env": {"DISABLE_AUTOUPDATER": "1", "DISABLE_UPDATES": "1"}` | both resolve to `Some("1")` with `source: Layer::User` (not `Absent`) — proves the nested-object lookup fix independent of the full install flow |

## Acceptance Criteria

- All 6 Test Matrix rows have a corresponding passing test
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
- [ ] C6 — Do both `env.DISABLE_AUTOUPDATER` and `env.DISABLE_UPDATES` resolve with `source: user` via `.config show`/`.config get` (not `Absent`)?

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

- `module/claude_version_core/src/config_catalog.rs` — extended by this task to register the 3 new keys for `.config show` visibility
- `module/claude_version_core/src/config_resolve.rs` — `resolve()` Step 3 fixed by this task to match dotted `env.*` catalog keys against the nested `env` sub-object
- `module/claude_version_core/src/settings_io.rs` — `json_parse_flat_object` visibility changed to `pub(crate)` by this task for reuse by `config_resolve.rs`
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

## Verification Findings

**Round 1** (2026-07-05, Full Round) — 3/4 PASS, ITERATE

- ❌ **MOST Goal Quality (FAIL, blocking):** Goal's Observable clause and Test Matrix T04 claimed `.config show` would resolve all 5 lock keys with `source: user`, but `config_catalog.rs`'s static registry (consumed by `resolve_all()`) had no entries for the 3 new keys, and no In Scope/Work Procedure step touched it — the claim was unachievable as originally scoped. **Fixed:** added a `config_catalog.rs` registration bullet to In Scope, a Requirements bullet, a new Work Procedure step 4, and a Related Documentation entry.
- ✅ **Scope Coherence:** PASS
- ✅ **Value/YAGNI:** PASS (non-blocking: "user explicitly asked" clause unverifiable from repo state alone; History's "6" count doesn't precisely reconcile with the 8-row Mechanism Coverage table — not fixed, cosmetic)
- ✅ **Implementation Readiness:** PASS (non-blocking: T05 alludes to an "existing Layer 1/2 test" that actually lives in a different crate — not fixed, cosmetic)

**Round 2** (2026-07-05, Delta Round) — 0/2 PASS, ITERATE

- ❌ **MOST Goal Quality (FAIL, blocking, re-verify):** The Round 1 fix (config_catalog.rs registration) was necessary but not sufficient — `config_resolve.rs`'s `resolve()` Step 3 flat-matches the whole dotted catalog key (`"env.DISABLE_AUTOUPDATER"`, `"env.DISABLE_UPDATES"`) against top-level settings.json keys, but both actually live inside the nested `"env"` sub-object under the bare field name — the flat match can never succeed, so both keys always resolve `Absent`, never `source: user`. This also affects the pre-existing `env.DISABLE_AUTOUPDATER` catalog entry (registered before this task, never previously reachable). **Fixed:** added `config_resolve.rs` + `settings_io.rs` visibility change to In Scope/Requirements/Work Procedure, new Test Matrix row T06, new Checklist item C6, new Related Documentation entries.
- ❌ **MOST Goal Quality (FAIL, blocking, fresh challenger — independent finding):** `lock_version(is_latest: bool)` has no parameter carrying the resolved semver, yet Requirements/T01/T03 require it to write/update `minimumVersion=<resolved semver>`, and Out of Scope explicitly (and incorrectly) claimed "no new parameters needed." **Fixed:** added a `resolved: &str` parameter to `lock_version()` to In Scope and Work Procedure; corrected the Out of Scope bullet to clarify the CLI-crate call-site signature is unchanged (`perform_install` already holds `resolved`) while `lock_version`'s own internal signature gains the parameter.

**Round 3** (2026-07-05, Delta Round) — 0/1 PASS, ITERATE

- ❌ **MOST Goal Quality (FAIL, blocking, re-verify):** The Goal's Testable clause and Work Procedure step 8 claimed a single `verb/test_only lock_version` run validates all 6 Test Matrix rows, but T06 lives in `config_resolve_test.rs`, whose existing 6 tests all follow an exclusive `at0N_002_*` naming convention with no relation to "lock_version" as a substring — `verb/test_only`'s filter is a nextest test-name substring match, so a `lock_version`-filtered run selects 0 tests in that file; T06 would never be confirmed by that command even though T01-T05 would be. Also reconfirmed all Round 1/Round 2 fixes remain sound (config_catalog.rs registration, nested-env-key `resolve()` Step 3 fix, `lock_version`'s `resolved` parameter, `pub(crate)` sequencing). Non-blocking: `module/claude_version/docs/pattern/001_version_lock.md`'s § Solution prose (distinct from the Mechanism Coverage table this task updates) will read stale after this task lands, since it still narrates only the pre-task layers — documentation-only, flagged for completeness, not fixed.
- ❌ **MOST Goal Quality (FAIL, blocking, fresh challenger — independent confirmation):** Independently found the identical `lock_version`-filter/T06 gap via the same reasoning, and independently reconfirmed all Round 1/Round 2 fixes remain sound. **Fixed (both findings):** required T06's test function name to include the substring `lock_version`, extending `config_resolve_test.rs`'s existing `at0N_002_*` prefix convention rather than replacing it, so the single `verb/test_only lock_version` run in step 8 now selects all 6 rows; updated the Goal's Testable clause and Work Procedure step 2 accordingly.

**Round 4** (2026-07-05, Delta Round) — 1/1 PASS, ITERATE (next: Full)

- ✅ **MOST Goal Quality (PASS, re-verify):** Confirmed the Round 3 T06-naming fix is mathematically sound (substring containment is transitive — nextest's fully-qualified test identifier necessarily contains "lock_version" if the function name does) and unambiguously worded, with a concrete conforming example. Independently re-verified all Round 1-3 fixes remain sound (config_catalog.rs registration, nested-env-key `resolve()` fix, `lock_version`'s `resolved` parameter, T06 naming mandate) via fresh source reads and empirical `cargo nextest list` runs. Non-blocking: Goal's Motivated clause slightly overstates `DISABLE_AUTOUPDATER`'s bypass framing for one disjunct (docs describe it as independent of `autoUpdates`); In Scope bullet 2's catalog-registration justification overstates necessity for 2 of 3 new keys (empirically tested against a standalone scratch build of the actual crate — only the nested `env.*` key genuinely needs registration; the 2 flat keys are auto-discovered by `resolve_all()`'s key-union step regardless of catalog membership).
- ✅ **MOST Goal Quality (PASS, fresh challenger — independent confirmation):** Independently confirmed all the same fixes sound via a full cross-check of Goal clauses against In Scope/Requirements/Test Matrix/Checklist. Considered and dismissed as non-issues: T01-T05 lacking T06's explicit substring-naming mandate (correct scoping — T06 alone conflicts with `config_resolve_test.rs`'s ambient `at0N_002_*` convention; T01-T05 live in `version_test.rs`, whose existing convention already naturally produces `lock_version`-containing names); Null Hypothesis's "2 of 8" phrasing (same class as Round 1's already-accepted cosmetic imprecision); C6/T04 checklist-wording asymmetry (a Checklist-completeness nuance, not a Goal-statement defect). No new blocking issue found.

Both dispatched agents PASS, but this was a Delta Round (only MOST Goal Quality re-dispatched) — per Round Type Selection, Scope Coherence, Value/YAGNI, and Implementation Readiness remain Passive Pass, unconfirmed since Round 1 despite three rounds of substantive Requirements/Work Procedure/Test Matrix changes. A confirming Full Round (Round 5) redispatching all 4 dimensions together is required before CONVERGED can be declared.

**Round 5** (2026-07-05, Full Round) — 4/4 PASS, CONVERGED

- ✅ **Scope Coherence (re-verify):** PASS. In Scope/Out of Scope confirmed still coherent and non-empty after 4 rounds of Requirements/Work Procedure changes; observable outcome remains meaningful.
- ✅ **MOST Goal Quality (re-verify):** PASS. Reconfirmed sound with zero new findings — 4th consecutive PASS.
- ✅ **Value/YAGNI (re-verify):** PASS. Null Hypothesis, concrete committed need, and scope boundaries hold after all prior-round changes.
- ✅ **Implementation Readiness (re-verify):** PASS. Verified from source, cold: Work Procedure's step 3→4 and 5→6 dependencies confirmed real via direct inspection of `settings_io.rs`/`config_resolve.rs`; all 6 Test Matrix rows confirmed achievable (T05 cross-crate reference reconfirmed non-blocking — the `verb/test` full-suite gate at step 11 is T05's actual backstop, not the filtered step 8 command); M1/AF2 measurement claims empirically verified against current source/docs. Non-blocking: T01-T05 have no explicit `lock_version`-substring naming mandate (only T06 does) — `version_test.rs`'s actual naming precedent is split between full-name and abbreviated forms, so a naming choice following the abbreviated precedent could under-select at step 8; does not block since step 11's unfiltered full-suite run is the actual completion gate, not step 8.
- ✅ **Fresh challenger (regression sweep, all 4 dimensions):** PASS on all 4 — independently confirmed no dimension regressed across Rounds 1-4's cumulative changes.

All 4 dimensions dispatched together in this Full Round, all PASS — same-round invariant satisfied per `governance/maav.rulebook.md § MAAV : Round Type Selection`. **CONVERGED.**

## Verification Record

- **Verified:** 2026-07-05
- **Cycle:** 5 MAAV rounds (Round 1 Full → Rounds 2-4 Delta → Round 5 Full, confirming) per `governance/maav.rulebook.md`
- **Final verdict:** CONVERGED — Scope Coherence ✅ · MOST Goal Quality ✅ · Value/YAGNI ✅ · Implementation Readiness ✅, all confirmed together in the same Full Round
- **Outstanding non-blocking notes (not fixed, carried forward for the implementer):**
  - T05 references pre-existing tests (`tc350`-`tc353`) that live in the `claude_version` CLI crate, not `claude_version_core` — cosmetic naming only; the actual regression backstop is Work Procedure step 11's full-suite `verb/test`
  - T01-T05 have no explicit `lock_version`-substring naming mandate (unlike T06) — `version_test.rs`'s existing naming precedent is split between full-name and abbreviated forms; if abbreviated, step 8's filtered run may under-select, though step 11 remains the actual completion gate
  - `module/claude_version/docs/pattern/001_version_lock.md`'s § Solution prose (distinct from the Mechanism Coverage table this task updates) will read stale after this task lands — documentation-only, flagged for completeness
