# Parameter :: `trace::`

Edge case tests for the `trace::` parameter. Tests validate boolean enforcement, default-off behavior, stderr output routing (stdout unchanged), and the timestamped diagnostic line format. Available on all `clp` commands to expose internal mechanics for diagnostics. Current test cases cover `.usage`; per-command cases are added as `trace::` is registered on each additional command (TSK-210).

**Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `trace::1` accepted — timestamped diagnostic lines appear on stderr | Trace Enabled |
| EC-2 | `trace::0` accepted — no trace output on stderr | Default Off |
| EC-3 | `trace::2` rejected (out of range) | Boundary Values |
| EC-4 | `trace::yes` rejected (type validation) | Type Validation |
| EC-5 | Default value is `0` (trace disabled) | Default |
| EC-6 | `trace::1` — trace goes to stderr; stdout output unchanged | Output Routing |
| EC-7 | `.account.use trace::1 touch::0` — accepted; no timestamped `account.use` diagnostic lines (no fetch ops) | Acceptance: `.account.use` |
| EC-8 | `.credentials.status trace::1` — accepted; timestamped diagnostic lines emitted for credential read | Acceptance: `.credentials.status` |
| EC-9 | `.accounts trace::1` — accepted on empty store; a timestamped diagnostic line for store not-found | Acceptance: `.accounts` |
| EC-10 | `.account.limits trace::1` — accepted; timestamped diagnostic line emitted before API call | Acceptance: `.account.limits` |
| EC-11 | `.account.save trace::1 dry::1` — accepted; timestamped diagnostic line emitted for credential read | Acceptance: `.account.save` |
| EC-12 | `.account.use trace::1` — accepted; account not found → exit 2; no "Unknown parameter" error | Acceptance: `.account.use` (2) |
| EC-13 | `.account.delete trace::1 dry::1` — accepted; timestamped diagnostic line emitted for store read | Acceptance: `.account.delete` |
| EC-14 | `.account.relogin trace::1 dry::1` — accepted; timestamped diagnostic line emitted | Acceptance: `.account.relogin` |
| EC-15 | DEPRECATED — `.account.rotate` removed; trace acceptance now covered by `.usage trace::1` (EC-1) | Acceptance: removed |
| EC-16 | `.token.status trace::1` — accepted; timestamped diagnostic line emitted for credential read | Acceptance: `.token.status` |
| EC-17 | `.paths trace::1` — accepted; timestamped diagnostic line emitted for path resolution | Acceptance: `.paths` |

## Test Coverage Summary

- Trace Enabled: 1 test (EC-1)
- Default Off: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Type Validation: 1 test (EC-4)
- Default: 1 test (EC-5)
- Output Routing: 1 test (EC-6)
- Acceptance\: `.account.use`: 2 tests (EC-7, EC-12)
- Acceptance\: `.credentials.status`: 1 test (EC-8)
- Acceptance\: `.accounts`: 1 test (EC-9)
- Acceptance\: `.account.limits`: 1 test (EC-10)
- Acceptance\: `.account.save`: 1 test (EC-11)
- Acceptance\: `.account.delete`: 1 test (EC-13)
- Acceptance\: `.account.relogin`: 1 test (EC-14)
- ~~Acceptance\: `.account.rotate`~~: DEPRECATED (EC-15 — command removed)
- Acceptance\: `.token.status`: 1 test (EC-16)
- Acceptance\: `.paths`: 1 test (EC-17)

**Total:** 17 edge cases

**Behavioral Divergence Pair:** EC-1 (trace enabled — diagnostics on stderr) ↔ EC-5 (absent by default — no diagnostic output)

## Test Cases
---

### EC-1: `trace::1` — timestamped diagnostic lines appear on stderr

- **Given:** `.usage` environment with valid credentials and at least one saved account.
- **When:** `clp .usage trace::1`
- **Then:** stderr contains at least one timestamped diagnostic line; exit 0.
- **Exit:** 0
- **Source fn:** `it034_trace_param_writes_to_stderr`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)
---

### EC-2: `trace::0` — explicit disable; no trace output

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage trace::0`
- **Then:** stderr contains no timestamped diagnostic lines; behavior identical to default; exit 0.
- **Exit:** 0
- **Source fn:** `it059_trace_0_no_trace_on_stderr`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)
---

### EC-3: `trace::2` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage trace::2`
- **Then:** Exit 1 with error referencing `trace::`; must be 0 or 1.
- **Exit:** 1
- **Source fn:** `it060_trace_2_rejected`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)
---

### EC-4: `trace::yes` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage trace::yes`
- **Then:** Exit 1 with type validation error referencing `trace::`.
- **Exit:** 1
- **Source fn:** `it061_trace_yes_rejected`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)
---

### EC-5: Default value is `0` (trace disabled)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage` (no `trace::` param)
- **Then:** stderr contains no timestamped diagnostic lines; behavior identical to `trace::0`; exit 0.
- **Exit:** 0
- **Source fn:** `it062_trace_default_off`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)
---

### EC-6: `trace::1` — trace output on stderr does not appear on stdout

- **Given:** `.usage` environment with valid credentials and at least one saved account.
- **When:** `clp .usage trace::1`
- **Then:** stdout contains the normal quota table output only (no timestamped diagnostic lines); stderr contains timestamped diagnostic lines; the two streams are independent; exit 0.
- **Exit:** 0
- **Source fn:** `it034_trace_param_writes_to_stderr`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-7: `.account.use trace::1 touch::0` — accepted; no timestamped `account.use` diagnostic lines emitted

- **Given:** Account `alice@home.com` saved. `touch::0` suppresses all fetch operations.
- **When:** `clp .account.use name::alice@home.com touch::0 trace::1`
- **Then:** Exits 0; `switched to 'alice@home.com'` on stdout; stderr contains no timestamped `account.use` diagnostic lines; `trace::1` is accepted without "unrecognized parameter" error.
- **Exit:** 0
- **Source fn:** `aw31_trace_touch_disabled_no_trace_lines` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-8: `.credentials.status trace::1` — accepted; trace emitted for credential read

- **Given:** Valid credentials file present.
- **When:** `clp .credentials.status trace::1`
- **Then:** Exits 0; stderr contains at least one timestamped diagnostic line for the credential file read; no "Unknown parameter" error.
- **Exit:** 0
- **Source fn:** `it_trace_credentials_status_accepted` (in `tests/cli/credentials_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-9: `.accounts trace::1` — accepted on empty store; trace emitted

- **Given:** Empty credential store (no accounts configured).
- **When:** `clp .accounts trace::1`
- **Then:** Exits 0; stderr contains a timestamped diagnostic line for store not-found; stdout shows "(no accounts configured)"; no "Unknown parameter" error.
- **Exit:** 0
- **Source fn:** `it_trace_accounts_accepted` (in `tests/cli/accounts_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-10: `.account.limits trace::1` — accepted; trace emitted before API call

- **Given:** Valid credentials and credential store directory present.
- **When:** `clp .account.limits trace::1`
- **Then:** stderr contains a timestamped diagnostic line for store read; no "Unknown parameter" error; command may exit 2 (API failure) but must not exit 1 for unknown-param.
- **Exit:** 0 or 2
- **Source fn:** `it_trace_account_limits_accepted` (in `tests/cli/account_limits_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-11: `.account.save trace::1 dry::1` — accepted; trace emitted for credential read

- **Given:** Valid credentials file and credential store present; `dry::1` suppresses write.
- **When:** `clp .account.save name::test@example.com dry::1 trace::1`
- **Then:** Exits 0 (dry-run); stderr contains a timestamped diagnostic line for credential file read; no "Unknown parameter" error.
- **Exit:** 0
- **Source fn:** `it_trace_account_save_accepted` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-12: `.account.use trace::1` — accepted; unknown account → exit 2

- **Given:** Empty credential store (account not found).
- **When:** `clp .account.use name::test@example.com trace::1`
- **Then:** Exits 2 (account not found); no "Unknown parameter" error; `trace::1` is accepted by the framework.
- **Exit:** 2
- **Source fn:** `it_trace_account_use_accepted` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-13: `.account.delete trace::1 dry::1` — accepted; trace emitted for store read

- **Given:** Account `test@example.com` saved; `dry::1` suppresses deletion.
- **When:** `clp .account.delete name::test@example.com dry::1 trace::1`
- **Then:** Exits 0 (dry-run); stderr contains a timestamped diagnostic line for store read; no "Unknown parameter" error.
- **Exit:** 0
- **Source fn:** `it_trace_account_delete_accepted` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-14: `.account.relogin trace::1 dry::1` — accepted; trace emitted

- **Given:** Account `work@acme.com` saved and active; `dry::1` suppresses re-auth.
- **When:** `clp .account.relogin dry::1 trace::1`
- **Then:** Exits 0 (dry-run); stderr contains a timestamped diagnostic line; no "Unknown parameter" error.
- **Exit:** 0
- **Source fn:** `it_trace_account_relogin_accepted` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-15: DEPRECATED — `.account.rotate` removed

> **DEPRECATED** — `.account.rotate` has been removed (Feature 038). Trace acceptance on the rotation path is covered by `.usage trace::1` (EC-1). The corresponding test `it_trace_account_rotate_accepted` should be removed during implementation.

---

### EC-16: `.token.status trace::1` — accepted; trace emitted for credential read

- **Given:** Valid credentials file present.
- **When:** `clp .token.status trace::1`
- **Then:** Exits 0; stderr contains a timestamped diagnostic line for credential file read; no "Unknown parameter" error.
- **Exit:** 0
- **Source fn:** `it_trace_token_status_accepted` (in `tests/cli/token_paths_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)

---

### EC-17: `.paths trace::1` — accepted; trace emitted for path resolution

- **Given:** Valid HOME set.
- **When:** `clp .paths trace::1`
- **Then:** Exits 0; stderr contains a timestamped diagnostic line for base path; no "Unknown parameter" error.
- **Exit:** 0
- **Source fn:** `it_trace_paths_accepted` (in `tests/cli/token_paths_test.rs`)
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/param/023_trace.md)
