# Test: `host::` Parameter

Edge case coverage for the `host::` parameter on `.account.save`. See [param/048_host.md](../../../../docs/cli/param/048_host.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `host::mybox` writes host label to `{name}.json` | Behavioral Divergence |
| EC-2 | Omitting `host::` auto-captures `$USER@$HOSTNAME` | Behavioral Divergence |
| EC-3 | `host::` (empty string) auto-captures `$USER@$HOSTNAME` | Empty Triggers Auto |
| EC-4 | `host::mybox` appears in `clp .usage cols::+host` | Display |
| EC-5 | Re-save with different `host::` overwrites old value in `{name}.json` | Update Semantics |
| EC-6 | `host::` value with spaces stored verbatim in `{name}.json` | Special Characters |

---

### EC-1: `host::mybox` writes host label to `{name}.json`

- **Given:** No pre-existing `test@example.com` account.
- **When:** `clp .account.save name::test@example.com host::mybox`
- **Then:** Exits 0. `{credential_store}/test@example.com.json` contains `"host": "mybox"`.
- **Exit:** 0
- **Source fn:** `as_save_writes_profile_json` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)

---

### EC-2: Omitting `host::` auto-captures `$USER@$HOSTNAME`

- **Given:** `$USER=alice`, `$HOSTNAME=myworkstation` in environment.
- **When:** `clp .account.save name::test@example.com` (no `host::`)
- **Then:** Exits 0. `{name}.json` contains `"host": "alice@myworkstation"`.
- **Exit:** 0
- **Source fn:** `as24_host_auto_capture_user_hostname` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)

---

### EC-3: `host::` empty string triggers auto-capture

- **Given:** `$USER=bob`, `$HOSTNAME=laptop` in environment.
- **When:** `clp .account.save name::test@example.com host::` (empty value)
- **Then:** Exits 0. `{name}.json` contains `"host": "bob@laptop"` (empty value triggers auto-capture, same as omitting `host::`).
- **Exit:** 0
- **Source fn:** `as25_host_empty_triggers_auto_capture` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)

---

### EC-4: `host::mybox` appears in `clp .usage cols::+host`

- **Given:** Account `test@example.com` saved with `host::mybox`.
- **When:** `clp .usage cols::+host`
- **Then:** Exits 0. Table row for `test@example.com` shows "mybox" in Host column.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it202_cols_host_shows_host_column` (in `tests/cli/usage_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)

---

### EC-5: Re-save with different `host::` overwrites old value in `{name}.json`

- **Given:** Account `test@example.com` saved with `host::oldbox`.
- **When:** `clp .account.save name::test@example.com host::newbox`
- **Then:** Exits 0. `{name}.json` now contains `"host": "newbox"`. Old value `"oldbox"` no longer present.
- **Exit:** 0
- **Source fn:** `as26_host_resave_overwrites` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)

---

### EC-6: `host::` value with spaces stored verbatim in `{name}.json`

- **Given:** No pre-existing account.
- **When:** `clp .account.save name::test@example.com host::my work laptop`
- **Then:** Exits 0. `{name}.json` contains `"host": "my work laptop"` (value with spaces preserved verbatim).
- **Exit:** 0
- **Source fn:** `as27_host_with_spaces` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)
