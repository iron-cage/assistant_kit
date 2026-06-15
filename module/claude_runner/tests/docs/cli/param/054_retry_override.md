# Parameter :: `--retry-override`

Edge case coverage for the `--retry-override` parameter (Tier 1: overrides all class-specific counts).
See [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-override` | Documentation |
| EC-2 | `--retry-override 0 --dry-run` → exit 0; zero accepted (disables all retries) | Behavioral Divergence |
| EC-3 | `--retry-override 3 --dry-run` → exit 0; nonzero accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_OVERRIDE=3 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_OVERRIDE=1 --retry-override 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_OVERRIDE=notanumber --dry-run` → silently ignored | Validation |
| EC-7 | `--retry-override 0` disables retry: Transient fake exits 2; still exits 2; no `[Transient]` in stderr | Integration |
| EC-8 | `--retry-override 2 --retry-on-transient 0`: override wins; 2 retries despite class-specific=0 | Integration |
| EC-9 | Override applies across classes: Service fake exits with `"API Error: "` + exit 1; `--retry-override 1` retries once | Integration |
| EC-10 | No override set; class-specific `--retry-on-transient 1` honored for Transient fake | Integration |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 4 tests (EC-7, EC-8, EC-9, EC-10)

**Total:** 10 edge cases

## Architectural Constraint

`--retry-override` is Tier 1 in the 3-tier resolution chain: when set, it replaces the retry
count for ALL error classes (Transient, Account, Auth, Service, Process, Unknown, Validation).
`--retry-override 0` completely disables retries even when class-specific counts are set.
Tier 1 has no predecessor flag. Integration tests use `--retry-override-delay 0` or the
corresponding class delay to avoid sleep.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_override_help_listed` | `retry_override_test.rs` |
| EC-2 | `ec2_retry_override_zero_dry_run` | `retry_override_test.rs` |
| EC-3 | `ec3_retry_override_nonzero_dry_run` | `retry_override_test.rs` |
| EC-4 | `ec4_clr_retry_override_env_var_accepted` | `retry_override_test.rs` |
| EC-5 | `ec5_retry_override_cli_wins_over_env` | `retry_override_test.rs` |
| EC-6 | `ec6_clr_retry_override_invalid_ignored` | `retry_override_test.rs` |
| EC-7 | `ec7_retry_override_zero_disables_all_retries` | `retry_override_test.rs` |
| EC-8 | `ec8_retry_override_beats_class_specific_zero` | `retry_override_test.rs` |
| EC-9 | `ec9_retry_override_applies_to_service_class` | `retry_override_test.rs` |
| EC-10 | `ec10_no_override_class_specific_honored` | `retry_override_test.rs` |

---

### EC-1: --help lists --retry-override

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-override`
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-2: --retry-override 0 --dry-run → exit 0; zero accepted

- **Given:** `--retry-override 0` and `--dry-run` set
- **When:** `clr --retry-override 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 disables ALL class retries; 3 enables 3 retries for every class
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-3: --retry-override 3 --dry-run → exit 0; nonzero accepted

- **Given:** `--retry-override 3` and `--dry-run` set
- **When:** `clr --retry-override 3 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_OVERRIDE=3 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_OVERRIDE=3` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_OVERRIDE=3 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-5: CLI wins over CLR_RETRY_OVERRIDE

- **Given:** `CLR_RETRY_OVERRIDE=1` set; `--retry-override 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_OVERRIDE=1 clr --retry-override 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_OVERRIDE=invalid → silently ignored

- **Given:** `CLR_RETRY_OVERRIDE=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_OVERRIDE=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-7: --retry-override 0 disables all retries regardless of class

- **Given:** fake exits 2 (Transient class); `--retry-override 0 --transient-delay 0 -p "x"`
- **When:** `clr --retry-override 0 --transient-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 2; no `[Transient]` retry message in stderr; only one invocation (no retry)
- **Exit:** 2
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-8: --retry-override beats class-specific zero

- **Given:** fake exits 2 (Transient) once then exits 0; `--retry-override 2 --retry-on-transient 0 --transient-delay 0 -p "x"`
- **When:** `clr --retry-override 2 --retry-on-transient 0 --transient-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Transient]` retry message; two invocations (override of 2 used)
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-9: Override applies to Service class

- **Given:** fake emits `"API Error: 500"` + exits 1 once then exits 0; `--retry-override 1 --service-delay 0 -p "x"`
- **When:** `clr --retry-override 1 --service-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Service]` retry message; two invocations (override used for Service class)
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask

---

### EC-10: No override set; class-specific retry honored for Transient

- **Given:** fake exits 2 once then 0; `--retry-on-transient 1 --transient-delay 0 -p "x"` (no override)
- **When:** `clr --retry-on-transient 1 --transient-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Transient]` retry message; two invocations; class-specific count used
- **Exit:** 0
- **Source:** [054_retry_override.md](../../../../docs/cli/param/054_retry_override.md)
- **Commands:** run, ask
