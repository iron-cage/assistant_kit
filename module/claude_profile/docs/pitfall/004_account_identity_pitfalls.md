# Pitfall: Account Identity Pitfalls

### Scope

- **Purpose**: Document failure modes in account identity inference and active-marker naming.
- **Responsibility**: Covers test fixture hostname collision and account name source priority ordering.
- **In Scope**: `_active_{host}_{user}` marker identity collisions; `account_save_routine()` name inference; BUG-308, BUG-212, BUG-582.
- **Out of Scope**: Active marker format (→ schema/005); multi-machine marker mechanics (→ state_machine/001).

### Pattern

Account identity — name inference, machine-specific markers, and test fixture naming — has several subtle failure modes.

### Pitfall 1 — Active marker filename can collide with test machine identity (BUG-308)

Tests that hardcode `_active_devbox_devuser` as an "other machine" marker will fail on machines where `hostname=devbox` and `user=devuser` — the fixture name matches the own marker, causing it to be returned by `active_marker_filename()` instead of treated as a foreign marker.

**Fix:** Use synthetic hostnames (`_active_testhost1_tst1`, `_active_testhost2_tst2`) that will never match a real machine. Add `assert_ne!` guards verifying the synthetic names differ from the actual own marker.

**Rule:** Never hardcode real-looking hostnames in test fixtures for `_active_*` markers. Always use synthetic names that cannot collide with any realistic machine identity.

### Pitfall 2 — Account name must not come from the `_active` marker or `~/.claude.json` first (BUG-212, BUG-582)

Early save logic read the active marker to infer the account name when `name::` was omitted. The active marker may be stale (another account was previously active).
BUG-212 put `oauthAccount.emailAddress` from `~/.claude.json` ahead of it, but that can be stale too: a running Claude process rewrites `~/.claude.json` from its
own profile, so after a switch it can still name the previous account (BUG-582).

**Fix:** `account_save_routine()` first takes the one saved slot whose `claudeAiOauth.refreshToken` equals the live session's, or whose `accessToken` does for a
redirect key (`slots_holding_session()`, BUG-582).
With no single holder it reads `oauthAccount.emailAddress` from `~/.claude.json`, and falls back to the `_active_*` marker only when that field is absent or empty.
The same lag affects identity: a save naming the holding slot keeps that slot's stored `oauthAccount` unless `~/.claude.json` names that account or
the slot ([002](../feature/002_account_save.md) step 6a, BUG-582), instead of copying another account's into it.

**Rule:** Account name inference priority: (1) `name::` param → (2) the one saved slot holding the live session → (3) `oauthAccount.emailAddress` from
`~/.claude.json` → (4) `_active_{host}_{user}` marker as last resort.

### Features

| File | Relationship |
|------|-------------|
| [feature/025_per_machine_active_marker.md](../feature/025_per_machine_active_marker.md) | Per-machine marker format and `active_marker_filename()` |
| [feature/002_account_save.md](../feature/002_account_save.md) | Name inference algorithm |

### Schema

| File | Relationship |
|------|-------------|
| [schema/005](../schema/005_active_marker.md) | Active marker schema |
| [schema/007](../schema/007_claude_json.md) | `~/.claude.json` `oauthAccount.emailAddress` field |
