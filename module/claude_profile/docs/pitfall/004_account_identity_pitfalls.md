# Pitfall: Account Identity Pitfalls

### Pattern

Account identity — name inference, machine-specific markers, and test fixture naming — has several subtle failure modes.

### Pitfall 1 — Active marker filename can collide with test machine identity (BUG-308)

Tests that hardcode `_active_w003_user1` as an "other machine" marker will fail on machines where `hostname=w003` and `user=user1` — the fixture name matches the own marker, causing it to be returned by `active_marker_filename()` instead of treated as a foreign marker.

**Fix:** Use synthetic hostnames (`_active_testhost1_tst1`, `_active_testhost2_tst2`) that will never match a real machine. Add `assert_ne!` guards verifying the synthetic names differ from the actual own marker.

**Rule:** Never hardcode real-looking hostnames in test fixtures for `_active_*` markers. Always use synthetic names that cannot collide with any realistic machine identity.

### Pitfall 2 — Account name must come from `oauthAccount.emailAddress`, not `_active` marker (BUG-212)

Early save logic read the active marker to infer the account name when `name::` was omitted. The active marker may be stale (another account was previously active). The correct source is `oauthAccount.emailAddress` from `~/.claude.json`, which reflects the identity of the current session.

**Fix:** `account_save_routine()` reads `oauthAccount.emailAddress` from `~/.claude.json` as the primary name source. Falls back to the `_active_*` marker only if `~/.claude.json` is absent or the field is missing.

**Rule:** Account name inference priority: (1) `name::` param → (2) `oauthAccount.emailAddress` from `~/.claude.json` → (3) `_active_{host}_{user}` marker as last resort.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/025_per_machine_active_marker.md](../feature/025_per_machine_active_marker.md) | Per-machine marker format and `active_marker_filename()` |
| [feature/002_account_save.md](../feature/002_account_save.md) | Name inference algorithm |
| [schema/005](../schema/005_active_marker.md) | Active marker schema |
| [schema/007](../schema/007_claude_json.md) | `~/.claude.json` `oauthAccount.emailAddress` field |
