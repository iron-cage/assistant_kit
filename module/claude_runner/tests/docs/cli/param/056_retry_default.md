# Parameter :: `--retry-default`

Edge case coverage for the `--retry-default` parameter (Tier 3: fallback count when no override or class-specific count is set).
See [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-default` | Documentation |
| EC-2 | `--retry-default 0 --dry-run` → exit 0; zero accepted (disables fallback retries) | Behavioral Divergence |
| EC-3 | `--retry-default 3 --dry-run` → exit 0; nonzero accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_DEFAULT=3 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_DEFAULT=1 --retry-default 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_DEFAULT=notanumber --dry-run` → silently ignored | Validation |
| EC-7 | Class-specific set → class-specific wins over fallback (Transient=1 beats fallback=5) | Integration |
| EC-8 | No class-specific and no override → fallback fires for Transient class | Integration |
| EC-9 | No class-specific and no override → fallback fires for Account class (Account not special) | Integration |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 3 tests (EC-7, EC-8, EC-9)

**Total:** 9 edge cases

## Architectural Constraint

`--retry-default` is Tier 3 in the 3-tier resolution chain. It is used only when neither
`--retry-override` (Tier 1) nor the class-specific `--retry-on-<class>` (Tier 2) is set.
The built-in default is 2 (applies when `--retry-default` is also absent). EC-7 demonstrates
Tier 2 beats Tier 3. EC-8 demonstrates Tier 3 fires when both Tier 1 and Tier 2 are absent.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_default_help_listed` | `retry_default_test.rs` |
| EC-2 | `ec2_retry_default_zero_dry_run` | `retry_default_test.rs` |
| EC-3 | `ec3_retry_default_nonzero_dry_run` | `retry_default_test.rs` |
| EC-4 | `ec4_clr_retry_default_env_var_accepted` | `retry_default_test.rs` |
| EC-5 | `ec5_retry_default_cli_wins_over_env` | `retry_default_test.rs` |
| EC-6 | `ec6_clr_retry_default_invalid_ignored` | `retry_default_test.rs` |
| EC-7 | `ec7_class_specific_beats_retry_default` | `retry_default_test.rs` |
| EC-8 | `ec8_retry_default_fires_when_no_class_or_override` | `retry_default_test.rs` |
| EC-9 | `ec9_retry_default_fires_for_account_class` | `retry_default_test.rs` |

---

### EC-1: --help lists --retry-default

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-default`
- **Exit:** 0
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask

---

### EC-2: --retry-default 0 --dry-run → exit 0; zero accepted

- **Given:** `--retry-default 0` and `--dry-run` set
- **When:** `clr --retry-default 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 means no fallback retries; 3 means 3 fallback retries per class
- **Exit:** 0
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask

---

### EC-3: --retry-default 3 --dry-run → exit 0; nonzero accepted

- **Given:** `--retry-default 3` and `--dry-run` set
- **When:** `clr --retry-default 3 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_DEFAULT=3 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_DEFAULT=3` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_DEFAULT=3 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask

---

### EC-5: CLI wins over CLR_RETRY_DEFAULT

- **Given:** `CLR_RETRY_DEFAULT=1` set; `--retry-default 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_DEFAULT=1 clr --retry-default 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used
- **Exit:** 0
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_DEFAULT=invalid → silently ignored

- **Given:** `CLR_RETRY_DEFAULT=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_DEFAULT=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask

---

### EC-7: Class-specific count beats --retry-default (Tier 2 > Tier 3)

- **Given:** fake exits 2 (Transient) always; `--retry-on-transient 1 --retry-default 5 --transient-delay 0 -p "x"`
- **When:** `clr --retry-on-transient 1 --retry-default 5 --transient-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 2; stderr shows `[Transient]` exhausted after 2 total invocations (1 retry from class-specific=1, not 5 from fallback)
- **Exit:** 2
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask

---

### EC-8: --retry-default fires when no class-specific and no override

- **Given:** fake exits 2 (Transient) once then 0; `--retry-default 3 --retry-default-delay 0 -p "x"` (no `--retry-on-transient`)
- **When:** `clr --retry-default 3 --retry-default-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; fallback count 3 used; stderr contains `[Transient]` retry message; two invocations
- **Exit:** 0
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask

---

### EC-9: --retry-default fires for Account class (Account not special)

- **Given:** fake emits `"You've hit your limit"` + exits 2 once then 0; `--retry-default-delay 0 -p "x"` (no `--retry-on-account`, no `--retry-override`)
- **When:** `clr --retry-default-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Account]` retry message; two invocations (fallback default=2 fires for Account class)
- **Exit:** 0
- **Source:** [056_retry_default.md](../../../../docs/cli/param/056_retry_default.md)
- **Commands:** run, ask
- **Note:** Validates that Account class has no special class-level default — it uses the same Tier 3 fallback as all other classes. **Divergence from EC-8:** EC-8 uses Transient class; EC-9 uses Account class (formerly special-cased to 0).
