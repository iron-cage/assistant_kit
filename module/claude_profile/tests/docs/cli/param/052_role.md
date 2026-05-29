# Test: `role::` Parameter (Account Metadata Label)

Edge case coverage for the `role::` free-text metadata label parameter on `.account.save`. See [param/052_role.md](../../../../docs/cli/param/052_role.md) for specification.

Note: This is distinct from param 016 `role::` (boolean display toggle for `.credentials.status`).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `role::work` writes role label to profile.json | Behavioral Divergence |
| EC-2 | Omitting `role::` stores empty role in profile.json | Behavioral Divergence |
| EC-3 | `role::` (empty string) stores empty role (same as omitting) | Empty String Semantics |
| EC-4 | `role::work` appears in `clp .usage cols::+role` | Display |
| EC-5 | Re-save with different `role::` overwrites old value in profile.json | Update Semantics |
| EC-6 | `role::` value with spaces stored verbatim in profile.json | Special Characters |

---

### EC-1: `role::work` writes role label to `profile.json`

- **Given:** No pre-existing `test@example.com` account.
- **When:** `clp .account.save name::test@example.com role::work`
- **Then:** Exits 0. `{credential_store}/test@example.com.profile.json` contains `"role": "work"`.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/052_role.md](../../../../docs/cli/param/052_role.md)

---

### EC-2: Omitting `role::` stores empty role in profile.json

- **Given:** No pre-existing account.
- **When:** `clp .account.save name::test@example.com` (no `role::` param)
- **Then:** Exits 0. `profile.json` contains `"role": ""` (empty string, not absent).
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/052_role.md](../../../../docs/cli/param/052_role.md)

---

### EC-3: `role::` (empty string) stores empty role

- **Given:** No pre-existing account.
- **When:** `clp .account.save name::test@example.com role::` (empty value)
- **Then:** Exits 0. `profile.json` contains `"role": ""` — same as omitting `role::`.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/052_role.md](../../../../docs/cli/param/052_role.md)

---

### EC-4: `role::work` appears in `clp .usage cols::+role`

- **Given:** Account `test@example.com` saved with `role::work`.
- **When:** `clp .usage cols::+role`
- **Then:** Exits 0. Table row for `test@example.com` shows "work" in Role column.
- **Exit:** 0
- **Live:** yes
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/052_role.md](../../../../docs/cli/param/052_role.md)

---

### EC-5: Re-save with different `role::` overwrites old value in profile.json

- **Given:** Account `test@example.com` saved with `role::personal`.
- **When:** `clp .account.save name::test@example.com role::dev`
- **Then:** Exits 0. `profile.json` now contains `"role": "dev"`. Old value `"personal"` no longer present.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/052_role.md](../../../../docs/cli/param/052_role.md)

---

### EC-6: `role::` value with spaces stored verbatim in profile.json

- **Given:** No pre-existing account.
- **When:** `clp .account.save name::test@example.com role::dev ops team`
- **Then:** Exits 0. `profile.json` contains `"role": "dev ops team"` (value with spaces preserved verbatim).
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/052_role.md](../../../../docs/cli/param/052_role.md)
