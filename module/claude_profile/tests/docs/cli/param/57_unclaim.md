# Test: `unclaim::` Parameter — REMOVED

> **REMOVED**: The `unclaim::` parameter on `.account.save` has been removed (production param 056).
> The ownership clear operation is now a dedicated command: `.account.unclaim name::EMAIL`.
> See [command/18_account_unclaim.md](../command/18_account_unclaim.md) for current test coverage.
> See [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md) for the removal notice.

All EC test cases in this file (EC-1 through EC-5) are **invalid** — the `unclaim::` parameter
no longer exists on `.account.save`. The test functions (`ec1_*` through `ec5_*`) must be
removed or replaced during the implementation task that adds `.account.unclaim`.

### Superseded Test Case Index (DO NOT IMPLEMENT)

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| EC-1 | `unclaim::1` on `.account.save` writes `owner: ""` | Core Behavior | **REMOVED** |
| EC-2 | `unclaim::1` overwrites existing non-empty `owner` with `""` | Core Behavior | **REMOVED** |
| EC-3 | `unclaim::` absent — `.account.save` stamps `current_identity()` as owner | Default Behavior | **REMOVED** |
| EC-4 | `unclaim::1` preserves non-ownership `{name}.json` fields via read-merge | Field Isolation | **REMOVED** |
| EC-5 | `unclaim::1` with `dry::1` — no files written; ownership not cleared | Dry-Run Guard | **REMOVED** |

---

### EC-1: `unclaim::1` writes `owner: ""`

- **Given:** Account `alice` exists in the credential store. `alice.json` has `"owner": "user@host1"`. Valid `.credentials.json` present.
- **When:** `clp .account.save name::alice unclaim::1`
- **Then:** Exits 0. `alice.json` contains `"owner": ""`. All enforcement gates disabled for `alice`.
- **Exit:** 0
- **Source fn:** `ec1_unclaim_writes_empty_owner`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)

---

### EC-2: `unclaim::1` overwrites non-empty owner

- **Given:** `alice.json` contains `"owner": "alice@host1"`. Valid `.credentials.json` present.
- **When:** `clp .account.save name::alice unclaim::1` (from any machine, regardless of current identity)
- **Then:** Exits 0. `alice.json` now contains `"owner": ""`. Prior owner string is replaced.
- **Exit:** 0
- **Source fn:** `ec2_unclaim_overwrites_existing_owner`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)

---

### EC-3: `unclaim::` absent — `.account.save` stamps `current_identity()` as owner

- **Given:** Any account state. `current_identity()` resolves to `"testuser@testmachine"`. Valid `.credentials.json` present.
- **When:** `clp .account.save name::alice` (no `unclaim::` arg, or `unclaim::0`)
- **Then:** Exits 0. `alice.json` contains `"owner": "testuser@testmachine"`. Enforcement gates active for other identities.
- **Exit:** 0
- **Source fn:** `ec3_no_unclaim_stamps_current_identity`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)

---

### EC-4: `unclaim::1` preserves non-ownership `{name}.json` fields

- **Given:** `alice.json` contains `{"_renewal_at": "2026-06-29T21:00:00Z", "owner": "alice@host1"}`.
- **When:** `clp .account.save name::alice unclaim::1`
- **Then:** Exits 0. `alice.json` retains `_renewal_at` via read-merge in `save()`. `owner` changes to `""`.
- **Exit:** 0
- **Source fn:** `ec4_unclaim_preserves_other_fields`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)

---

### EC-5: `unclaim::1` with `dry::1` — no files written

- **Given:** `alice.json` contains `"owner": "alice@host1"`.
- **When:** `clp .account.save name::alice unclaim::1 dry::1`
- **Then:** Exits 0. Dry-run message printed. `alice.json` still contains `"owner": "alice@host1"` — unchanged. No credential or metadata files written.
- **Exit:** 0
- **Source fn:** `ec5_unclaim_dry_run_no_write`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)
