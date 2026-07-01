# Command :: 14. `.account.renewal` — Integration Tests

Set or clear a billing renewal timestamp override (`_renewal_at`) stored in `{name}.json`.

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | `at::` writes `_renewal_at` to `{name}.json` | Account exists; `at::2026-06-29T21:00:00Z`; verifies file contains field | 0 |
| IT-2 | `from_now::+1h` writes future `_renewal_at` | Account exists; `from_now::+1h`; verifies ISO-8601 string is present | 0 |
| IT-3 | `clear::1` removes `_renewal_at` from `{name}.json` | Account has `_renewal_at`; after `clear::1`, field absent | 0 |
| IT-4 | `at::` and `from_now::` together exits 1 | Both params provided; error names the conflict | 1 |
| IT-5 | `at::` and `clear::` together exits 1 | Both params provided; error names the conflict | 1 |
| IT-6 | `from_now::` and `clear::` together exits 1 | Both params provided; error names the conflict | 1 |
| IT-7 | No operation param provided exits 1 | `name::` only, no `at::`/`from_now::`/`clear::` | 1 |
| IT-8 | `dry::1` prints would-be value without writing | `at::2026-06-29T21:00:00Z dry::1`; file unchanged after command | 0 |
| IT-9 | `name::all` updates every saved account | Two accounts; `from_now::+0m`; both `.json` updated | 0 |
| IT-10 | `name::` with unknown account exits 2 | Account not in credential store | 2 |
| IT-11 | `name::alice,bob` comma-list updates both | Two accounts; `at::2026-06-29T21:00:00Z`; both updated | 0 |
| IT-12 | Existing `oauthAccount` content preserved on write | Account with full `oauthAccount`; after `at::`; `oauthAccount` unchanged | 0 |
| IT-13 | Past `at::` value accepted (auto-advanced by `.usage`) | `at::2020-01-01T00:00:00Z`; exits 0; field written verbatim | 0 |
| IT-14 | Unknown parameter rejected → exit 1 | `clp .account.renewal unknown::x` | 1 |
| IT-15 | Single prefix resolves to full email | Account `alice@acme.com` saved; `name::alice at::2026-07-01T00:00:00Z`; `_renewal_at` written to `alice@acme.com.json` | 0 |
| IT-16 | Comma-list with prefix tokens resolves each | Accounts `alice@acme.com` and `bob@acme.com` saved; `name::alice,bob at::2026-07-01T00:00:00Z`; both `.json` files updated | 0 |

---

### IT-1: `at::` writes `_renewal_at` to `{name}.json`

- **Given:** Account `alice@acme.com` exists in credential store
- **When:** `clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z`
- **Then:** Exits 0. `alice@acme.com.json` contains `"_renewal_at": "2026-06-29T21:00:00Z"`.
- **Exit:** 0
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md), [param/049_at.md](../../../../docs/cli/param/049_at.md)

---

### IT-2: `from_now::+1h` writes future `_renewal_at`

- **Given:** Account `alice@acme.com` exists in credential store
- **When:** `clp .account.renewal name::alice@acme.com from_now::+1h`
- **Then:** Exits 0. `alice@acme.com.json` contains a `_renewal_at` field with a valid ISO-8601 timestamp string approximately 1 hour in the future.
- **Exit:** 0
- **Source:** [param/050_from_now.md](../../../../docs/cli/param/050_from_now.md)

---

### IT-3: `clear::1` removes `_renewal_at` from `{name}.json`

- **Given:** Account `alice@acme.com` exists with `_renewal_at` set
- **When:** `clp .account.renewal name::alice@acme.com clear::1`
- **Then:** Exits 0. `alice@acme.com.json` does NOT contain the `_renewal_at` field.
- **Exit:** 0
- **Source:** [param/051_clear.md](../../../../docs/cli/param/051_clear.md)

---

### IT-4: `at::` and `from_now::` together exits 1

- **Given:** Account `alice@acme.com` exists
- **When:** `clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z from_now::+1h`
- **Then:** Exits 1. stderr names the conflict between `at::` and `from_now::`.
- **Exit:** 1
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### IT-5: `at::` and `clear::` together exits 1

- **Given:** Account `alice@acme.com` exists
- **When:** `clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z clear::1`
- **Then:** Exits 1. stderr names the conflict between `at::` and `clear::`.
- **Exit:** 1
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### IT-6: `from_now::` and `clear::` together exits 1

- **Given:** Account `alice@acme.com` exists
- **When:** `clp .account.renewal name::alice@acme.com from_now::+1h clear::1`
- **Then:** Exits 1. stderr names the conflict between `from_now::` and `clear::`.
- **Exit:** 1
- **Source:** [004_parameter_interactions.md](../../../../docs/cli/004_parameter_interactions.md)

---

### IT-7: No operation param provided exits 1

- **Given:** Account `alice@acme.com` exists
- **When:** `clp .account.renewal name::alice@acme.com` (no `at::`, `from_now::`, or `clear::`)
- **Then:** Exits 1. stderr indicates that at least one of `at::`, `from_now::`, or `clear::` is required.
- **Exit:** 1
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md)

---

### IT-8: `dry::1` prints would-be value without writing

- **Given:** Account `alice@acme.com` exists; `alice@acme.com.json` present
- **When:** `clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z dry::1`
- **Then:** Exits 0. stdout contains the would-be `_renewal_at` value. `alice@acme.com.json` is NOT modified after the command.
- **Exit:** 0
- **Source:** [param/004_dry.md](../../../../docs/cli/param/004_dry.md)

---

### IT-9: `name::all` updates every saved account

- **Given:** Two accounts `alice@acme.com` and `bob@acme.com` saved
- **When:** `clp .account.renewal name::all from_now::+0m`
- **Then:** Exits 0. Both `alice@acme.com.json` and `bob@acme.com.json` contain an updated `_renewal_at` field.
- **Exit:** 0
- **Source:** [param/001_name.md](../../../../docs/cli/param/001_name.md)

---

### IT-10: `name::` with unknown account exits 2

- **Given:** Account `nobody@example.com` is not in the credential store
- **When:** `clp .account.renewal name::nobody@example.com at::2026-06-29T21:00:00Z`
- **Then:** Exits 2. stderr indicates account not found.
- **Exit:** 2
- **Source:** [param/001_name.md](../../../../docs/cli/param/001_name.md)

---

### IT-11: `name::alice,bob` comma-list updates both

- **Given:** Accounts `alice@acme.com` and `bob@acme.com` saved
- **When:** `clp .account.renewal name::alice@acme.com,bob@acme.com at::2026-06-29T21:00:00Z`
- **Then:** Exits 0. Both `alice@acme.com.json` and `bob@acme.com.json` contain `"_renewal_at": "2026-06-29T21:00:00Z"`.
- **Exit:** 0
- **Source:** [param/001_name.md](../../../../docs/cli/param/001_name.md)

---

### IT-12: Existing `oauthAccount` content preserved on write

- **Given:** `alice@acme.com.json` contains a full `oauthAccount` object with all credential fields populated
- **When:** `clp .account.renewal name::alice@acme.com at::2026-06-29T21:00:00Z`
- **Then:** Exits 0. `alice@acme.com.json` `oauthAccount` field is unchanged — only `_renewal_at` is added/updated.
- **Exit:** 0
- **Source:** [schema/002_account_json.md](../../../../docs/schema/002_account_json.md)

---

### IT-13: Past `at::` value accepted (auto-advanced by `.usage`)

- **Given:** Account `alice@acme.com` exists
- **When:** `clp .account.renewal name::alice@acme.com at::2020-01-01T00:00:00Z`
- **Then:** Exits 0. `alice@acme.com.json` contains `"_renewal_at": "2020-01-01T00:00:00Z"` verbatim — no rejection of past timestamps.
- **Exit:** 0
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md)

---

### IT-14: Unknown parameter rejected → exit 1

- **Given:** Account `alice@acme.com` exists
- **When:** `clp .account.renewal unknown::x`
- **Then:** Exits 1. stderr indicates unrecognized parameter.
- **Exit:** 1
- **Source:** [invariant/006_param_defaults.md](../../../../docs/invariant/006_param_defaults.md)

---

### IT-15: Single prefix resolves to full email

- **Given:** Account `alice@acme.com` saved
- **When:** `clp .account.renewal name::alice at::2026-07-01T00:00:00Z`
- **Then:** Exits 0. `alice@acme.com.json` contains `"_renewal_at": "2026-07-01T00:00:00Z"`.
- **Exit:** 0
- **Source:** [param/001_name.md](../../../../docs/cli/param/001_name.md)

---

### IT-16: Comma-list with prefix tokens resolves each

- **Given:** Accounts `alice@acme.com` and `bob@acme.com` saved
- **When:** `clp .account.renewal name::alice,bob at::2026-07-01T00:00:00Z`
- **Then:** Exits 0. Both `alice@acme.com.json` and `bob@acme.com.json` contain `"_renewal_at": "2026-07-01T00:00:00Z"`.
- **Exit:** 0
- **Source:** [param/001_name.md](../../../../docs/cli/param/001_name.md)
