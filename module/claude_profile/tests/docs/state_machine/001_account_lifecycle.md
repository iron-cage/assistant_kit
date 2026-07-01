# State Machine 001: Account Lifecycle

AC test cases for `docs/state_machine/001_account_lifecycle.md`. Tests the
`absent/saved/active` lifecycle states driven by `.account.save`, `.account.use`,
and `.account.delete` commands.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | `absent → saved` via account.save | Transition | ✅ |
| AC-2 | `saved → saved` via re-save (credential snapshot updated) | Transition | ✅ |
| AC-3 | `saved → active` via account.use | Transition | ✅ |
| AC-4 | `active → saved` when another account becomes active | Transition | ✅ |
| AC-5 | `saved → absent` via account.delete | Transition | ✅ |
| AC-6 | `absent → absent` — delete on non-existent is no-op (exits 2) | Boundary | ✅ |
| AC-7 | Full lifecycle roundtrip: absent→saved→active→absent | Roundtrip | ✅ |

---

### AC-1: `absent → saved` via account.save

- **Given:** No credential file exists for `test-account` in the credential store.
- **When:** `.account.save name::test-account` is executed.
- **Then:** `{name}.credentials.json` is created in the credential store. Account transitions
  from `absent` to `saved`. Active marker is NOT written (no `_active_{host}_{user}` marker).
- **Source fn:** `save_bv2_transitions_absent_to_saved` in
  `tests/cli/command_verb_test.rs`
- **Source:** [state_machine/001_account_lifecycle.md](../../../docs/state_machine/001_account_lifecycle.md)

---

### AC-2: `saved → saved` via re-save (credential snapshot updated)

- **Given:** `test-account` is already in `saved` state (credential file exists, no active marker).
- **When:** `.account.save name::test-account` is executed again with the same account.
- **Then:** Credential file is overwritten (snapshot updated). State remains `saved`. No lifecycle
  change occurs. Re-saving is idempotent with respect to state — only the credential snapshot
  content is updated.
- **Source fn:** `save_bv1_resave_same_credentials_idempotent` in
  `tests/cli/command_verb_test.rs`
- **Source:** [state_machine/001_account_lifecycle.md](../../../docs/state_machine/001_account_lifecycle.md)

---

### AC-3: `saved → active` via account.use

- **Given:** `test-account` is in `saved` state.
- **When:** `.account.use name::test-account` is executed.
- **Then:** `{name}.credentials.json` contents are written to `~/.claude/.credentials.json`.
  Active marker `_active_{host}_{user}` is written with value `test-account`. Account transitions
  from `saved` to `active`.
- **Source fn:** `use_bv2_transitions_saved_to_active` in
  `tests/cli/command_verb_test.rs`
- **Source:** [state_machine/001_account_lifecycle.md](../../../docs/state_machine/001_account_lifecycle.md)

---

### AC-4: `active → saved` when another account becomes active

- **Given:** `account-A` is the `active` account (marker = `account-A`). `account-B` is `saved`.
- **When:** `.account.use name::account-B` is executed.
- **Then:** Active marker is overwritten to `account-B`. `account-A` transitions from `active` to
  `saved` (its credential file persists; it no longer holds the active marker). `account-B`
  transitions from `saved` to `active`.
- **Source fn:** `account_nc1_full_lifecycle_roundtrip` in
  `tests/cli/command_noun_test.rs` (step 3 of the roundtrip: alice active → saved when bob uses
  `.account.use`; asserted via `alice_cred.exists()` at line 86. Shared with AC-7 by design —
  the displacement behavior is inherently part of the roundtrip; no independent test needed since
  step 3 provides an explicit targeted assertion.)
- **Source:** [state_machine/001_account_lifecycle.md](../../../docs/state_machine/001_account_lifecycle.md)

---

### AC-5: `saved → absent` via account.delete

- **Given:** `test-account` is in `saved` state (credential file exists, NOT the active account).
- **When:** `.account.delete name::test-account` is executed.
- **Then:** `{name}.credentials.json` is removed from the credential store. Account transitions
  from `saved` to `absent`. Active marker is unaffected.
- **Source fn:** `delete_bv2_transitions_saved_to_absent` in
  `tests/cli/command_verb_test.rs`
- **Source:** [state_machine/001_account_lifecycle.md](../../../docs/state_machine/001_account_lifecycle.md)

---

### AC-6: `absent → absent` — delete on non-existent account is no-op (exits 2)

- **Given:** No credential file exists for `ghost-account`.
- **When:** `.account.delete name::ghost-account` is executed.
- **Then:** Command exits 2 (not-found). No file changes occur. State remains `absent` (no-op).
- **Source fn:** `ad04_delete_nonexistent_exits_2` in
  `tests/cli/account_mutations_test.rs`
- **Source:** [state_machine/001_account_lifecycle.md](../../../docs/state_machine/001_account_lifecycle.md)

---

### AC-7: Full lifecycle roundtrip — absent→saved→active→absent

- **Given:** Empty credential store.
- **When:** The full sequence executes: save → use → save-another → use-another → delete-first.
- **Then:** Account transitions through all states correctly. After deletion, the credential store
  is in a consistent state with no orphaned markers. The account that was demoted from active to
  saved is still deletable.
- **Source fn:** `account_nc1_full_lifecycle_roundtrip` in
  `tests/cli/command_noun_test.rs`
- **Source:** [state_machine/001_account_lifecycle.md](../../../docs/state_machine/001_account_lifecycle.md)
