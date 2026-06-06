# Parameter :: `--expect-retries`

Edge case coverage for the `--expect-retries` parameter. See [032_expect_retries.md](../../../../docs/cli/param/032_expect_retries.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--expect-retries 3` → up to 4 total attempts | Behavioral Divergence |
| EC-2 | `--expect-retries 0` → no retries (1 attempt only) | Behavioral Divergence |
| EC-3 | `--expect-retries 256` → exit 1, value out of range | Error Handling |
| EC-4 | `CLR_EXPECT_RETRIES=3` → applies as default | Env Var |
| EC-5 | `--expect-retries 3` without `--expect-strategy retry` → silently ignored | Edge Case |
| EC-6 | No `--expect-retries` flag + retry strategy → default 0 retries → 1 attempt, exit 3 | Behavioral |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Behavioral: 1 test (EC-6)
- Edge Case: 1 test (EC-5)
- Error Handling: 1 test (EC-3)
- Env Var: 1 test (EC-4)

**Total:** 6 edge cases

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `t16_retries_3_makes_4_total_attempts` | `expect_validation_test.rs` |
| EC-2 | `t13_retries_0_means_single_attempt` | `expect_validation_test.rs` |
| EC-3 | `t11_out_of_range_retries_exits_1` | `expect_validation_test.rs` |
| EC-4 | `t14_clr_expect_retries_env_var_applied` | `expect_validation_test.rs` |
| EC-5 | `t15_retries_without_retry_strategy_ignored` | `expect_validation_test.rs` |
| EC-6 | `t17_no_retries_flag_default_zero_means_single_attempt` | `expect_validation_test.rs` |

---

### EC-1: `--expect-retries 3` → up to 4 total attempts

- **Given:** claude always returns "maybe"; `--expect "yes|no" --expect-strategy retry --expect-retries 3`
- **When:** mocked to always fail
- **Then:** Exit 3 after exactly 4 invocations (1 initial + 3 retries)
- **Exit:** 3
- **Source:** [032_expect_retries.md](../../../../docs/cli/param/032_expect_retries.md)
- **Commands:** run, ask

---

### EC-2: `--expect-retries 0` → no retries

- **Given:** claude returns "maybe"; `--expect "yes|no" --expect-strategy retry --expect-retries 0`
- **When:** mocked to return "maybe"
- **Then:** Exit 3 after exactly 1 invocation (no retries)
- **Exit:** 3
- **Source:** [032_expect_retries.md](../../../../docs/cli/param/032_expect_retries.md)
- **Commands:** run, ask

---

### EC-3: `--expect-retries 256` → exit 1 (out of range)

- **Given:** `--expect "yes|no" --expect-strategy retry --expect-retries 256`
- **When:** parse time
- **Then:** Exit 1; stderr contains error about value exceeding u8 range (max 255)
- **Exit:** 1
- **Source:** [032_expect_retries.md](../../../../docs/cli/param/032_expect_retries.md)
- **Commands:** run, ask

---

### EC-4: `CLR_EXPECT_RETRIES=3` applies as default

- **Given:** `CLR_EXPECT_RETRIES=3`; `--expect "yes|no" --expect-strategy retry`; no `--expect-retries` on CLI
- **When:** mocked to always fail
- **Then:** Exit 3 after 4 attempts (env var applied as default); same as explicit `--expect-retries 3`
- **Exit:** 3
- **Source:** [032_expect_retries.md](../../../../docs/cli/param/032_expect_retries.md)
- **Commands:** run, ask

---

### EC-5: Without `--expect-strategy retry` → silently ignored

- **Given:** `--expect-retries 5` with `--expect "yes|no" --expect-strategy fail`
- **When:** `clr --dry-run --expect "yes|no" --expect-strategy fail --expect-retries 5 "task"`
- **Then:** Exit 0; no error; retry count has no effect (strategy is fail)
- **Exit:** 0
- **Source:** [032_expect_retries.md](../../../../docs/cli/param/032_expect_retries.md)
- **Commands:** run, ask

---

### EC-6: No `--expect-retries` flag → default 0 retries → 1 attempt

- **Given:** claude always returns "maybe"; `--expect "yes|no" --expect-strategy retry`; no `--expect-retries` on CLI
- **When:** mocked to always fail
- **Then:** Exit 3 after exactly 1 invocation; `unwrap_or(0)` produces 0 retries by default — same outcome as explicit `--expect-retries 0` but exercising the implicit default path
- **Exit:** 3
- **Source:** [032_expect_retries.md](../../../../docs/cli/param/032_expect_retries.md)
- **Commands:** run, ask
