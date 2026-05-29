# Test: `host::` Parameter

Edge case coverage for the `host::` parameter on `.account.save`. See [param/048_host.md](../../../../docs/cli/param/048_host.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `host::mybox` writes host label to profile.json | Profile Write |
| EC-2 | Omitting `host::` auto-captures `$USER@$HOSTNAME` | Auto-Capture |
| EC-3 | `host::` (empty string) auto-captures `$USER@$HOSTNAME` | Empty Triggers Auto |
| EC-4 | `host::mybox` appears in `clp .usage cols::+host` | Display |

---

### EC-1: `host::mybox` writes host label to `profile.json`

- **Given:** No pre-existing `test@example.com` account.
- **When:** `clp .account.save name::test@example.com host::mybox`
- **Then:** Exits 0. `{credential_store}/test@example.com.profile.json` contains `"host": "mybox"`.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)

---

### EC-2: Omitting `host::` auto-captures `$USER@$HOSTNAME`

- **Given:** `$USER=alice`, `$HOSTNAME=myworkstation` in environment.
- **When:** `clp .account.save name::test@example.com` (no `host::`)
- **Then:** Exits 0. `profile.json` contains `"host": "alice@myworkstation"`.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)

---

### EC-3: `host::` empty string triggers auto-capture

- **Given:** `$USER=bob`, `$HOSTNAME=laptop` in environment.
- **When:** `clp .account.save name::test@example.com host::` (empty value)
- **Then:** Exits 0. `profile.json` contains `"host": "bob@laptop"` (empty value triggers auto-capture, same as omitting `host::`).
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)

---

### EC-4: `host::mybox` appears in `clp .usage cols::+host`

- **Given:** Account `test@example.com` saved with `host::mybox`.
- **When:** `clp .usage cols::+host`
- **Then:** Exits 0. Table row for `test@example.com` shows "mybox" in Host column.
- **Exit:** 0
- **Live:** yes
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/048_host.md](../../../../docs/cli/param/048_host.md)
