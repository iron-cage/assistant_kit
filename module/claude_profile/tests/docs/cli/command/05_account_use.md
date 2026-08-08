# Test: `.account.use`

Integration test planning for the `.account.use` command. See [command/namespace.md](../../../../docs/cli/command/001_account.md#command-5-accountuse) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Use overwrites `~/.claude/.credentials.json` with named account | Basic Invocation |
| IT-2 | Use updates per-machine active marker to new name | Marker Update |
| IT-3 | Use with nonexistent account exits 2 with "not found" message | Error Handling |
| IT-4 | Use with non-email name exits 1 | Validation |
| IT-5 | `dry::1` prints action without modifying credentials | Dry Run |
| IT-6 | Credential file content matches source account after use | Data Integrity |
| IT-7 | Other accounts in store are not modified by use | Isolation |
| IT-8 | Use with already-active account succeeds (idempotent) | Idempotency |
| IT-9 | Atomic write: no partial file on simulated crash | Atomicity |
| IT-10 | Missing `name::` parameter exits 1 (required) | Required Param |
| IT-11 | `.credentials.status` shows new account email after use | Email Consistency |
| IT-12 | Use with path-unsafe chars in email local part exits 1 | Validation |
| IT-13 | Positional bare arg `alice@home.com` (no `name::`) switches account | Positional Syntax |
| IT-14 | Prefix `car` resolves to `carol@example.com` and switches account | Prefix Resolution |
| IT-15 | Ambiguous prefix matches two accounts → exit 1 | Prefix Resolution / Error |
| IT-16 | Exact local-part wins over longer ambiguous prefix | Prefix Resolution |
| IT-17 | `touch::1` live-token switch — exits 0, `switched` in stdout; subprocess dispatch not observed | Touch Subprocess |
| IT-18 | `touch::0` with idle account — pure rotation, no subprocess | Touch Subprocess |
| IT-19 | Same test as IT-17; "no subprocess" premise superseded by BUG-285 (now idempotent dispatch) | Touch Subprocess |
| IT-20 | `touch::1` with fetch failure — switch completes, exits 0 | Touch Subprocess |
| IT-21 | `imodel::bad` on `.account.use` exits 1 with valid values in stderr | Validation |
| IT-22 | `effort::bad` on `.account.use` exits 1 with valid values in stderr | Validation |
| IT-23 | `touch::`, `refresh::`, `imodel::`, `effort::`, `trace::` appear in `.account.use --help` | Help Output |
| IT-28 | `refresh::bad` exits 1 naming valid values `0`, `1`, `false`, `true` | Validation |
| IT-24 | `trace::1 touch::1` — 3 trace lines unconditional; 3 more only when live fetch succeeds | Trace Output |
| IT-25 | `trace::1 touch::0` — no timestamped `account.use` diagnostic lines emitted | Trace Suppression |
| IT-26 | `trace::bad` exits 1 naming valid values `0`, `1`, `false`, `true` | Validation |
| IT-27 | `oauthAccount.organizationName` in `~/.claude.json` reflects switched-to account (BUG-219 guard) | Org Identity |
| IT-29 | `oauthAccount.emailAddress` in `~/.claude.json` patched unconditionally even when `{name}.json` absent (BUG-254 guard) | oauthAccount Email |
| IT-30 | Positional bare arg after `key::value` param (reversed order) | Positional Syntax |

### Test Coverage Summary

- Basic Invocation: 1 test
- Marker Update: 1 test
- Error Handling: 1 test
- Validation: 6 tests
- Dry Run: 1 test
- Data Integrity: 1 test
- Isolation: 1 test
- Idempotency: 1 test
- Atomicity: 1 test
- Required Param: 1 test
- Email Consistency: 1 test
- Positional Syntax: 2 tests
- Prefix Resolution: 3 tests
- Touch Subprocess: 4 tests
- Help Output: 1 test
- Trace Output: 2 tests
- Trace Suppression: 1 test
- Org Identity: 1 test
- oauthAccount Email: 1 test

**Total:** 30 integration tests

---

### IT-1: Use overwrites credentials with named account

- **Given:** Two accounts saved in `~/.persistent/claude/credential/`: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Per-machine active marker (`_active_{hostname}_{user}`) set to `work`. `~/.claude/.credentials.json` contains `work` credentials.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'` on stdout, exit 0.; credentials file replaced with `personal` account content
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-2: Use updates per-machine active marker to new name

- **Given:** Two accounts saved: `work@acme.com` and `personal@home.com`. Per-machine active marker (`_active_{hostname}_{user}`) contains `work@acme.com`.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'` on stdout, exit 0.; per-machine active marker reads `personal@home.com`
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-3: Use with nonexistent account exits 2

- **Given:** Account store contains only `work@acme.com.credentials.json`. No `ghost@example.com.credentials.json` exists.
- **When:** `clp .account.use name::ghost@example.com`
- **Then:** Error message on stderr containing "not found", exit 2.; stderr contains "not found"; no state mutation
- **Exit:** 2
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-4: Use with non-email name exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`. Per-machine active marker is `work@acme.com`.
- **When:** `clp .account.use name::notanemail`
- **Then:** Error message on stderr indicating the name must be a valid email address, exit 1.; no state mutation
- **Exit:** 1
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-5: Dry run prints action without modifying credentials

- **Given:** Two accounts saved: `work@acme.com` (active) and `personal@home.com`. Record SHA-256 of `~/.claude/.credentials.json` and the per-machine active marker before command.
- **When:** `clp .account.use name::personal@home.com dry::1`
- **Then:** `[dry-run] would switch to 'personal@home.com'` on stdout, exit 0.; stdout contains dry-run message; no files modified
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-6: Credential file content matches source account after use

- **Given:** Account `personal@home.com` saved with known credential content containing specific `expiresAt`, `oauthAccessToken`, and `claudeAiSubscriptionType` values.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; credentials file is byte-identical to source account file
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-7: Other accounts in store not modified by use

- **Given:** Three accounts saved: `work@acme.com`, `personal@home.com`, `backup@archive.com`. Record SHA-256 of all three `.credentials.json` files in `~/.persistent/claude/credential/`.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; all non-target account files unchanged; source account file unchanged
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-8: Use with already-active account succeeds

- **Given:** Account `work@acme.com` saved and active. Per-machine active marker contains `work@acme.com`. `~/.claude/.credentials.json` matches `work@acme.com` credentials.
- **When:** `clp .account.use name::work@acme.com`
- **Then:** `switched to 'work@acme.com'`, exit 0.; state unchanged; no errors
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-9: Atomic write produces no partial file on simulated crash

- **Given:** Account `personal@home.com` saved. Set up filesystem observation to detect temporary files. Optionally, use a signal or filesystem constraint to interrupt mid-write.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; no `.json.tmp` residue; credentials file is always complete
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-10: Missing `name::` parameter exits 1

- **Given:** Account store contains `work@acme.com` account. No special state needed.
- **When:** `clp .account.use`
- **Then:** Error message on stderr indicating missing required parameter `name::`, exit 1.; no state mutation; error message references missing parameter
- **Exit:** 1
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse)

---

### IT-11: `.credentials.status` shows new account email after use

- **Given:** Two accounts saved via `.account.save` in order: first `work@acme.com` (with `emailAddress: "work@acme.com"` in its `~/.claude.json` snapshot), then `personal@home.com` (with `emailAddress: "personal@home.com"` in its snapshot). After both saves, `personal@home.com` is the active account and `~/.claude.json` contains `"emailAddress": "personal@home.com"`.
- **When:** `clp .account.use name::work@acme.com` then `clp .credentials.status`
- **Then:** `.credentials.status` output contains `Email: work@acme.com` (not `personal@home.com`). Exit 0.; `~/.claude.json` restored from `work@acme.com`'s snapshot; `credentials.status Email:` field reflects the switched-to account
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse), [004_account_use.md AC-05](../../../../docs/feature/004_account_use.md)

---

### IT-12: Use with path-unsafe chars in email local part exits 1

> Test asserts exit code only — does not read stderr content or check filesystem state. Contrast with the sibling `.account.save` coverage (`as17_save_slash_in_email_local_part_exits_1`), which checks both the stderr message ("path-unsafe characters") and that the credential store is not created.

- **Given:** Any account store state (the name is rejected before any store lookup).
- **When:** `clp .account.use name::a/b@c.com`
- **Then:** Exit 1.
- **Exit:** 1
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command-5-accountuse), [004_account_use.md AC-06](../../../../docs/feature/004_account_use.md), [aw11 in account_mutations_test_b.rs]

---

### IT-13: Positional bare arg switches account

- **Given:** Two accounts saved: `work@acme.com` (active) and `personal@home.com`.
- **When:** `clp .account.use personal@home.com` (no `name::` prefix)
- **Then:** Exits 0; `switched to 'personal@home.com'` on stdout; outcome identical to `clp .account.use name::personal@home.com`.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-01](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-14: Prefix resolves to single account

- **Given:** Two accounts saved: `carol@example.com` and `amy@example.com`. Per-machine active marker = `amy@example.com`.
- **When:** `clp .account.use car` (prefix form, no `@`)
- **Then:** Exits 0; `switched to 'carol@example.com'` on stdout; credentials overwritten with `carol@example.com` content.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-05](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-15: Ambiguous prefix exits 1

- **Given:** Two accounts saved: `alice@example.com` and `amy@example.com`.
- **When:** `clp .account.use a` (prefix matches both accounts)
- **Then:** Exits 1; stderr contains "ambiguous" and lists both matching account names.
- **Exit:** 1
- **Source:** [015_name_shortcut_syntax.md AC-06](../../../../docs/feature/015_name_shortcut_syntax.md)
- **Source fn:** `aw15_use_prefix_ambiguous_exits_1`

---

### IT-16: Exact local-part wins over longer ambiguous prefix

- **Given:** Three accounts saved: `i1@wbox.pro`, `i11@wbox.pro`, `i12@wbox.pro`. `i1@wbox.pro` is active.
- **When:** `clp .account.use i1`
- **Then:** Exits 0; `switched to 'i1@wbox.pro'` on stdout; active marker set to `i1@wbox.pro` (exact local-part match wins — no ambiguous-prefix error).
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-11](../../../../docs/feature/015_name_shortcut_syntax.md)
- **Source fn:** `aw16_exact_local_part_wins_over_ambiguous_prefix`

---

### IT-17: `touch::1` live-token switch — exits 0, `switched` in stdout (subprocess dispatch not observed)

> `aw27_lim_it_touch_with_live_token` (shared with IT-19) is a live/uncontrolled test — it does not force the target account into an idle 5h window; the account's real quota state at test time depends on the live token's history. Its own doc comment says either `pre_switch_touch_ctx` outcome (`Some`/idle or `None`/active-or-fetch-fail) "must exit 0," and treats the subprocess as fire-and-forget ("its success or failure does not affect the command exit code"). Assertions check only `assert_exit(&out, 0)` and that stdout contains `"switched"` — never whether a subprocess actually dispatched.

- **Given:** A live OAuth token account is used; `aw27` does not construct a specifically-idle fixture — the target account's actual 5h-window state at test time is whatever the live token's account happens to have.
- **When:** `clp .account.use name::target@example.com` (default `touch::1`), using a live token.
- **Then:** Exits 0; stdout contains `"switched"`. Whether a subprocess was actually dispatched is not observed by this test.
- **Exit:** 0
- **Live:** yes (requires a live OAuth token; `lim_it`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-01](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw27_lim_it_touch_with_live_token`

---

### IT-18: `touch::0` with idle account — pure rotation, no subprocess

- **Given:** One account `alice@home.com` saved with valid token and idle 5h window (`five_hour.resets_at` is absent). Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com touch::0`
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; no subprocess spawned.
- **Exit:** 0
- **Source:** [feature/027_account_use_post_switch_touch.md AC-02](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw22_touch_disabled_switch_succeeds`

---

### IT-19: Already-active account also dispatches subprocess (idempotently) — not skipped

> Shares `aw27_lim_it_touch_with_live_token` with IT-17 — same uncontrolled fixture, no idle/active distinction (see IT-17's note). The original "no subprocess spawned" premise is also stale: Fix(BUG-285) removed the `AlreadyActive` skip path from `PreSwitchOutcome` (AC-03) — "the idle check that previously skipped the subprocess for already-active accounts used server-side `resets_at` as a local subprocess identity oracle (category error)." The subprocess now dispatches idempotently (exits immediately) even when the account is already active, rather than being skipped.

- **Given:** A live OAuth token account is used; same shared, uncontrolled fixture as IT-17.
- **When:** `clp .account.use name::target@example.com` (default `touch::1`), using a live token.
- **Then:** Exits 0; stdout contains `"switched"`. Per AC-03 the subprocess now dispatches idempotently even when the account is already active (no longer skipped) — but this specific outcome is not asserted by `aw27` either, which checks only the shared exit0/message behavior.
- **Exit:** 0
- **Live:** yes (requires a live OAuth token; `lim_it`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-03](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw27_lim_it_touch_with_live_token`

---

### IT-20: `touch::1` with fetch failure — switch completes, exits 0

- **Given:** Account `alice@home.com` saved with an invalid/expired `accessToken` (quota fetch will fail with auth error). Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; touch skipped silently due to fetch failure. No error message surfaces.
- **Exit:** 0
- **Source:** [feature/027_account_use_post_switch_touch.md AC-04](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw23_touch_skipped_no_access_token`

---

### IT-21: `imodel::bad` exits 1 with valid values in stderr

> Test asserts only 4 of the 5 valid values — `auto`, `sonnet`, `opus`, `keep`. It does not check for `haiku`, even though `imodel::`'s validator (`src/usage/types.rs`) formats all five into the error message: `"imodel:: must be one of: auto, sonnet, opus, keep, haiku"`.

- **Given:** Any account store state (empty is fine).
- **When:** `clp .account.use name::alice@home.com imodel::bad`
- **Then:** Exits 1; stderr contains `auto`, `sonnet`, `opus`, `keep` (test does not assert `haiku` is present).
- **Exit:** 1
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw24_imodel_bad_value_exits_1`

---

### IT-22: `effort::bad` exits 1 with valid values in stderr

> Test asserts only 3 of the 5 valid values — `auto`, `high`, `max`. It does not check for `low` or `normal`, even though `effort::`'s validator (`src/usage/types.rs`) formats all five into the error message: `"effort:: must be one of: auto, high, max, low, normal"`.

- **Given:** Any account store state (empty is fine).
- **When:** `clp .account.use name::alice@home.com effort::bad`
- **Then:** Exits 1; stderr contains `auto`, `high`, `max` (test does not assert `low` or `normal` are present).
- **Exit:** 1
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw25_effort_bad_value_exits_1`

---

### IT-23: `touch::`, `refresh::`, `imodel::`, `effort::`, `trace::` appear in `.account.use --help`

- **Given:** Any state.
- **When:** `clp .account.use --help` (or `.account.use help::1`)
- **Then:** Exits 0; help output contains `touch::` with default `1`, `refresh::` with default `1`, `imodel::` with default `auto`, `effort::` with default `auto`, and `trace::` with default `0`.
- **Exit:** 0
- **Source:** [feature/027_account_use_post_switch_touch.md AC-09, AC-16](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw26_help_shows_touch_imodel_effort`

---

### IT-24: `trace::1 touch::1` — subprocess dispatch trace lines (partially conditional on live quota fetch)

> `aw28_lim_it_trace_idle_account_all_lines` unconditionally asserts only: the `· account.use` trace prefix, `reading`/`reading: OK`, and the absence of `idle check:` (Fix(BUG-285) removed that line). The `quota fetch: OK`, `subprocess: scheduled (idle check removed)`, `model:`, and `subprocess: spawned` lines are checked only inside `if err.contains("quota fetch: OK")` — when the live fetch fails, none of these four are asserted at all (a silent `eprintln!` skip, not a test failure). No line-order assertion exists anywhere in the test (every check is a `contains()` substring test, not a positional one); `effort:` is never checked at all.

- **Given:** Account `alice@home.com` saved with a live token; whether the live quota fetch succeeds is not controlled by the test.
- **When:** `clp .account.use name::alice@home.com trace::1`
- **Then:** Exits 0; stdout contains `switched`. Stderr always contains the `· account.use` trace prefix, `reading`, and `reading: OK`, and never contains `idle check:`. Only when the live fetch succeeds (stderr contains `quota fetch: OK`) does stderr additionally contain `subprocess: scheduled (idle check removed)`, `model:`, and `subprocess: spawned` — none of these four are asserted, in any order, when the fetch fails.
- **Exit:** 0
- **Live:** yes (requires valid token; fetch-success trace lines depend on the live quota fetch outcome)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10–AC-14](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw28_lim_it_trace_idle_account_all_lines`

---

### IT-25: `trace::1 touch::0` — no timestamped `account.use` diagnostic lines emitted

- **Given:** Account `alice@home.com` saved.
- **When:** `clp .account.use name::alice@home.com touch::0 trace::1`
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout. Stderr contains no timestamped `account.use` diagnostic lines.
- **Exit:** 0
- **Source:** [feature/027_account_use_post_switch_touch.md AC-15](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw31_trace_touch_disabled_no_trace_lines`

---

### IT-26: `trace::bad` exits 1 naming valid values

- **Given:** Any account store state (empty is fine — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com trace::bad`
- **Then:** Exits 1; stderr contains the four valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source:** [feature/027_account_use_post_switch_touch.md AC-16](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw32_trace_bad_value_exits_1`

---

### IT-27: `oauthAccount.organizationName` reflects switched-to account (BUG-219 guard)

> **Core-library-level test, not a CLI integration test** — `mre_bug_219_switch_account_stale_org_name` calls `account::switch_account("i6@test.com", &store, &paths)` directly (in `claude_profile_core`, not the `clp` binary). No subprocess is spawned, so there is no CLI exit code or stdout to observe; success is `.unwrap()` not panicking, and the result is verified by reading `~/.claude.json` back and parsing it with `parse_string_field`. The fixture also never saves an `i7@test.com` account in the credential store — it only pre-writes `~/.claude.json` directly (simulating i7 as the previously-active session) plus `i6@test.com.credentials.json` and `i6@test.com.json` directly via `std::fs::write`.

- **Given:** `~/.claude.json` pre-written directly with `oauthAccount = {emailAddress: "i7@test.com", organizationName: "i7 Org", organizationUuid: "uuid-i7"}` (simulating i7 as the previously-active session; no `i7@test.com` file exists in the credential store). `{store}/i6@test.com.credentials.json` and `{store}/i6@test.com.json` are written directly; `i6@test.com.json`'s `oauthAccount` subtree still has the stale `"i7 Org"` values, but its top-level `organization_name = "i6 Org"` / `organization_uuid = "uuid-i6"` are correct.
- **When:** `account::switch_account("i6@test.com", &store, &paths)` called directly (no CLI subprocess).
- **Then:** Call returns `Ok(())`. `~/.claude.json`, read back and parsed via `parse_string_field`, contains `oauthAccount.organizationName = "i6 Org"` and `organizationUuid = "uuid-i6"` (i6's top-level values, not the stale `"i7 Org"`/`"uuid-i7"` snapshot), and `oauthAccount.emailAddress = "i6@test.com"`.
- **Exit:** N/A (core-library call, not a CLI invocation)
- **Source:** [feature/004_account_use.md BUG-219](../../../../docs/feature/004_account_use.md)
- **Source fn:** `mre_bug_219_switch_account_stale_org_name` (in `claude_profile_core/tests/account_test.rs`)

---

### IT-28: `refresh::bad` exits 1 naming valid values

- **Given:** Any account store state (empty is fine — validation runs before any I/O).
- **When:** `clp .account.use name::alice@home.com refresh::bad`
- **Then:** Exits 1; stderr contains valid values `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source:** [feature/027_account_use_post_switch_touch.md AC-09](../../../../docs/feature/027_account_use_post_switch_touch.md), [params/019_refresh.md](../../../../docs/cli/param/019_refresh.md)
- **Source fn:** `aw34_refresh_bad_value_exits_1`

---

### IT-29: `oauthAccount.emailAddress` patched unconditionally when `{name}.json` absent (BUG-254 guard)

- **Given:** Account `bob@acme.com` has `bob@acme.com.credentials.json` but NO `bob@acme.com.json` metadata file. Current active is `alice@acme.com` with `~/.claude.json` containing `oauthAccount.emailAddress = "alice@acme.com"`.
- **When:** `clp .account.use name::bob@acme.com`
- **Then:** Exits 0; `switched to 'bob@acme.com'` on stdout. `~/.claude.json` contains `oauthAccount.emailAddress = "bob@acme.com"` — patched unconditionally even without metadata file. All other `oauthAccount` fields retain their previous values from alice's session. `_active_{hostname}_{user}` marker contains `bob@acme.com`.
- **Exit:** 0
- **Source:** [feature/004_account_use.md AC-09](../../../../docs/feature/004_account_use.md)
- **Source fn:** `mre_bug254_switch_account_patches_email_when_metadata_absent` (core), `aw12_switch_patches_email_when_metadata_absent` (FT)

---

### IT-30: Positional bare arg after `key::value` param (reversed order)

- **Given:** Two accounts saved: `work@acme.com` (active) and `personal@home.com`.
- **When:** `clp .account.use dry::1 personal@home.com` (key::value before bare positional name)
- **Then:** Exits 0; dry-run output shows intent for `personal@home.com`. Identical result to `clp .account.use personal@home.com dry::1`. Argument order does not affect positional rewrite.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-14](../../../../docs/feature/015_name_shortcut_syntax.md)
- **Source fn:** `aw36_positional_after_key_value`
