# Pitfall Tests: Account Identity Pitfalls

Test cases verifying that each guard documented in `docs/pitfall/004_account_identity_pitfalls.md`
is in place and prevents the described account identity failure mode.

**Source:** [docs/pitfall/004_account_identity_pitfalls.md](../../../docs/pitfall/004_account_identity_pitfalls.md)
**Case prefix:** `PP-` (Pitfall Protection)

### Pitfall Guard Index

| ID | Pitfall | Bug | Guard Verified By |
|----|---------|-----|-------------------|
| PP-1 | Active marker fixture names must not collide with real machine identity | BUG-308 | `test_write_quota_cache_preserves_touch_idle_false` (synthetic hostnames in all touch tests) |
| PP-2 | Account name from `oauthAccount.emailAddress`, not `_active` marker | BUG-212 | `test_apply_refresh_lifecycle_active_marker_unchanged` (FT-13+) and account save tests |

---

### PP-1: Active marker fixture names use synthetic hostnames that cannot collide

- **Given:** A test creates an `_active_{host}_{user}` marker as a fixture representing a
  "different machine's" marker.
- **When:** `active_marker_filename()` is called on the real test machine.
- **Then:** The synthetic marker name (`_active_testhost1_tst1`, `_active_testhost2_tst2`, etc.)
  does NOT match the machine's own marker — `assert_ne!` guards confirm non-collision.
  Fix BUG-308: tests that hardcoded `_active_w003_user1` failed on machines where
  `hostname=w003` and `user=user1`.
- **Rule:** Never hardcode real-looking hostnames in test fixtures for `_active_*` markers.
  Always use synthetic names that cannot match any realistic machine identity.
- **Source fn:** All active-marker tests in `tests/usage/touch_tests.rs`,
  `tests/usage/refresh_tests_a.rs`, and `tests/cli/cli_runner.rs` use synthetic names
  (`testhost1`/`tst1`); `tests/cli/account_owner_param_test.rs` fixture validation
- **Source:** [pitfall/004_account_identity_pitfalls.md §P1](../../../docs/pitfall/004_account_identity_pitfalls.md)

---

### PP-2: Account name from `oauthAccount.emailAddress`, not `_active` marker

- **Given:** Two accounts are saved — `alice@acme.com` (active marker) and
  `work@acme.com` (current `oauthAccount.emailAddress` in `~/.claude.json`).
- **When:** `.account.save` runs WITHOUT an explicit `name::` parameter.
- **Then:** The account name is inferred from `oauthAccount.emailAddress` (`work@acme.com`),
  NOT from the `_active_{host}_{user}` marker content (`alice@acme.com`). Fix BUG-212.
- **Rule:** Account name inference priority: (1) `name::` param → (2)
  `oauthAccount.emailAddress` from `~/.claude.json` → (3) `_active_*` marker as last resort.
  The marker may be stale (pointing to a previously active account).
- **Source fn:** `test_apply_refresh_lifecycle_active_marker_unchanged` in
  `tests/usage/refresh_tests_a.rs` (verifies marker is not used as identity source during
  refresh); account save integration tests in `tests/cli/account_mutations_test.rs`
- **Source:** [pitfall/004_account_identity_pitfalls.md §P2](../../../docs/pitfall/004_account_identity_pitfalls.md)
