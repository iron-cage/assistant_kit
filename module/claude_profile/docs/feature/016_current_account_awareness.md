# Feature: Current Account Awareness

### Scope

- **Purpose**: Expose the "current account" (the one whose credentials are live in `~/.claude/.credentials.json`) alongside the "active account" (`_active` marker) in both `.accounts` and `.usage`, with a clear visual indicator when they diverge.
- **Responsibility**: Documents the `is_current` field for `.accounts` and the `*` active-divergence marker for `.usage`, plus the shared token-matching algorithm.
- **In Scope**: `current::` field-presence toggle and `Current:` line in `.accounts`; `*` flag for `_active`-but-not-current accounts in `.usage`; JSON field additions (`is_current` in both commands, `is_active` in `.usage`); graceful degradation when `~/.claude/.credentials.json` is unreadable.
- **Out of Scope**: `.token.status` (reads live credentials but doesn't compare against saved accounts); `.account.limits` (single-account query, no divergence display); `.credentials.status` (live-only, no account store involved).

### Design

Two distinct concepts govern account identity:

| Concept | Source | Managed by |
|---------|--------|-----------|
| **Active account** | `{credential_store}/_active` file content | `clp .account.use` |
| **Current account** | `accessToken` match in `~/.claude/.credentials.json` | `claude auth login` or any external credential change |

These diverge when an external actor (`claude auth login`, a direct credential write, or automated rotation) changes `~/.claude/.credentials.json` without going through `clp`. The divergence is a normal operational state that the user needs to see clearly.

**Current account detection algorithm (shared):**
1. Read `accessToken` from `~/.claude/.credentials.json` → live token.
2. For each saved account, read `accessToken` from `{credential_store}/{name}.credentials.json`.
3. The account whose stored token equals the live token is the current account (`is_current = true`). At most one account matches.
4. When `~/.claude/.credentials.json` is absent or unreadable: `is_current = false` for all accounts (graceful degradation).

**Changes to `.accounts`:**
- New `is_current: bool` per-account field derived from the detection algorithm above.
- `Current:` output line (default on) controlled by `current::` field-presence toggle.
- Text format: `Current:  yes` / `Current:  no` (same indentation as `Active:`).
- When credentials file unreadable: `Current:` line is suppressed entirely for all accounts (no misleading `no`).
- JSON: new `is_current` boolean field per account object.

**Changes to `.usage`:**
- `AccountQuota` internal struct: rename `active` → `is_current`; add `is_active: bool` (from `_active` marker).
- Flag column semantics (single character, priority order):
  1. `✓` — `is_current = true` (this account is currently in use by Claude)
  2. `*` — `is_active = true` AND `is_current = false` (saved-active but not the live account)
  3. `→` — recommendation marker (unchanged)
  4. ` ` — none of the above
- When current = active (normal case): only `✓` appears; no `*` on any row.
- When current ≠ active: `✓` on current row, `*` on active row; divergence is immediately visible.
- When credentials file unreadable: no `✓` on any row; `*` still appears for the `_active` account.
- JSON: field `active` renamed to `is_current`; new `is_active` boolean field added per object.

**Divergence display example (`.accounts`):**

```
alice@acme.com
  Active:  no
  Current: yes    ← live credentials belong to alice
  Sub:     max
  Expires: in 7h

work@acme.com
  Active:  yes    ← _active points here
  Current: no     ← but alice is actually live
  Sub:     pro
  Expires: in 5h
```

**Divergence display example (`.usage`):**

```
  Account         Expires    5h Left  ...
✓ alice@acme.com  in 7h 24m  86%     ...   ← current
* work@acme.com   in 5h 02m  100%    ...   ← _active but not current
  other@acme.com  EXPIRED    —       ...
```

### Acceptance Criteria

- **AC-01**: `.accounts` shows `Current:  yes` on the account whose `accessToken` matches `~/.claude/.credentials.json`; all other accounts show `Current:  no`.
- **AC-02**: `.accounts` suppresses the `Current:` line entirely when `~/.claude/.credentials.json` is absent or unreadable.
- **AC-03**: `.accounts` `current::0` suppresses the `Current:` line; `current::1` (default) shows it.
- **AC-04**: `.accounts` JSON includes an `is_current` boolean field per account object.
- **AC-05**: `.usage` `✓` marks the account whose `accessToken` matches live credentials (NOT the `_active` marker).
- **AC-06**: `.usage` `*` marks the account with the `_active` marker when it differs from the current account; no `*` appears when current = active.
- **AC-07**: When `.usage` credentials file is unreadable: no `✓` on any row; `*` still marks the `_active` account.
- **AC-08**: `.usage` JSON output uses `is_current` (replacing the old `active` field) and adds `is_active` per object.
- **AC-09**: When no saved account matches the live token, `.usage` prepends a synthetic `(current session)` row with `✓` and no `*`.
- **AC-10**: `.accounts` `is_current` and `.usage` `is_current` both use the same detection algorithm (token equality, no hashing, no prefix matching).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/commands.rs` | `accounts_routine()` — `is_current` detection, `Current:` line, `current::` param |
| source | `src/usage.rs` | `fetch_all_quota()` — `is_current` via token matching; `is_active` from `_active` marker; `*` flag rendering |
| doc | [003_account_list.md](003_account_list.md) | `.accounts` base command — field table and AC extended here |
| doc | [009_token_usage.md](009_token_usage.md) | `.usage` base command — flag column and JSON schema extended here |
| doc | [cli/params.md](../cli/params.md#parameter--18-current) | `current::` field-presence parameter |
| doc | [cli/commands.md](../cli/commands.md) | Syntax blocks for `.accounts` and `.usage` |
| test | `tests/cli/accounts_test.rs` | IT-26, IT-27, IT-28 — current detection in `.accounts` |
| test | `tests/cli/usage_test.rs` | IT-13..IT-16 — live detection and active divergence in `.usage` |
