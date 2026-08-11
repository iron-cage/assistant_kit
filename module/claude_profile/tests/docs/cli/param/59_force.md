# Test: `force::` Parameter (Ownership Gate Bypass)

Edge case coverage for the `force::` bool bypass param on `.account.use`, `.account.delete`,
`.account.relogin`, and `.accounts` (with `owner::0` / `owner::USER@MACHINE` — Feature 064).
See [param/058_force.md](../../../../docs/cli/param/058_force.md) for specification.

`force::1` bypasses G5–G8 ownership enforcement gates. Does NOT bypass G1–G4 (read-side gates).
When combined with `dry::1`: gate is bypassed but mutation is still previewed without writing.
`force::1` without a mutation param is silently ignored (no error).

**Behavioral Divergence Pair:** EC-1 ↔ EC-5 — `force::1` on an other-owned account bypasses G5 and exits 0 (switch performed); `force::0` (the default) enforces G5 and exits 1 with ownership violation message.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `force::1` on `.account.use` bypasses G5; exits 0 | G5 Bypass |
| EC-2 | `force::1` on `.account.delete` bypasses G6; exits 0 | G6 Bypass |
| EC-3 | `force::1` on `.account.relogin` bypasses G7; exits 0 | G7 Bypass |
| EC-4 | `force::1 owner::0 name::X` on `.accounts` bypasses G8; exits 0 (Feature 064) | G8 Bypass |
| EC-5 | `force::0` (default) — ownership gates enforced normally | Default |
| EC-6 | `force::1 dry::1` — gate bypassed, mutation previewed without writing | Dry-run Interaction |
| EC-7 | `force::1` without mutation param — silently ignored; no error | No-op |
| EC-8 | `force::1` does NOT bypass G1–G4 (read-side gates) | Scope Limit |

---

### EC-1: `force::1` on `.account.use` bypasses G5; exits 0

- **Given:** Account `alice@corp.com` has `alice@corp.com.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.use name::alice@corp.com force::1`
- **Then:** Exits 0. G5 ownership gate bypassed — no ownership violation message. `~/.claude/.credentials.json` updated to `alice@corp.com`'s credentials. Active marker updated.
- **Exit:** 0
- **Source fn:** `ft18_use_force_bypasses_g5` (in `tests/cli/account_ownership_test.rs`)
- **Source:** [param/058_force.md](../../../../docs/cli/param/058_force.md)

---

### EC-2: `force::1` on `.account.delete` bypasses G6; exits 0

- **Given:** Account `alice@corp.com` has `alice@corp.com.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.delete name::alice@corp.com force::1`
- **Then:** Exits 0. G6 ownership gate bypassed. `alice@corp.com.credentials.json` and `alice@corp.com.json` deleted from credential store.
- **Exit:** 0
- **Source fn:** `ft19_delete_force_bypasses_g6` (in `tests/cli/account_ownership_test.rs`)
- **Source:** [param/058_force.md](../../../../docs/cli/param/058_force.md)

---

### EC-3: `force::1` on `.account.relogin` bypasses G7; exits 0

- **Given:** Account `alice@corp.com` has `alice@corp.com.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.relogin name::alice@corp.com force::1`
- **Then:** Exits 0. G7 ownership gate bypassed — no ownership violation message. The 6-step relogin procedure proceeds.
- **Exit:** 0
- **Source fn:** `ft20_relogin_force_bypasses_g7` (in `tests/cli/account_ownership_test.rs`)
- **Source:** [param/058_force.md](../../../../docs/cli/param/058_force.md)

---

### EC-4: `force::1 owner::0 name::X` on `.accounts` bypasses G8; exits 0 (Feature 064)

- **Given:** Account `alice@corp.com` has `alice@corp.com.json` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .accounts owner::0 name::alice@corp.com force::1` (formerly `unclaim::1 name::X force::1` — Feature 064)
- **Then:** Exits 0. G8 ownership gate bypassed. `alice@corp.com.json` updated with `"owner": ""`. stdout contains `unclaimed alice@corp.com`.
- **Exit:** 0
- **Source fn:** `ft20_accounts_unclaim_force_bypasses_g8` (in `tests/cli/accounts_ft_test.rs`); also covered by `ec14_owner_zero_force_bypasses_g8` (in `tests/cli/account_owner_param_test.rs`)
- **Source:** [param/058_force.md](../../../../docs/cli/param/058_force.md)

---

### EC-5: `force::0` (default) — ownership gates enforced normally

- **Given:** Account `alice@corp.com` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.use name::alice@corp.com force::0` (equivalent to omitting `force::`)
- **Then:** Exits 1 with `"ownership violation: this account is owned by other@remote"`. No switch performed. G5 gate enforced normally.
- **Exit:** 1
- **Source fn:** `ft08_use_exits_1_when_not_owned` (in `tests/cli/account_ownership_test.rs`)
- **Source:** [param/058_force.md](../../../../docs/cli/param/058_force.md)

---

### EC-6: `force::1 dry::1` — gate bypassed; mutation previewed without writing (no mtime check — content/existence checks only)

> **Semantic drift correction:** the G5 sub-block (`.account.use`) in the cited test does not capture or compare any file mtime — it never touches `~/.claude/.credentials.json` at all. It asserts only: exit 0, no ownership-violation text in stderr, and `[dry-run]` present in stdout. The doc's "mtime unchanged" claim is not verified by any of the four sub-blocks in this function: G6 (`.account.delete`) checks file *existence* (`account_exists(...)`) rather than mtime; G7 (`.account.relogin`) has no additional file check beyond the same three checks as G5; G8 (`.accounts owner::0`) checks *content equality* (`assert_eq!` on `meta_before`/`meta_after`) rather than mtime. No sub-block in this function performs mtime capture or comparison.

- **Given:** Account `alice@corp.com` with `"owner": "other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.use name::alice@corp.com force::1 dry::1`
- **Then:** Exits 0. No ownership violation (G5 bypassed). stdout contains `[dry-run]` preview message. (The test does NOT capture or compare `~/.claude/.credentials.json` mtime — it never touches that file in this sub-block.)
- **Exit:** 0
- **Note:** force runs BEFORE dry: gate is bypassed first, then dry-run prevents the actual write. Same ordering applies to delete, relogin, and `.accounts owner::0 name::X` (Feature 064) — but none of the four sub-cases verify "no write occurred" via mtime; G6 uses file-existence, G8 uses content-equality, G5 and G7 use exit/stderr/stdout checks only.
- **Source fn:** `ft21_force_dry_bypasses_gate_previews` (in `tests/cli/account_ownership_test.rs`) — single function covering all four G5/G6/G7/G8 sub-cases in one body; none of the four sub-blocks perform mtime capture/comparison, contrary to this EC's original claim
- **Source:** [param/058_force.md](../../../../docs/cli/param/058_force.md)

---

### EC-7: `force::1` without mutation param — silently ignored; no error

- **Given:** Credential store with accounts. No `owner::`, `assignee::`, or explicit switch in progress.
- **When (case A):** `clp .accounts force::1` (list accounts, no mutation)
- **Then (case A):** Exits 0. Normal `.accounts` listing output. No error about `force::1`. `force::` is silently ignored.
- **When (case B):** `clp .accounts force::1 assignee::testuser@testmachine name::alice@corp.com` (Feature 065; formerly `active::` — Feature 064)
- **Then (case B):** Exits 0. Marker file written normally. `force::1` has no gate to bypass on `assignee::` path — silently ignored.
- **Exit:** 0 (both cases)
- **Source fn:** `ft21_force_no_effect_without_unclaim` (in `tests/cli/accounts_ft_test.rs`)
- **Source:** [param/058_force.md](../../../../docs/cli/param/058_force.md)

---

### EC-8: `force::1` does NOT bypass G1–G4 (read-side gates)

- **Given:** Account `alice@corp.com` with `"owner": "other@remote"`. Cache contains valid quota. Current identity ≠ `"other@remote"`.
- **When:** `clp .usage name::alice@corp.com force::1 trace::1`
- **Then:** Exits 0. `... · fetch  alice@corp.com  skipped (reason: not owned)` appears in output (G1 gate active — cache-as-primary). HTTP fetch NOT performed. Quota columns show cached values with `~` prefix. `force::1` does NOT trigger live fetch for non-owned accounts.
- **Exit:** 0 with cache-sourced data
- **Note:** G1–G4 are read-side gates that intentionally suppress load on non-owned accounts. `force::` is scoped to write mutations only — G5–G8.
- **Source fn:** *(coverage gap — no test combines `force::1` with a plain `.usage` call on a non-owned/cached account to prove G1 isn't bypassed. Closest adjacent evidence: `t07_gate9_not_bypassed_by_force_during_rotate` in `tests/cli/account_claim_lock_reserve_test.rs` proves `force::1` doesn't bypass Gate 9, a different gate; `it259_solo_current_not_owned_no_http` in `tests/cli/usage_solo_test.rs` proves G1 fires under `solo::1`, but without `force::1` present at all)*
- **Source:** [param/058_force.md](../../../../docs/cli/param/058_force.md)
