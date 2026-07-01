# Feature 063 — Explicit Ownership Claim

### Test Case Index

| ID | Test | Verifies | Status |
|----|------|----------|--------|
| FT-01 | `ft_owner_sets_owner_field` | AC-01: `owner::user1@w003 name::X` writes owner field | ✅ |
| FT-02 | `ft_owner_requires_name` | AC-02: exits 1 when `name::` absent | ✅ |
| FT-03 | `ft_owner_g8_blocks_non_owner` | AC-03: G8 gate — owned by another → exit 1 | ✅ |
| FT-04 | `ft_owner_unowned_passes_g8` | AC-04: unowned account → write succeeds | ✅ |
| FT-05 | `ft_owner_mutual_exclusion_unclaim` | AC-05: `owner:: + unclaim::1` → exit 1 (now via REMOVED_TOGGLE on `unclaim::1`, Feature 064) | ✅ |
| FT-06 | `ft_owner_dry_run_preview` | AC-06: dry::1 → preview, no file writes | ✅ |
| FT-07 | `ft_owner_force_bypasses_g8` | AC-07: force::1 bypasses G8 for other-owned account | ✅ |
| FT-08 | `ft_owner_trace_emits_diagnostic` | AC-08: trace::1 → stderr diagnostic | ✅ |
| FT-09 | `ft_owner_prefix_resolution` | AC-09: short name resolves to full email | ✅ |
| FT-10 | `ft_owner_empty_value_rejected` | AC-10: empty owner:: → exit 1 | ✅ |
| FT-11 | `ft_owner_gates_respect_new_value` | AC-11: subsequent ops respect new owner | ✅ |
| FT-12 | `ft_owner_works_on_usage` | AC-12: `.usage owner::` same behavior as `.accounts owner::` | ✅ |

**Total:** 12 test cases (12 FT)

---

### FT-01: `owner::user1@w003 name::X` writes owner field

- **Given:** `alice@corp.com.json` exists in credential store. Account is unowned (`"owner": ""`).
- **When:** `clp .accounts owner::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. `alice@corp.com.json` contains `"owner": "user1@w003"`. stdout contains `owned alice@corp.com by user1@w003`. Credentials file unchanged.
- **Exit:** 0
- **Source fn:** `ft_owner_sets_owner_field` (in `tests/cli/account_mutations_test.rs`)

---

### FT-02: `owner::` without `name::` exits 1

- **Given:** Credential store with accounts present. No `name::` provided.
- **When:** `clp .accounts owner::user1@w003` (no `name::`)
- **Then:** Exits 1. stderr contains usage error indicating `name::` is required when setting a non-`0` owner identity.
- **Exit:** 1
- **Source fn:** `ft_owner_requires_name` (in `tests/cli/account_mutations_test.rs`)

---

### FT-03: G8 gate blocks non-owner from setting `owner::`

- **Given:** `alice@corp.com.json` has `"owner": "other@remote"`. Caller identity is not `"other@remote"`. No `force::1`.
- **When:** `clp .accounts owner::user1@w003 name::alice@corp.com`
- **Then:** Exits 1. stderr contains ownership violation message. `alice@corp.com.json` is NOT modified.
- **Exit:** 1
- **Source fn:** `ft_owner_g8_blocks_non_owner` (in `tests/cli/account_mutations_test.rs`)

---

### FT-04: Unowned account passes G8; `owner::` write succeeds

- **Given:** `alice@corp.com.json` has `"owner": ""` (unowned). Caller is `user1@w003`.
- **When:** `clp .accounts owner::user1@w003 name::alice@corp.com`
- **Then:** Exits 0. G8 passes because account is unowned. `alice@corp.com.json` contains `"owner": "user1@w003"`.
- **Exit:** 0
- **Source fn:** `ft_owner_unowned_passes_g8` (in `tests/cli/account_mutations_test.rs`)

---

### FT-05: `owner::` combined with `unclaim::1` exits 1 (REMOVED_TOGGLE)

- **Given:** `alice@corp.com.json` exists. `unclaim::1` is a REMOVED_TOGGLE (Feature 064).
- **When:** `clp .accounts owner::user1@w003 unclaim::1 name::alice@corp.com`
- **Then:** Exits 1. `unclaim::1` triggers REMOVED_TOGGLE handler with migration message directing to `owner::0`. No file writes performed.
- **Exit:** 1
- **Source fn:** `ft_owner_mutual_exclusion_unclaim` (in `tests/cli/account_mutations_test.rs`)

---

### FT-06: `dry::1` previews ownership write without modifying files

- **Given:** `alice@corp.com.json` exists and is unowned.
- **When:** `clp .accounts owner::user1@w003 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would set owner of alice@corp.com to user1@w003`. `alice@corp.com.json` is NOT modified — still contains `"owner": ""`.
- **Exit:** 0
- **Source fn:** `ft_owner_dry_run_preview` (in `tests/cli/account_mutations_test.rs`)

---

### FT-07: `force::1` bypasses G8 for other-owned account

- **Given:** `alice@corp.com.json` has `"owner": "other@remote"`. Caller is `user1@w003` (G8 would block without force).
- **When:** `clp .accounts owner::user1@w003 name::alice@corp.com force::1`
- **Then:** Exits 0. G8 bypassed by `force::1`. `alice@corp.com.json` contains `"owner": "user1@w003"`.
- **Exit:** 0
- **Source fn:** `ft_owner_force_bypasses_g8` (in `tests/cli/account_mutations_test.rs`)

---

### FT-08: `trace::1` emits ownership gate diagnostic to stderr

- **Given:** `alice@corp.com.json` exists and is unowned. `trace::1` enabled.
- **When:** `clp .accounts owner::user1@w003 name::alice@corp.com trace::1`
- **Then:** Exits 0. Owner field written. stderr contains a diagnostic line referencing the G8 gate evaluation result.
- **Exit:** 0
- **Source fn:** `ft_owner_trace_emits_diagnostic` (in `tests/cli/account_mutations_test.rs`)

---

### FT-09: Short name prefix resolves to full email

- **Given:** `alice@corp.com.credentials.json` exists (unique prefix `alice`). Account is unowned.
- **When:** `clp .accounts owner::user1@w003 name::alice`
- **Then:** Exits 0. Prefix `"alice"` resolves to `alice@corp.com`. `alice@corp.com.json` contains `"owner": "user1@w003"`.
- **Exit:** 0
- **Source fn:** `ft_owner_prefix_resolution` (in `tests/cli/account_mutations_test.rs`)

---

### FT-10: Empty `owner::` value (not `0`) exits 1

- **Given:** Any environment with accounts in store.
- **When:** `clp .accounts owner:: name::alice@corp.com` (empty string value)
- **Then:** Exits 1. stderr contains error indicating empty value is invalid; directs user to use `owner::0` to clear ownership.
- **Exit:** 1
- **Source fn:** `ft_owner_empty_value_rejected` (in `tests/cli/account_mutations_test.rs`)

---

### FT-11: Subsequent G8-gated ops respect newly set `owner` field

- **Given:** `alice@corp.com.json` is unowned. Step 1: set owner. Step 2: different caller attempts same G8-gated op without force.
- **When-1:** `clp .accounts owner::user1@w003 name::alice@corp.com` (caller = user1@w003)
- **When-2:** `clp .accounts owner::other@remote name::alice@corp.com` (different caller, no force)
- **Then-1:** Exits 0. `alice@corp.com.json` owner set to `"user1@w003"`.
- **Then-2:** Exits 1. G8 enforces the newly set owner — `other@remote` is blocked.
- **Exit:** 0 (step 1), 1 (step 2)
- **Source fn:** `ft_owner_gates_respect_new_value` (in `tests/cli/account_mutations_test.rs`)

---

### FT-12: `.usage owner::` behaves identically to `.accounts owner::`

- **Given:** `alice@corp.com.json` exists and is unowned (reset between sub-cases).
- **When-A:** `clp .accounts owner::user1@w003 name::alice@corp.com`
- **When-B:** `clp .usage owner::user1@w003 name::alice@corp.com`
- **Then-A:** Exits 0. `alice@corp.com.json` contains `"owner": "user1@w003"`.
- **Then-B:** Exits 0. Identical result — `owner::` mutation works the same on `.usage` and `.accounts`.
- **Exit:** 0 both cases
- **Source fn:** `ft_owner_works_on_usage` (in `tests/cli/account_mutations_test.rs`)
