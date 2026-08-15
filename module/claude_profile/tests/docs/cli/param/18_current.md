# Parameter 018: `current::` — Edge Cases

> **Removed** (Feature 037): `current::` as a standalone boolean toggle is fully removed — `.accounts` (the only command that ever accepted it) now rejects any `current::` value outright with `parameter 'current' removed — use 'cols::-current' instead` (see `REMOVED_TOGGLES` in `src/commands/accounts.rs`). See [docs/cli/param/018_current.md](../../../../docs/cli/param/018_current.md) for the removal notice. The `Current:`-line display behavior the toggle used to gate remains live on `.accounts` — now controlled by the `cols::` column-selector modifier (`cols::-current` / `cols::+current`; generic `cols::` mechanics are documented in [33_cols.md](33_cols.md), which has no `current`-specific case of its own) — and this file remains the case-list home for that display behavior per One Element One Spec, backed by `accounts_list_test_b.rs` (`acc31_accounts_shows_current_yes_no`, `acc32_accounts_suppresses_current_when_creds_absent`, `acc33_accounts_current_param_and_json`) rather than dedicated `current::`-named tests. **Citation correction:** all 9 `ecN_current_*`-named functions cited below never existed anywhere in the test suite — confirmed by grep across `src/`, `claude_profile_core/tests/`, and `tests/`, where each name appears only in this doc file itself. They are fabricated citations, not stale post-removal renames. Cases below are re-cited to their real covering test where behavior matches; cases with no identified current equivalent are marked as a coverage gap.

**Behavioral Divergence Pair:** EC-01 ↔ EC-02 — bare `.accounts` (default, no `cols::` modifier) shows the `Current:` line per account, comparing each stored `accessToken` against the live `~/.claude/.credentials.json`; `cols::-current` omits the `Current:` line entirely from all account blocks — same command shape apart from the modifier, observably different output presence.

Setup context for these cases is shared with the `.accounts` command-level integration tests in [command/03_accounts.md](../command/03_accounts.md) (IT-26, IT-27, IT-28) — this file documents the `Current:`-line's own parameter-level behavior with its own complete case list, per One Element One Spec.

### Test Case Index

| ID | Test | Scenario | Expected | Status |
|----|------|----------|----------|--------|
| EC-01 | `acc31_accounts_shows_current_yes_no` | bare `.accounts` (default), live token matches account `work@acme.com` | `work@acme.com` block shows `Current: yes`; `alice@acme.com` block shows `Current: no` | ✅ |
| EC-02 | `acc33_accounts_current_param_and_json` (part a) | `cols::-current` | no `Current:` line appears in any account block | ✅ |
| EC-03 | *(coverage gap)* | omitted-vs-provided contrast — moot now that `current::` can never be provided | N/A — default-shown behavior already covered by EC-01 | N/A |
| EC-04 | *(coverage gap)* | `current::maybe` (any value) | now rejected as a removed parameter, not a boolean-validation error | N/A |
| EC-05 | `acc32_accounts_suppresses_current_when_creds_absent` | bare `.accounts`, `~/.claude/.credentials.json` absent | `Current:` line suppressed for all accounts | ✅ |
| EC-06 | *(coverage gap)* | `current::true` / `current::false` | now rejected identically to any other `current::` value — no longer "accepted as aliases" | N/A |
| EC-07 | `acc33_accounts_current_param_and_json` (part b) | `format::json` (standalone) | JSON output includes `is_current` per account object unconditionally | ✅ |
| EC-08 | *(coverage gap)* | live creds present but matching no stored account | no identified current equivalent — no test exercises an all-non-matching live-creds scenario | N/A |
| EC-09 | *(coverage gap)* | current account ≠ active account (divergence case) | no identified current equivalent — no test asserts `Active:` and `Current:` together | N/A |

**Total:** 9 edge cases documented — 4 with live coverage (EC-01, EC-02, EC-05, EC-07), 5 coverage gaps (EC-03, EC-04, EC-06, EC-08, EC-09)

---

### EC-01: bare `.accounts` (default) — shows matching `Current: yes`/`Current: no` line

- **Given:** Two accounts with `accessToken` fields: `work@acme.com` (token `tok-work`) and `alice@acme.com` (token `tok-alice`). The live `~/.claude/.credentials.json` has an `accessToken` matching `work@acme.com`'s stored token.
- **When:** `clp .accounts` (bare — `current::` can no longer be passed as a literal parameter at all; this is the only reachable state)
- **Then:** Exits 0. `work@acme.com` block contains `Current: yes`; `alice@acme.com` block contains `Current: no`.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [command/03_accounts.md IT-26](../command/03_accounts.md)

---

### EC-02: `cols::-current` — omits `Current:` line entirely

- **Given:** One account `alice@acme.com` with `accessToken` `tok-alice`. Live `~/.claude/.credentials.json` matches it.
- **When:** `clp .accounts cols::-current`
- **Then:** Exits 0. stdout does NOT contain any `Current:` line — the `cols::` column-selector modifier suppresses the line entirely, independent of match state. This is the live successor mechanism; there is no `current::0` toggle anymore.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [command/03_accounts.md IT-28](../command/03_accounts.md)

---

### EC-03: Omitted-vs-provided contrast — coverage gap (moot)

> **Coverage gap.** The original claim — omitting `current::` behaves identically to explicitly passing `current::1` — is now moot: `current::` cannot be provided at all (any value is rejected as a removed parameter; see the file-level note above), so there is no "omitted vs. provided" contrast left to construct. The always-on default-shows-the-line behavior itself is already covered by EC-01 (`acc31_accounts_shows_current_yes_no`).

- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md)

---

### EC-04: Invalid value rejected — coverage gap

> **Coverage gap.** No test in the suite passes `current::` as a literal CLI argument with any value (confirmed by grep — no `"current::` string appears in any `.rs` test file). The original claim is also stale: any `current::` value, valid-boolean or not, now triggers the same `parameter 'current' removed` rejection (`REMOVED_TOGGLES` in `src/commands/accounts.rs`), not a type-validation error specific to a non-boolean value like `maybe`.

- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md)

---

### EC-05: Absent credentials file suppresses `Current:` for all accounts

- **Given:** Two accounts saved (`work@acme.com`, `alice@acme.com`). `~/.claude/.credentials.json` is absent — deliberately not written.
- **When:** `clp .accounts`
- **Then:** Exits 0. stdout does NOT contain any `Current:` line — the live token cannot be determined, so the line is suppressed for all accounts. (Live coverage is for the file-absent case only; an unreadable-but-present file, e.g. permission-denied, is not separately tested.)
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [command/03_accounts.md IT-27](../command/03_accounts.md)

---

### EC-06: `true`/`false` aliases — coverage gap (behavior claim stale)

> **Coverage gap.** No test passes `current::` as a literal CLI argument (same grep evidence as EC-04). The original claim is additionally stale: `current::true`/`current::false` are no longer "accepted as aliases" — every `current::` value, including `true`/`false`, is now rejected identically as a removed parameter.

- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md)

---

### EC-07: `format::json` always includes `is_current`

- **Given:** One account `alice@acme.com` with `accessToken` `tok-alice`. Live `~/.claude/.credentials.json` matches it.
- **When:** `clp .accounts format::json` (standalone — not combined with `cols::-current` or any suppression modifier in the same invocation)
- **Then:** Exits 0. Valid JSON array where the object contains the `is_current` boolean field. The real test proves this unconditionally via a plain `format::json` call rather than in combination with a line-suppression modifier as originally claimed — `format::json` always includes it per the parameter's Notes.
- **Exit:** 0
- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [command/03_accounts.md IT-28](../command/03_accounts.md)

---

### EC-08: No matching account — coverage gap

> **Coverage gap.** No test exercises a present-and-readable live-credentials file whose `accessToken` matches NEITHER stored account (all accounts showing `Current: no` simultaneously). `acc31_accounts_shows_current_yes_no` covers a partial-match case (one account matches, one doesn't); no all-non-matching scenario exists in the suite.

- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md)

---

### EC-09: Active/current divergence — coverage gap

> **Coverage gap.** No test combines `Active:` and `Current:` assertions to demonstrate independent divergence (active account ≠ current account). `acc31`/`acc33` assert `Current:` only; no test in the suite asserts `Active:` and `Current:` together on the same output.

- **Source:** [param/018_current.md](../../../../docs/cli/param/018_current.md), [feature/016_current_account_awareness.md](../../../../docs/feature/016_current_account_awareness.md)
