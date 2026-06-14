# Test: `unclaim::` Parameter

Edge case coverage for the `unclaim::` parameter on `.account.save`. See [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `unclaim::1` on a freshly saved account writes `owner: ""` | Core Behavior |
| EC-2 | `unclaim::1` overwrites existing non-empty `owner` with `""` | Core Behavior |
| EC-3 | `unclaim::0` (or absent) — normal save writes `owner: current_identity()` | Default Behavior |
| EC-4 | `unclaim::1` preserves all other `{name}.json` fields via read-merge | Field Isolation |
| EC-5 | `unclaim::1` with `dry::1` — no files written; ownership not cleared | Dry-Run Guard |

---

### EC-1: `unclaim::1` writes `owner: ""`

- **Given:** Account `alice` with no pre-existing `alice.json` (or existing file, any state).
- **When:** `clp .account.save name::alice unclaim::1`
- **Then:** Exits 0. `alice.json` contains `"owner": ""`. All enforcement gates disabled for `alice`.
- **Exit:** 0
- **Source fn:** `ec1_unclaim_writes_empty_owner`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)

---

### EC-2: `unclaim::1` overwrites non-empty owner

- **Given:** `alice.json` contains `"owner": "alice@host1"`.
- **When:** `clp .account.save name::alice unclaim::1` (from any machine, regardless of current identity)
- **Then:** Exits 0. `alice.json` now contains `"owner": ""`. Prior owner string is replaced.
- **Exit:** 0
- **Source fn:** `ec2_unclaim_overwrites_existing_owner`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)

---

### EC-3: `unclaim::0` (default) writes `current_identity()` as owner

- **Given:** Any account state. `current_identity()` resolves to `"user@host1"`.
- **When:** `clp .account.save name::alice` (no `unclaim::` arg, or `unclaim::0`)
- **Then:** Exits 0. `alice.json` contains `"owner": "user@host1"`. Enforcement gates active from other machines.
- **Exit:** 0
- **Source fn:** `ec3_default_sets_owner_to_current_identity`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)

---

### EC-4: `unclaim::1` preserves other `{name}.json` fields

- **Given:** `alice.json` contains `{"expires_at_ms": 12345, "host": "workstation", "role": "work", "owner": "alice@host1"}`.
- **When:** `clp .account.save name::alice unclaim::1`
- **Then:** Exits 0. `alice.json` retains `expires_at_ms: 12345`, `host: "workstation"`, `role: "work"`. Only `owner` changes to `""`.
- **Exit:** 0
- **Source fn:** `ec4_unclaim_preserves_other_fields`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)

---

### EC-5: `unclaim::1` with `dry::1` — no files written

- **Given:** `alice.json` contains `"owner": "alice@host1"`.
- **When:** `clp .account.save name::alice unclaim::1 dry::1`
- **Then:** Exits 0. Dry-run message printed. `alice.json` still contains `"owner": "alice@host1"` — unchanged.
- **Exit:** 0
- **Source fn:** `ec5_unclaim_dry_run_no_write`
- **Source:** [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md)
