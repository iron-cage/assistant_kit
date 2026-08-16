# Schema 007: Claude State — `~/.claude.json`

SC test cases for `docs/schema/007_claude_json.md`. Verifies write isolation (`clp`
writes to this file are limited to `.account.use`'s narrow identity patch — see SC-3),
absent fields and file produce graceful N/A output, and the `oauthAccount.emailAddress`
field is used as the default account name.

**Source:** [docs/schema/007_claude_json.md](../../../docs/schema/007_claude_json.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | Absent `~/.claude.json` — save succeeds, all metadata fields show N/A | Error Path | ✅ |
| SC-2 | `emailAddress` used as default account name when `name::` omitted | Field Semantics | ✅ |
| SC-3 | `clp` writes to `~/.claude.json` limited to `.account.use`'s identity patch | Write Isolation | ✅ |
| SC-4 | Absent `oauthAccount` subfields show N/A without error | Graceful Missing | ✅ |

---

### SC-1: Absent `~/.claude.json` — save succeeds and all metadata shows N/A

- **Given:** `~/.claude.json` does not exist on disk
- **When:** `.account.save` is invoked
- **Then:** The save completes successfully; `displayName`, `organizationRole`, `billingType` fields in `{name}.json` are either absent or N/A — no error is raised for a missing source file
- **Source fn:** `acc27_save_succeeds_without_claude_json` (cli/accounts_list_test_b.rs)
- **Source:** [docs/schema/007_claude_json.md §Graceful Missing-Field Handling](../../../docs/schema/007_claude_json.md)

---

### SC-2: `oauthAccount.emailAddress` used as default account name when `name::` omitted

- **Given:** `~/.claude.json` contains `oauthAccount.emailAddress = "alice@acme.com"` and `.account.save` is invoked without `name::` parameter
- **When:** `account_save_routine()` resolves the account name
- **Then:** The account is saved as `alice@acme.com` — the email address from `~/.claude.json` is used as the default name
- **Source fn:** `onboarding_ua2_name_inference_and_missing_source` in `tests/cli/user_story_test.rs`
- **Source:** [docs/schema/007_claude_json.md §Fields Read by clp](../../../docs/schema/007_claude_json.md)

---

### SC-3: `clp` writes to `~/.claude.json` are limited to `.account.use`'s identity patch

- **Given:** `~/.claude.json` contains `oauthAccount.emailAddress` set to a stale value (from
  a prior account's snapshot) alongside unrelated top-level data (e.g. `someGlobalKey`).
- **When:** `.account.use name::<target>` (`switch_account()`) runs.
- **Then:** `oauthAccount.emailAddress` is patched in-place to the target account name
  (BUG-217 fix) — a narrow, surgical write, not a wholesale rewrite; all other top-level
  keys (e.g. `someGlobalKey`) are preserved unchanged.
- **Note:** This test doc previously claimed `~/.claude.json` is never modified by ANY `clp`
  operation including `.account.use` — that claim is contradicted by this test's own
  assertions and has been narrowed here. No dedicated test currently verifies that
  `.account.save`, `.usage`, or `.model` leave `~/.claude.json` completely untouched;
  source inspection (`src/commands/account_ops.rs`, `src/commands/credentials.rs`) shows
  only `read_to_string` calls against `claude_json_file()` on those paths, never `fs::write`,
  but this is unverified by a dedicated runtime test. Flagged for review: whether the
  "read-only contract" framing (including in the source doc
  `docs/schema/007_claude_json.md`) should be revised to document `.account.use`'s patch as
  a deliberate, scoped exception.
- **Source fn:** `mre_bug_217_switch_account_enforces_emailaddress` in
  `tests/cli/account_relogin_test_b.rs`
- **Source:** [docs/schema/007_claude_json.md §Read-Only Contract](../../../docs/schema/007_claude_json.md)

---

### SC-4: Absent `oauthAccount` subfields show N/A without error

- **Given:** `~/.claude.json` exists with `oauthAccount.emailAddress` set but `taggedId` absent
- **When:** `.credentials.status uuid::1` reads `~/.claude.json`
- **Then:** The missing `taggedId` subfield produces `ID: N/A` in the output — no error, no panic for a partial `oauthAccount` object
- **Source fn:** `cred22_uuid_missing_tagged_id_shows_na` in `tests/cli/credentials_test.rs`
- **Source:** [docs/schema/007_claude_json.md §Graceful Missing-Field Handling](../../../docs/schema/007_claude_json.md)
