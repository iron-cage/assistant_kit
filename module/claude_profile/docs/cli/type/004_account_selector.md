# Type :: 4. `AccountSelector`

**Purpose:** Represents any form of account identification accepted at the CLI before resolution to a concrete [`AccountName`](001_account_name.md). The adapter layer accepts all three forms and resolves them to an `AccountName` before passing to core functions.

**Fundamental Type:** Logical type — handled by adapter resolution logic, not a concrete Rust struct. Resolution produces an `AccountName`.

**Forms:**

| Form | Example | Resolution |
|------|---------|------------|
| Full email (contains `@`) | `alice@acme.com` | Validated as email → `AccountName` |
| Local-part prefix (no `@`) | `car`, `alice` | Prefix-matched against saved accounts |
| Positional bare arg | `clp .account.use alice@home.com` | Same resolution as the two forms above |

**Resolution Algorithm:**

```
input contains '@'?
  yes → AccountName::new(input)  [validates email format]
  no  → prefix match: filter saved accounts where name.starts_with(input)
          sort alphabetically
          0 matches  → exit 1 "no account matching prefix 'X'"
          1 match    → AccountName::new(matched)
          2+ matches → exit 1 "ambiguous prefix 'X': matches alice@a.com, alice@b.com, ..."
```

**Constraints:**
- The resolved `AccountName` must satisfy all `AccountName` constraints (non-empty, valid email, path-safe)
- Prefix resolution is case-sensitive (matches use `str::starts_with`)
- Multiple prefix matches cause exit 1 — the user must be more specific

**Parameters:** [`name::`](../param/001_name.md) — the parameter that accepts `AccountSelector` input

**Commands:** [`.accounts`](../command/001_account.md#command--3-accounts) *(optional)*, [`.account.use`](../command/001_account.md#command--5-accountuse), [`.account.delete`](../command/001_account.md#command--6-accountdelete), [`.account.limits`](../command/001_account.md#command--11-accountlimits) *(optional)*

**Notes:**
- `.account.save` does NOT use prefix resolution — its `name::` value must be a full email (or omitted for auto-inference from `emailAddress` in `~/.claude.json`).
- `AccountSelector` is a documentation concept describing the adapter layer's resolution contract. The concrete Rust type that appears in function signatures after resolution is always `AccountName`.
