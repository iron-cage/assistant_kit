# Test: Feature 015 — Account Name Shortcut Syntax

### Scope

- **Purpose**: Test cases for account name shortcut syntax resolution.
- **Source**: `docs/feature/015_name_shortcut_syntax.md`
- **Covers**: AC-01 through AC-14

Feature behavioral requirement test cases for `docs/feature/015_name_shortcut_syntax.md`. Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `.account.use alice@home.com` positional — identical to `name::` form | AC-01 |
| FT-02 | `.account.delete alice@oldco.com` positional — identical to `name::` form | AC-02 |
| FT-03 | `.accounts alice@home.com` positional — shows one block | AC-03 |
| FT-04 | `.account.limits alice@acme.com` positional — shows limits | AC-04 |
| FT-05 | Prefix `car` resolves to `carol@example.com` | AC-05 |
| FT-06 | Ambiguous prefix exits 1 with match list | AC-06 |
| FT-07 | Unknown prefix exits 2 with not-found error | AC-07 |
| FT-08 | Explicit `name::EMAIL` form still works unchanged | AC-08 |
| FT-09 | Positional and `dry::` can be combined | AC-09 |
| FT-10 | Usage Examples section shows positional form | AC-10 |
| FT-11 | Exact local-part match wins over longer prefix matches | AC-11 |
| FT-12 | `.account.renewal name::alice` (single prefix) resolves and writes `_renewal_at` | AC-12 |
| FT-13 | `.account.renewal name::alice,bob` (comma-list) resolves each token independently | AC-13 |
| FT-14 | Reversed arg order: `key::value` before bare name | AC-14 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `.account.use` positional bare arg | AC-01 | Positional |
| FT-02 | `.account.delete` positional bare arg | AC-02 | Positional |
| FT-03 | `.accounts` positional bare arg | AC-03 | Positional |
| FT-04 | `.account.limits` positional bare arg | AC-04 | Positional |
| FT-05 | Prefix resolves to single matching account | AC-05 | Prefix |
| FT-06 | Ambiguous prefix → exit 1 with match list | AC-06 | Prefix |
| FT-07 | Unknown prefix → exit 2 not-found | AC-07 | Prefix |
| FT-08 | Explicit `name::EMAIL` still works on all commands | AC-08 | Backward Compat |
| FT-09 | Positional + `dry::1` combined | AC-09 | Combinations |
| FT-10 | Usage Examples shows positional form | AC-10 | Documentation |
| FT-11 | Exact local-part match over ambiguous prefix | AC-11 | Prefix |
| FT-12 | `.account.renewal` single prefix resolves | AC-12 | Renewal Prefix |
| FT-13 | `.account.renewal` comma-list each token resolved | AC-13 |
| FT-14 | Reversed arg order: `key::value` before bare name | AC-14 | Combinations |

**Total:** 14 FT cases

---

### FT-01: `.account.use` positional bare arg

- **Given:** `alice@home.com` is saved in the store.
- **When:** `clp .account.use alice@home.com` (no `name::` prefix)
- **Then:** Switches to `alice@home.com`. Identical result to `clp .account.use name::alice@home.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `aw13_use_positional_bare_arg`
- **Source:** [015_name_shortcut_syntax.md AC-01](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-02: `.account.delete` positional bare arg

- **Given:** `alice@oldco.com` is saved in the store.
- **When:** `clp .account.delete alice@oldco.com` (no `name::` prefix)
- **Then:** Deletes `alice@oldco.com`. Identical result to `clp .account.delete name::alice@oldco.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `ad13_delete_positional_bare_arg`
- **Source:** [015_name_shortcut_syntax.md AC-02](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-03: `.accounts` positional bare arg

- **Given:** `alice@home.com` is saved in the store.
- **When:** `clp .accounts alice@home.com` (no `name::` prefix)
- **Then:** Shows one block for `alice@home.com`. Exit 0.
- **Exit:** 0
- **Source fn:** `acc29_accounts_positional_bare_arg`
- **Source:** [015_name_shortcut_syntax.md AC-03](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-04: `.account.limits` positional bare arg

- **Given:** `alice@acme.com` is saved in the store.
- **When:** `clp .account.limits alice@acme.com`
- **Then:** Shows limits for `alice@acme.com`. Exits 0 or 2 depending on data availability; not exit 1 (no validation error).
- **Exit:** 0 or 2
- **Source fn:** `lim09_limits_positional_bare_arg`
- **Source:** [015_name_shortcut_syntax.md AC-04](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-05: Prefix resolves to single matching account

- **Given:** `carol@example.com` is the only saved account whose local part starts with `car`.
- **When:** `clp .account.use car`
- **Then:** Resolves to `carol@example.com`; switches successfully. Exit 0.
- **Exit:** 0
- **Source fn:** `aw14_use_prefix_resolves`, `acc30_accounts_prefix_resolves`, `lim10_limits_prefix_resolves`
- **Source:** [015_name_shortcut_syntax.md AC-05](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-06: Ambiguous prefix → exit 1 with match list

- **Given:** Both `alice@example.com` and `amy@example.com` are saved.
- **When:** `clp .account.use a`
- **Then:** Exits 1 with a message listing the ambiguous matches (e.g., `alice@example.com`, `amy@example.com`).
- **Exit:** 1
- **Source fn:** `aw15_use_prefix_ambiguous_exits_1`
- **Source:** [015_name_shortcut_syntax.md AC-06](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-07: Unknown prefix → exit 2 not-found

- **Given:** No account whose name starts with `ghost` exists.
- **When:** `clp .account.use ghost`
- **Then:** Exits 2 with `account not found: 'ghost'`.
- **Exit:** 2
- **Source fn:** `aw03_switch_nonexistent_exits_2`
- **Source:** [015_name_shortcut_syntax.md AC-07](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-08: Explicit `name::EMAIL` still works on all commands

- **Given:** `alice@home.com` is saved in the store.
- **When:** `clp .account.use name::alice@home.com` (explicit form)
- **Then:** Works identically to the positional form. Exit 0.
- **Exit:** 0
- **Source fn:** `aw01_switch_swaps_credentials`
- **Source:** [015_name_shortcut_syntax.md AC-08](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-09: Positional + `dry::1` combined

- **Given:** `alice@home.com` is saved in the store.
- **When:** `clp .account.use alice@home.com dry::1`
- **Then:** Dry-run output shows intent for `alice@home.com`. Both positional and `dry::` params are accepted together. Exit 0.
- **Exit:** 0
- **Source fn:** `aw02_switch_dry_run`
- **Source:** [015_name_shortcut_syntax.md AC-09](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-10: Usage Examples shows positional form

- **Given:** The CLI binary is invoked with `--help` on `.account.use`.
- **When:** `clp .account.use --help` (or equivalent)
- **Then:** The Examples section shows `clp .account.use alice@acme.com` (without `name::` prefix).
- **Exit:** 0
- **Source fn:** `aw35_help_shows_positional_example`
- **Source:** [015_name_shortcut_syntax.md AC-10](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-11: Exact local-part match wins over ambiguous prefix

- **Given:** Three accounts: `i1@wbox.pro`, `i11@wbox.pro`, `i12@wbox.pro`.
- **When:** `clp .account.use i1`
- **Then:** Resolves to `i1@wbox.pro` (exact local-part match). Does NOT exit 1 with ambiguous-prefix error. Exit 0.
- **Exit:** 0
- **Source fn:** `aw16_exact_local_part_wins_over_ambiguous_prefix`
- **Source:** [015_name_shortcut_syntax.md AC-11](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-12: `.account.renewal` single prefix resolves and writes `_renewal_at`

- **Given:** `alice@acme.com` is the only saved account whose local part is `alice`.
- **When:** `clp .account.renewal name::alice at::2026-07-01T00:00:00Z`
- **Then:** Resolves `alice` to `alice@acme.com`. Writes `_renewal_at` for that account. Exit 0.
- **Exit:** 0
- **Source fn:** `ft17_account_renewal_single_prefix_resolves`
- **Source:** [015_name_shortcut_syntax.md AC-12](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-13: `.account.renewal` comma-list resolves each token independently

- **Given:** Both `alice@acme.com` and `bob@acme.com` are saved; their local parts are unique.
- **When:** `clp .account.renewal name::alice,bob at::2026-07-01T00:00:00Z`
- **Then:** Both tokens resolve via prefix: `alice` → `alice@acme.com`, `bob` → `bob@acme.com`. Both accounts get `_renewal_at` written. Exit 0.
- **Exit:** 0
- **Source fn:** `ft18_account_renewal_comma_list_prefix_tokens`
- **Source:** [015_name_shortcut_syntax.md AC-13](../../../docs/feature/015_name_shortcut_syntax.md)

---

### FT-14: Reversed arg order: `key::value` before bare name

- **Given:** `alice@home.com` is saved in the store.
- **When:** `clp .account.use dry::1 alice@home.com` (key::value param before bare positional name)
- **Then:** Exits 0; dry-run output shows intent for `alice@home.com`. Identical result to `clp .account.use alice@home.com dry::1`. Argument order does not affect positional rewrite.
- **Exit:** 0
- **Source fn:** `aw36_positional_after_key_value`, `ad16_delete_positional_after_key_value`, `ar10_relogin_positional_after_key_value`, `acc51_accounts_positional_after_key_value`, `lim11_limits_positional_after_key_value`
- **Source:** [015_name_shortcut_syntax.md AC-14](../../../docs/feature/015_name_shortcut_syntax.md)
