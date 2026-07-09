# Parameter 018: `current::` — Edge Cases

**Behavioral Divergence Pair:** EC-01 ↔ EC-02 — `current::1` (default) shows the `Current:` line per account, comparing each stored `accessToken` against the live `~/.claude/.credentials.json`; `current::0` omits the `Current:` line entirely from all account blocks — same command shape, observably different output presence.

Setup context for these cases is shared with the `.accounts` command-level integration tests in [command/03_accounts.md](../command/03_accounts.md) (IT-26, IT-27, IT-28) — this file documents `current::`'s own parameter-level behavior with its own complete case list, per One Element One Spec.

### Test Case Index

| ID | Test | Scenario | Expected | Status |
|----|------|----------|----------|--------|
| EC-01 | `ec1_current_1_default_shows_matching_line` | `current::1` (default), live token matches account `work@acme.com` | `work@acme.com` block shows `Current: yes`; other accounts show `Current: no` | ✅ |
| EC-02 | `ec2_current_0_omits_line_entirely` | `current::0` | no `Current:` line appears in any account block | ✅ |
| EC-03 | `ec3_current_omitted_defaults_to_1` | `.accounts` with no `current::` | behaves identically to `current::1` — line shown | ✅ |
| EC-04 | `ec4_current_invalid_value_exits_1` | `current::maybe` (non-boolean) | exit 1 — invalid boolean value rejected | ✅ |
| EC-05 | `ec5_current_1_credentials_unreadable_suppressed` | `current::1`, `~/.claude/.credentials.json` absent or unreadable | `Current:` line suppressed for all accounts regardless of toggle — equivalent to `current::0` | ✅ |
| EC-06 | `ec6_current_true_false_aliases_accepted` | `current::true` and `current::false` | `true` behaves as `1`, `false` behaves as `0` | ✅ |
| EC-07 | `ec7_current_0_format_json_still_includes_is_current` | `current::0 format::json` | JSON output still includes `is_current` per account object regardless of toggle | ✅ |
| EC-08 | `ec8_current_1_no_match_all_no` | `current::1`, no stored account's `accessToken` matches the live credentials file | all account blocks show `Current: no` | ✅ |
| EC-09 | `ec9_current_and_active_independent_divergence` | current account ≠ active account (divergence case) | active account row shows `Active: yes` / `Current: no`; current account row shows `Active: no` / `Current: yes` | ✅ |

**Total:** 9 edge case tests

---

### EC-01: `current::1` (default) — shows matching `Current: yes`/`Current: no` line

- **Given:** Two accounts: `alice@acme.com` (active) and `work@acme.com`. The live `~/.claude/.credentials.json` has an `accessToken` matching `work@acme.com`'s stored token.
- **When:** `clp .accounts current::1`
- **Then:** Exits 0. `work@acme.com` block contains `Current: yes`; `alice@acme.com` block contains `Current: no`.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [command/03_accounts.md IT-26](../command/03_accounts.md)

---

### EC-02: `current::0` — omits `Current:` line entirely

- **Given:** Two accounts saved. Live `~/.claude/.credentials.json` matches one of them.
- **When:** `clp .accounts current::0`
- **Then:** Exits 0. stdout does NOT contain any `Current:` line in any account block — the field-presence toggle suppresses the line entirely, independent of match state.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [command/03_accounts.md IT-28](../command/03_accounts.md)

---

### EC-03: Omitted `current::` defaults to `1`

- **Given:** Two accounts saved. Live `~/.claude/.credentials.json` matches one of them.
- **When:** `clp .accounts` (no `current::` provided)
- **Then:** Exits 0. Behavior is identical to `current::1` — `Current:` line shown per account, same as EC-01.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md)

---

### EC-04: Invalid boolean value rejected

- **Given:** Any account and credential state.
- **When:** `clp .accounts current::maybe`
- **Then:** Exits 1. stderr indicates `maybe` is not a valid boolean value for `current::`.
- **Exit:** 1
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md)

---

### EC-05: Unreadable credentials file suppresses `Current:` regardless of toggle

- **Given:** Two accounts saved. `~/.claude/.credentials.json` is absent (or has no read permission).
- **When:** `clp .accounts current::1`
- **Then:** Exits 0. Output contains `Active:`, `Sub:`, `Tier:`, `Expires:`, `Email:` lines for each account but does NOT contain any `Current:` line — the live token cannot be determined, so the line is suppressed for all accounts even though `current::1` requested it.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [command/03_accounts.md IT-27](../command/03_accounts.md)

---

### EC-06: `true`/`false` boolean aliases accepted

- **Given:** Two accounts saved. Live `~/.claude/.credentials.json` matches one of them.
- **When:**
  1. `clp .accounts current::true`
  2. `clp .accounts current::false`
- **Then:** 1. Behaves identically to `current::1` — `Current:` line shown, exit 0.
  2. Behaves identically to `current::0` — `Current:` line omitted, exit 0.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md)

---

### EC-07: `current::0` does not affect `format::json` — `is_current` always present

- **Given:** Two accounts saved. Live `~/.claude/.credentials.json` matches one of them.
- **When:** `clp .accounts current::0 format::json`
- **Then:** Exits 0. Valid JSON array where every object contains the `is_current` boolean field regardless of the `current::0` toggle — `format::json` always includes it per the parameter's Notes.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [command/03_accounts.md IT-28](../command/03_accounts.md)

---

### EC-08: `current::1` with no matching account — all show `Current: no`

- **Given:** Two accounts saved: `alice@acme.com` and `work@acme.com`. `~/.claude/.credentials.json` exists and is readable but its `accessToken` does not match either stored account's token.
- **When:** `clp .accounts current::1`
- **Then:** Exits 0. Both `alice@acme.com` and `work@acme.com` blocks contain `Current: no`.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md)

---

### EC-09: Active and current diverge — independent per-account flags

- **Given:** `alice@acme.com` is the active account (per-machine active marker). The live `~/.claude/.credentials.json` token matches `work@acme.com` instead (current ≠ active).
- **When:** `clp .accounts current::1`
- **Then:** Exits 0. `alice@acme.com` block shows `Active: yes` and `Current: no`. `work@acme.com` block shows `Active: no` and `Current: yes`. The two flags are independent and both correctly diverge.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [feature/016_current_account_awareness.md](../../../../docs/feature/016_current_account_awareness.md)
