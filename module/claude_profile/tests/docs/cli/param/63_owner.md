# Parameter 062: `owner::` — Edge Cases

### Test Case Index

| ID | Test | Scenario | Expected | Status |
|----|------|----------|----------|--------|
| EC-01 | `ec1_owner_sets_custom_identity` | `owner::alice@laptop name::X` | writes `"owner": "alice@laptop"` to `{name}.json` | ✅ |
| EC-02 | `ec2_owner_empty_rejected` | `owner::` with empty value | exit 1 with "use owner::0 to clear" | ✅ |
| EC-03 | `ec3_owner_and_unclaim_removed_toggle` | `owner::user1@w003 unclaim::1 name::X` | exit 1 — `unclaim::1` is REMOVED_TOGGLE (Feature 064) | ✅ |
| EC-04 | `ec4_owner_missing_name_exits_1` | `owner::user1@w003` (no name::) | exit 1 with usage error | ✅ |
| EC-05 | `ec5_owner_g8_foreign_owner_blocked` | account owned by `other@host`, caller is not `other@host` | exit 1 ownership violation | ✅ |
| EC-06 | `ec6_owner_force_bypasses_g8` | same as EC-05 + `force::1` | write succeeds, exit 0 | ✅ |
| EC-07 | `ec7_owner_dry_no_file_writes` | `owner::user1@w003 name::X dry::1` | `[dry-run]` message, no `{name}.json` change | ✅ |
| EC-08 | `ec8_owner_overwrite_existing` | account already owned by caller → `owner::new@identity` | overwrites to new identity | ✅ |
| EC-09 | `ec9_owner_idempotent_same_value` | `owner::user1@w003` when already `owner: "user1@w003"` | no-op write, exit 0 | ✅ |
| EC-10 | `ec10_owner_zero_clears_ownership` | `owner::0 name::alice@corp.com` | writes `owner: ""` to `{name}.json`; exits 0 | ⏳ |
| EC-11 | `ec11_owner_zero_no_name_batch_clears` | `owner::0` alone (no `name::`) | clears ownership for all owned accounts in filter; non-owned skipped with "skip" message | ⏳ |
| EC-12 | `ec12_owner_zero_comma_list_batch_clear` | `owner::0 name::X,Y,Z` | clears ownership for X, Y, Z independently; G8 per account; exits 0 | ⏳ |
| EC-13 | `ec13_owner_set_comma_list_batch_set` | `owner::user1@w003 name::X,Y,Z` | sets owner for X, Y, Z independently; G8 per account; exits 0 | ⏳ |
| EC-14 | `ec14_owner_zero_force_bypasses_g8` | `owner::0 name::X force::1` when X owned by different identity | writes `owner: ""`; exits 0 | ⏳ |
| EC-15 | `ec15_owner_zero_dry_run` | `owner::0 name::X dry::1` | `[dry-run] would clear owner of X`; no file written; exits 0 | ⏳ |
| EC-16 | `ec16_owner_zero_force_dry_run` | `owner::0 name::X force::1 dry::1` | bypasses G8 + dry-run; `[dry-run]` message; no file written; exits 0 | ⏳ |

**Total:** 16 edge case tests

---

### EC-10: `owner::0 name::alice@corp.com` clears ownership

- **Given:** `alice@corp.com.json` exists with `"owner": "user1@w003"`. Caller identity is `user1@w003` (G8 passes).
- **When:** `clp .accounts owner::0 name::alice@corp.com`
- **Then:** Exits 0. `alice@corp.com.json` contains `"owner": ""`. stdout contains `unclaimed alice@corp.com`. Credentials file unchanged.
- **Exit:** 0
- **Source:** [param/062_owner.md](../../../../docs/cli/param/062_owner.md), [feature/064_active_marker_and_owner_redesign.md](../../../../docs/feature/064_active_marker_and_owner_redesign.md) AC-08

---

### EC-11: `owner::0` alone (no `name::`) — batch-clears all owned accounts

- **Given:** Credential store has accounts A (owned by caller), B (unowned), C (owned by caller). Current filter returns all three.
- **When:** `clp .accounts owner::0` (no `name::`)
- **Then:** Exits 0. A and C have `owner: ""` written. B is skipped with "skip" message (not owned). No credential files modified.
- **Exit:** 0
- **Source:** [param/062_owner.md](../../../../docs/cli/param/062_owner.md), [feature/064_active_marker_and_owner_redesign.md](../../../../docs/feature/064_active_marker_and_owner_redesign.md) AC-09

---

### EC-12: `owner::0 name::X,Y,Z` — batch-clear via comma-list

- **Given:** `alice@corp.com.json`, `bob@corp.com.json`, `charlie@corp.com.json` all exist. Caller owns all three.
- **When:** `clp .accounts owner::0 name::alice@corp.com,bob@corp.com,charlie@corp.com`
- **Then:** Exits 0. All three `.json` files contain `"owner": ""`. G8 evaluated independently for each. Stdout contains `unclaimed` for each.
- **Exit:** 0
- **Source:** [param/062_owner.md](../../../../docs/cli/param/062_owner.md), [feature/064_active_marker_and_owner_redesign.md](../../../../docs/feature/064_active_marker_and_owner_redesign.md) AC-10

---

### EC-13: `owner::user1@w003 name::X,Y,Z` — batch-set via comma-list

- **Given:** `alice@corp.com.json`, `bob@corp.com.json`, `charlie@corp.com.json` all exist. All unowned (G8 passes for each).
- **When:** `clp .accounts owner::user1@w003 name::alice@corp.com,bob@corp.com,charlie@corp.com`
- **Then:** Exits 0. All three `.json` files contain `"owner": "user1@w003"`. G8 evaluated independently for each. Stdout contains `owned {name} by user1@w003` for each.
- **Exit:** 0
- **Source:** [param/062_owner.md](../../../../docs/cli/param/062_owner.md), [feature/064_active_marker_and_owner_redesign.md](../../../../docs/feature/064_active_marker_and_owner_redesign.md) AC-11

---

### EC-14: `owner::0 name::X force::1` bypasses G8

- **Given:** `alice@corp.com.json` has `"owner": "other@remote"`. Caller identity is `user1@w003` (G8 would block without force).
- **When:** `clp .accounts owner::0 name::alice@corp.com force::1`
- **Then:** Exits 0. `alice@corp.com.json` contains `"owner": ""`. G8 bypassed by `force::1`. Stdout contains `unclaimed alice@corp.com`.
- **Exit:** 0
- **Source:** [param/062_owner.md](../../../../docs/cli/param/062_owner.md), [feature/064_active_marker_and_owner_redesign.md](../../../../docs/feature/064_active_marker_and_owner_redesign.md) AC-12

---

### EC-15: `owner::0 name::X dry::1` — dry-run preview

- **Given:** `alice@corp.com.json` exists with `"owner": "user1@w003"`. Caller identity is `user1@w003` (G8 passes).
- **When:** `clp .accounts owner::0 name::alice@corp.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would clear owner of alice@corp.com`. `alice@corp.com.json` is NOT modified — still contains `"owner": "user1@w003"`. No files written.
- **Exit:** 0
- **Source:** [param/062_owner.md](../../../../docs/cli/param/062_owner.md), [feature/064_active_marker_and_owner_redesign.md](../../../../docs/feature/064_active_marker_and_owner_redesign.md) AC-16

---

### EC-16: `owner::0 name::X force::1 dry::1` — bypass G8 and dry-run

- **Given:** `alice@corp.com.json` has `"owner": "other@remote"`. Caller identity is `user1@w003` (G8 would block without force).
- **When:** `clp .accounts owner::0 name::alice@corp.com force::1 dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would clear owner of alice@corp.com`. G8 bypassed by `force::1`. No files written — dry-run takes precedence over actual write.
- **Exit:** 0
- **Source:** [param/062_owner.md](../../../../docs/cli/param/062_owner.md), [feature/064_active_marker_and_owner_redesign.md](../../../../docs/feature/064_active_marker_and_owner_redesign.md) AC-17
