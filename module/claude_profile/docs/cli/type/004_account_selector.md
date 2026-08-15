# Type: 4. `AccountSelector`

**Purpose:** Represents any form of account identification accepted at the CLI before resolution to a concrete [`AccountName`](001_account_name.md). The adapter layer accepts all three forms and resolves them to an `AccountName` before passing to core functions.

**Fundamental Type:** Logical type — handled by adapter resolution logic, not a concrete Rust struct. Resolution produces a plain `String` (validated downstream by `validate_name()`/`validate_redirect_name()` — see [`AccountName`](001_account_name.md), which is likewise a validation contract on `&str`/`String`, not a concrete Rust type).

**Forms:**

| Form | Example | Resolution |
|------|---------|------------|
| Full email (contains `@`) | `alice@acme.com` | Validated as email → `AccountName` |
| Local-part prefix (no `@`) | `car`, `alice` | Prefix-matched against saved accounts |
| Positional bare arg | `clp .account.use alice@home.com` | Same resolution as the two forms above |

**Resolution Algorithm:**

```
input contains '@'?
  yes → returned as-is (email-format validation deferred to validate_name() downstream)
  no  → path-unsafe chars ('/', '\', '*')? → exit 1 "contains invalid characters"
        else prefix match:
          1. exact local-part match: filter saved accounts where local_part == input
             exactly 1 → resolve to that account (prevents i1 from matching i11, i12)
          2. prefix scan: filter saved accounts where name.starts_with(input)
          0 matches  → exit 2 "account 'X' not found"
          1 match    → resolve to that account
          2+ matches → exit 1 "ambiguous prefix 'X': matches alice@a.com, alice@b.com, ..."
```

(`resolve_account_name()`, `src/commands/cmd_args.rs`) — returns a plain `String`, not any wrapper type.

**Constraints:**
- The resolved `AccountName` must satisfy all `AccountName` constraints (non-empty, valid email, path-safe)
- Prefix resolution is case-sensitive (matches use `str::starts_with`)
- Multiple prefix matches cause exit 1 — the user must be more specific

**Notes:**
- `.account.save` does NOT use prefix resolution — its `name::` value must be a full email (or omitted for auto-inference from the per-machine `_active` marker in the credential store).
- `.account.renewal` additionally accepts `name::all` (targets all saved accounts) and `name::a@x.com,b@x.com` (comma-separated list). Prefix resolution applies to each individual token in the comma list; `all` is handled as a keyword and bypasses resolution.
- `AccountSelector` is a documentation concept describing the adapter layer's resolution contract; so is [`AccountName`](001_account_name.md) — `resolve_account_name()` (`src/commands/cmd_args.rs`) returns a plain `String` in function signatures after resolution, not a concrete `AccountName` Rust type.

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`name::`](../param/001_name.md) | Accepts any account selector form |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | Optional — account list display |
| 2 | [`.account.use`](../command/001_account.md#command-5-accountuse) | Activates selected account |
| 3 | [`.account.delete`](../command/001_account.md#command-6-accountdelete) | Removes selected account |
| 4 | [`.account.limits`](../command/001_account.md#command-11-accountlimits) | Optional — limits for selected account |
| 5 | [`.account.relogin`](../command/001_account.md#command-12-accountrelogin) | Refreshes credentials for selected account |
| 6 | [`.account.renewal`](../command/001_account.md#command-14-accountrenewal) | Schedules renewal for selected account |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Selector resolves active account for rotation |
| 2 | [Account Onboarding](../user_story/002_onboarding.md) | Selector used for delete and relogin flows |
