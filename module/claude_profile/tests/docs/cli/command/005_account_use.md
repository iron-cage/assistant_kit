# Test: `.account.use`

Integration test planning for the `.account.use` command. See [command/namespace.md](../../../../docs/cli/command/001_account.md#command--5-accountuse) for specification.

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
| IT-17 | `touch::1` with idle account — subprocess spawned after switch | Touch Subprocess |
| IT-18 | `touch::0` with idle account — pure rotation, no subprocess | Touch Subprocess |
| IT-19 | `touch::1` with already-active account — no subprocess spawned | Touch Subprocess |
| IT-20 | `touch::1` with fetch failure — switch completes, exits 0 | Touch Subprocess |
| IT-21 | `imodel::bad` on `.account.use` exits 1 with valid values in stderr | Validation |
| IT-22 | `effort::bad` on `.account.use` exits 1 with valid values in stderr | Validation |
| IT-23 | `touch::`, `refresh::`, `imodel::`, `effort::`, `trace::` appear in `.account.use --help` | Help Output |
| IT-28 | `refresh::bad` exits 1 naming valid values `0`, `1`, `false`, `true` | Validation |
| IT-24 | `trace::1 touch::1` idle account — 6 trace lines emitted to stderr in order | Trace Output |
| IT-25 | `trace::1 touch::0` — no `[trace] account.use` lines emitted | Trace Suppression |
| IT-26 | `trace::bad` exits 1 naming valid values `0`, `1`, `false`, `true` | Validation |
| IT-27 | `oauthAccount.organizationName` in `~/.claude.json` reflects switched-to account (BUG-219 guard) | Org Identity |

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
- Positional Syntax: 1 test
- Prefix Resolution: 3 tests
- Touch Subprocess: 4 tests
- Help Output: 1 test
- Trace Output: 2 tests
- Trace Suppression: 1 test
- Org Identity: 1 test

**Total:** 28 integration tests

---

### IT-1: Use overwrites credentials with named account

- **Given:** Two accounts saved in `~/.persistent/claude/credential/`: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Per-machine active marker (`_active_{hostname}_{user}`) set to `work`. `~/.claude/.credentials.json` contains `work` credentials.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'` on stdout, exit 0.; credentials file replaced with `personal` account content
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-2: Use updates per-machine active marker to new name

- **Given:** Two accounts saved: `work@acme.com` and `personal@home.com`. Per-machine active marker (`_active_{hostname}_{user}`) contains `work@acme.com`.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'` on stdout, exit 0.; per-machine active marker reads `personal@home.com`
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-3: Use with nonexistent account exits 2

- **Given:** Account store contains only `work@acme.com.credentials.json`. No `ghost@example.com.credentials.json` exists.
- **When:** `clp .account.use name::ghost@example.com`
- **Then:** Error message on stderr containing "not found", exit 2.; stderr contains "not found"; no state mutation
- **Exit:** 2
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-4: Use with non-email name exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`. Per-machine active marker is `work@acme.com`.
- **When:** `clp .account.use name::notanemail`
- **Then:** Error message on stderr indicating the name must be a valid email address, exit 1.; no state mutation
- **Exit:** 1
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-5: Dry run prints action without modifying credentials

- **Given:** Two accounts saved: `work@acme.com` (active) and `personal@home.com`. Record SHA-256 of `~/.claude/.credentials.json` and the per-machine active marker before command.
- **When:** `clp .account.use name::personal@home.com dry::1`
- **Then:** `[dry-run] would switch to 'personal@home.com'` on stdout, exit 0.; stdout contains dry-run message; no files modified
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-6: Credential file content matches source account after use

- **Given:** Account `personal@home.com` saved with known credential content containing specific `expiresAt`, `oauthAccessToken`, and `claudeAiSubscriptionType` values.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; credentials file is byte-identical to source account file
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-7: Other accounts in store not modified by use

- **Given:** Three accounts saved: `work@acme.com`, `personal@home.com`, `backup@archive.com`. Record SHA-256 of all three `.credentials.json` files in `~/.persistent/claude/credential/`.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; all non-target account files unchanged; source account file unchanged
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-8: Use with already-active account succeeds

- **Given:** Account `work@acme.com` saved and active. Per-machine active marker contains `work@acme.com`. `~/.claude/.credentials.json` matches `work@acme.com` credentials.
- **When:** `clp .account.use name::work@acme.com`
- **Then:** `switched to 'work@acme.com'`, exit 0.; state unchanged; no errors
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-9: Atomic write produces no partial file on simulated crash

- **Given:** Account `personal@home.com` saved. Set up filesystem observation to detect temporary files. Optionally, use a signal or filesystem constraint to interrupt mid-write.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; no `.json.tmp` residue; credentials file is always complete
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-10: Missing `name::` parameter exits 1

- **Given:** Account store contains `work@acme.com` account. No special state needed.
- **When:** `clp .account.use`
- **Then:** Error message on stderr indicating missing required parameter `name::`, exit 1.; no state mutation; error message references missing parameter
- **Exit:** 1
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse)

---

### IT-11: `.credentials.status` shows new account email after use

- **Given:** Two accounts saved via `.account.save` in order: first `work@acme.com` (with `emailAddress: "work@acme.com"` in its `~/.claude.json` snapshot), then `personal@home.com` (with `emailAddress: "personal@home.com"` in its snapshot). After both saves, `personal@home.com` is the active account and `~/.claude.json` contains `"emailAddress": "personal@home.com"`.
- **When:** `clp .account.use name::work@acme.com` then `clp .credentials.status`
- **Then:** `.credentials.status` output contains `Email: work@acme.com` (not `personal@home.com`). Exit 0.; `~/.claude.json` restored from `work@acme.com`'s snapshot; `credentials.status Email:` field reflects the switched-to account
- **Exit:** 0
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse), [004_account_use.md AC-05](../../../../docs/feature/004_account_use.md)

---

### IT-12: Use with path-unsafe chars in email local part exits 1

- **Given:** Any account store state (the name is rejected before any store lookup).
- **When:** `clp .account.use name::a/b@c.com`
- **Then:** Error message on stderr indicating the name contains path-unsafe characters, exit 1. No filesystem modification.
- **Exit:** 1
- **Source:** [command/001_account.md — .account.use](../../../../docs/cli/command/001_account.md#command--5-accountuse), [004_account_use.md AC-06](../../../../docs/feature/004_account_use.md), [aw11 in account_mutations_test.rs]

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

### IT-17: `touch::1` with idle account — subprocess spawned

- **Given:** One account `alice@home.com` saved with valid token and idle 5h window (`five_hour.resets_at` is absent). Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; an isolated subprocess is dispatched to start a 5h session for the idle account.
- **Exit:** 0
- **Live:** yes (requires valid token with idle 5h window to observe subprocess dispatch)
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

### IT-19: `touch::1` with already-active account — no subprocess spawned

- **Given:** Account `alice@home.com` saved with valid token and active 5h window (`five_hour.resets_at` is set). Per-machine active marker set to a different account.
- **When:** `clp .account.use name::alice@home.com` (default `touch::1`)
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; credentials rotated; no subprocess spawned (account already has active 5h session).
- **Exit:** 0
- **Live:** yes (requires valid token with active `five_hour.resets_at`)
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

- **Given:** Any account store state (empty is fine).
- **When:** `clp .account.use name::alice@home.com imodel::bad`
- **Then:** Exits 1; stderr contains all five valid values: `auto`, `sonnet`, `opus`, `haiku`, `keep`.
- **Exit:** 1
- **Source:** [feature/027_account_use_post_switch_touch.md AC-07](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw24_imodel_bad_value_exits_1`

---

### IT-22: `effort::bad` exits 1 with valid values in stderr

- **Given:** Any account store state (empty is fine).
- **When:** `clp .account.use name::alice@home.com effort::bad`
- **Then:** Exits 1; stderr contains all five valid values: `auto`, `low`, `normal`, `high`, `max`.
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

### IT-24: `trace::1 touch::1` idle account — 6 trace lines emitted to stderr

- **Given:** Account `alice@home.com` saved with valid token and idle 5h window (`five_hour.resets_at` absent).
- **When:** `clp .account.use name::alice@home.com trace::1`
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout. Stderr (in order) contains: `reading {path}`, `reading: OK`, `quota fetch: OK`, `idle check: resets_at=absent → idle`, `model: {model}  effort: {effort}`, `subprocess: spawned`. Prefix of every trace line is `[trace] account.use  alice@home.com`.
- **Exit:** 0
- **Live:** yes (requires valid token with idle `five_hour.resets_at = None`)
- **Source:** [feature/027_account_use_post_switch_touch.md AC-10–AC-14](../../../../docs/feature/027_account_use_post_switch_touch.md)
- **Source fn:** `aw28_lim_it_trace_idle_account_all_lines`

---

### IT-25: `trace::1 touch::0` — no `[trace] account.use` lines emitted

- **Given:** Account `alice@home.com` saved.
- **When:** `clp .account.use name::alice@home.com touch::0 trace::1`
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout. Stderr contains no `[trace] account.use` lines.
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

- **Given:** Account `i7@test.com` saved with `{credential_store}/i7@test.com.claude.json` containing `oauthAccount.organizationName = "i7 Org"` (and active, so `~/.claude.json` has this org). Account `i6@test.com` saved with `{credential_store}/i6@test.com.claude.json` also containing `organizationName = "i7 Org"` (snapshot was captured while i7 was active — stale cross-account contamination). Account `i6@test.com` has `{credential_store}/i6@test.com.roles.json` with `organization_name = "i6 Org"`.
- **When:** `clp .account.use name::i6@test.com`
- **Then:** Exits 0; `switched to 'i6@test.com'` on stdout. `~/.claude.json` contains `oauthAccount.organizationName = "i6 Org"` (reflecting i6's org from `roles.json`), NOT `"i7 Org"` (the stale snapshot value). `oauthAccount.emailAddress = "i6@test.com"`.
- **Exit:** 0
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
