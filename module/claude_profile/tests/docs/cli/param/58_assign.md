# Test: `assign::` Parameter — REMOVED (Feature 064)

> **REMOVED (Feature 064)**: The `assign::` parameter on `.accounts` and `.usage` has been removed.
> Marker assignment was moved to `active::USER@MACHINE name::X` (Feature 064), which was then renamed to `assignee::USER@MACHINE name::X` (Feature 065).
>
> Any invocation of `assign::` exits 1 with the migration message:
> "REMOVED — use `assignee::USER@MACHINE name::X` instead"
>
> See [param/057_assign.md](../../../../docs/cli/param/057_assign.md) for the removal notice.
> See [feature/065_assignee_param_redesign.md](../../../../docs/feature/065_assignee_param_redesign.md) for the current interface.

All EC test cases in this file (EC-1 through EC-8) are **superseded** — `assign::` no longer exists as an active
parameter. The equivalent behaviors are now covered by `64_assignee.md` EC-1 through EC-11 (the `assignee::USER@MACHINE`
param handles assign, unassign, dry-run, validation, sanitization, and isolation).

### Superseded Test Case Index (DO NOT IMPLEMENT)

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| EC-1 | `assign::1 name::X` writes marker for current machine | Behavioral | **REMOVED** |
| EC-2 | `assign::1 name::X for::U@M` writes marker for specified machine | Behavioral | **REMOVED** |
| EC-3 | `assign::1` (no `name::`) emits live usage block; no marker written | Behavioral Divergence | **REMOVED** |
| EC-4 | `assign::0` (default) — no marker written | Default | **REMOVED** |
| EC-5 | `assign::1 name::unknown` exits 1 — account not in credential store | Validation | **REMOVED** |
| EC-6 | `assign::1 dry::1 name::X` previews without writing | Dry-run | **REMOVED** |
| EC-7 | `force::1 assign::1 name::X` writes normally; `force::` silently ignored | Interaction | **REMOVED** |
| EC-8 | `assign::1 name::` (empty string name) exits 1 | Validation | **REMOVED** |

---

### EC-1: `assign::1 name::X` writes marker for current machine *(SUPERSEDED)*

- **Given:** `alice@corp.com.credentials.json` exists in credential store. No existing marker for current machine.
- **When:** `clp .accounts assign::1 name::alice@corp.com`
- **Then:** Exits 0. `{credential_store}/_active_{machine}_{user}` contains `alice@corp.com`. No other files modified (credentials, `{name}.json`, `~/.claude.json` all unchanged).
- **Exit:** 0
- **Source fn:** `as01_current_machine_marker_written`
- **Source:** [param/057_assign.md](../../../../docs/cli/param/057_assign.md)

---

### EC-2: `assign::1 name::X for::U@M` writes marker for specified machine *(SUPERSEDED)*

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts assign::1 name::alice@corp.com for::bob@laptop`
- **Then:** Exits 0. `{credential_store}/_active_laptop_bob` contains `alice@corp.com`. Current machine's marker unchanged. No credential files modified.
- **Exit:** 0
- **Source fn:** `as02_remote_machine_marker_written`
- **Source:** [param/057_assign.md](../../../../docs/cli/param/057_assign.md)

---

### EC-3: `assign::1` (no `name::`) emits live usage block; no marker written *(SUPERSEDED)*

- **Given:** Current machine identity resolves to `testuser@testmachine`. Active account is `alice@corp.com`.
- **When:** `clp .accounts assign::1` (no `name::`)
- **Then:** Exits 0. stdout contains current machine identity, active account name, and copy-paste examples (`clp .accounts assign::1 name::alice@corp.com`, etc.). No `_active_{machine}_{user}` file is created or modified.
- **Exit:** 0
- **Source fn:** `as03_no_name_emits_usage_block`
- **Source:** [param/057_assign.md](../../../../docs/cli/param/057_assign.md)

---

### EC-4: `assign::0` (default) — no marker written *(SUPERSEDED)*

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts assign::0 name::alice@corp.com` (or `.accounts name::alice@corp.com` with no `assign::`)
- **Then:** Exits 0. No `_active_*` marker file is created. Normal `.accounts` listing behavior.
- **Exit:** 0
- **Source fn:** `as04_assign_zero_no_write`
- **Source:** [param/057_assign.md](../../../../docs/cli/param/057_assign.md)

---

### EC-5: `assign::1 name::unknown` exits 1 — account not in credential store *(SUPERSEDED)*

- **Given:** Credential store does NOT contain `missing@corp.com`.
- **When:** `clp .accounts assign::1 name::missing@corp.com`
- **Then:** Exits 1. Error message indicates account not found. No `_active_*` file created.
- **Exit:** 1
- **Source fn:** `as05_unknown_account_exits_1`
- **Source:** [param/057_assign.md](../../../../docs/cli/param/057_assign.md)

---

### EC-6: `assign::1 dry::1 name::X` previews without writing *(SUPERSEDED)*

- **Given:** `alice@corp.com.credentials.json` exists in credential store. Note any existing marker mtime.
- **When:** `clp .accounts assign::1 dry::1 name::alice@corp.com`
- **Then:** Exits 0. stdout contains `[dry-run]` preview message describing marker write target. No `_active_*` marker file is created or modified. Marker mtime unchanged.
- **Exit:** 0
- **Source fn:** `as06_dry_run_no_write`
- **Source:** [param/057_assign.md](../../../../docs/cli/param/057_assign.md)

---

### EC-7: `force::1 assign::1 name::X` writes normally; `force::` silently ignored *(SUPERSEDED)*

- **Given:** `alice@corp.com.credentials.json` exists in credential store. `alice@corp.com.json` has `"owner": "other@remote"` (owned by different identity).
- **When:** `clp .accounts force::1 assign::1 name::alice@corp.com`
- **Then:** Exits 0. Marker `_active_{machine}_{user}` written with `alice@corp.com`. `force::1` has no effect — assign has no ownership gate. No error related to force.
- **Exit:** 0
- **Note:** Assign is ownership-neutral: does not read or modify the `owner` field; `force::` has nothing to bypass.
- **Source fn:** `as07_force_ignored_on_assign`
- **Source:** [param/057_assign.md](../../../../docs/cli/param/057_assign.md)

---

### EC-8: `assign::1 name::` (empty string) exits 1 *(SUPERSEDED)*

- **Given:** `alice@corp.com.credentials.json` exists in credential store.
- **When:** `clp .accounts assign::1 name::`  (empty name value, distinct from absent `name::`)
- **Then:** Exits 1. Error message indicates name value cannot be empty. No `_active_*` marker file written. Distinct from the no-`name::` case (EC-3), which emits a live usage block.
- **Exit:** 1
- **Source fn:** `as08_empty_name_value_exits_1`
- **Source:** [param/057_assign.md](../../../../docs/cli/param/057_assign.md)
