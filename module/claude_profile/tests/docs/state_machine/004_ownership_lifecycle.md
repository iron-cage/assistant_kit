# State Machine 004: Ownership Lifecycle

AC test cases for `docs/state_machine/004_ownership_lifecycle.md`. Tests the
`unclaimed/owned_here/owned_elsewhere` state transitions driven by `owner::` parameter,
`is_owned` computation, and gate enforcement (G2/G4).

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | `unclaimed` (owner="") acts as `owned_here` — passes use gate | State | ✅ |
| AC-2 | `save` is ownership-neutral — never writes `owner` field | Invariant | ✅ |
| AC-3 | `unclaimed → owned_here` via `owner::X` (current identity) | Transition | ✅ |
| AC-4 | `unclaimed → owned_here` — G8 allows write when account is unclaimed | Transition | ✅ |
| AC-5 | `owned_here → unclaimed` via `owner::0` (clears owner field) | Transition | ✅ |
| AC-6 | G4 gate — touch skips `owned_elsewhere` account | Gate | ✅ |
| AC-7 | G4 gate — touch skips `occupied_elsewhere` account | Gate | ✅ |
| AC-8 | G2 gate — refresh skips `owned_elsewhere` account | Gate | ✅ |

---

### AC-1: `unclaimed` (owner="") acts as `owned_here` — passes use gate

- **Given:** An account with an empty `owner` field in `{name}.json`. This is the `unclaimed`
  state. `is_owned = (owner == "" || owner == current_identity())`.
- **When:** An operation that checks `is_owned` (e.g., `.account.use` or a rotation gate) runs.
- **Then:** The account passes the ownership gate. `is_owned = true` when `owner` is empty.
  Unclaimed accounts are treated as available to any machine.
- **Source fn:** `cc9_unclaimed_account_passes_use_gate` in
  `tests/cli/account_ownership_test.rs`
- **Source:** [state_machine/004_ownership_lifecycle.md](../../../docs/state_machine/004_ownership_lifecycle.md)

---

### AC-2: `save` is ownership-neutral — never writes `owner` field

- **Given:** An account with `owner` field set to a specific identity (e.g., `user@machine`).
- **When:** `.account.save name::X` is executed for this account (re-save or first save).
- **Then:** The `owner` field in `{name}.json` is NOT modified. `account.save` always passes
  `owner: None` to `save()` — it is intentionally neutral on ownership. Ownership must be
  explicitly set via `.accounts owner::`. Re-saving never accidentally unclaims or re-claims.
- **Source fn:** `ft01_save_does_not_stamp_owner` in
  `tests/cli/account_ownership_test.rs`
- **Source:** [state_machine/004_ownership_lifecycle.md](../../../docs/state_machine/004_ownership_lifecycle.md)

---

### AC-3: `unclaimed → owned_here` via `owner::X` (current identity)

- **Given:** An account in `unclaimed` state (owner="" or absent).
- **When:** `.accounts owner::user@machine name::X` is executed where `user@machine` is the
  current machine's `current_identity()`.
- **Then:** The `owner` field in `{name}.json` is written with `user@machine`. Account
  transitions to `owned_here`. Subsequent `is_owned` checks return `true`.
- **Source fn:** `ft_owner_sets_owner_field` in
  `tests/cli/account_owner_param_test.rs`
- **Source:** [state_machine/004_ownership_lifecycle.md](../../../docs/state_machine/004_ownership_lifecycle.md)

---

### AC-4: `unclaimed → owned_here` — G8 allows ownership write when target is unclaimed

- **Given:** An account in `unclaimed` state (`owner=""`, `is_owned=true`).
- **When:** `.accounts owner::X name::account` is executed.
- **Then:** The write succeeds without `force::1`. G8 evaluates `!gate_ownership || aq.is_owned`
  — unclaimed accounts have `is_owned=true`, so G8 passes. No force flag needed to claim an
  unclaimed account.
- **Source fn:** `ft_owner_unowned_passes_g8` in
  `tests/cli/account_owner_param_test.rs`
- **Source:** [state_machine/004_ownership_lifecycle.md](../../../docs/state_machine/004_ownership_lifecycle.md)

---

### AC-5: `owned_here → unclaimed` via `owner::0` (clears owner field)

- **Given:** An account in `owned_here` state (`owner = current_identity()`).
- **When:** `.accounts owner::0 name::X` is executed (unclaim).
- **Then:** The `owner` field is cleared to `""`. Account transitions to `unclaimed`. Subsequent
  `is_owned` checks return `true` (unclaimed = available). The owner field is removed/emptied,
  not deleted from the JSON schema.
- **Source fn:** `ft02_unclaim_clears_owner` in
  `tests/cli/account_ownership_test.rs`
- **Source:** [state_machine/004_ownership_lifecycle.md](../../../docs/state_machine/004_ownership_lifecycle.md)

---

### AC-6: G4 gate — touch skips `owned_elsewhere` account (`is_owned=false`)

- **Given:** An account where `is_owned=false` (owner field matches a different machine's
  identity). `is_occupied_elsewhere=false`.
- **When:** `apply_touch()` evaluates the G4 ownership gate.
- **Then:** Touch is skipped. `reason: not owned` is emitted in trace mode. Touching credentials
  on a foreign-owned account would attempt to modify tokens that belong to another machine's
  active session.
- **Source fn:** `ft07_touch_skips_non_owned_with_trace` in
  `tests/usage/touch_tests.rs`
- **Source:** [state_machine/004_ownership_lifecycle.md](../../../docs/state_machine/004_ownership_lifecycle.md)

---

### AC-7: G4 gate — touch skips `occupied_elsewhere` account

- **Given:** An account where `is_occupied_elsewhere=true` (account is actively in use on
  another machine, i.e., it is the active account on a different host).
- **When:** `apply_touch()` evaluates the G4 gate.
- **Then:** Touch is skipped. `reason: occupied elsewhere` is emitted in trace mode. The G4
  gate checks BOTH `!is_owned` AND `is_occupied_elsewhere` — both conditions independently
  block touch regardless of the other.
- **Source fn:** `ft_touch_skips_occupied_elsewhere_with_trace` in
  `tests/usage/touch_tests.rs`
- **Source:** [state_machine/004_ownership_lifecycle.md](../../../docs/state_machine/004_ownership_lifecycle.md)

---

### AC-8: G2 gate — refresh skips `owned_elsewhere` account

- **Given:** An account where `is_owned=false` (owned by another machine).
- **When:** `apply_refresh()` evaluates the G2 refresh predicate.
- **Then:** Refresh is skipped. `reason: not_owned` trace is emitted. The G2 gate prevents
  `refresh_account_token()` from being called on foreign-owned accounts — refreshing their
  tokens would immediately invalidate the other machine's active session. Fix BUG-295.
- **Source fn:** `mre_bug295_apply_refresh_trace_reason_not_owned` in
  `tests/usage/refresh_tests_b.rs`
- **Source:** [state_machine/004_ownership_lifecycle.md](../../../docs/state_machine/004_ownership_lifecycle.md)
