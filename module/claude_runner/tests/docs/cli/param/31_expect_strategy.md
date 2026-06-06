# Parameter :: `--expect-strategy`

Edge case coverage for the `--expect-strategy` parameter. See [031_expect_strategy.md](../../../../docs/cli/param/031_expect_strategy.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `fail` (default) → exit 3 on mismatch | Behavioral Divergence |
| EC-2 | `retry` — output matches on 2nd attempt → exit 0 | Behavioral |
| EC-3 | `retry` — all retries exhausted → exit 3 | Behavioral |
| EC-4 | `default:no` → outputs "no", exit 0 on mismatch | Behavioral Divergence |
| EC-5 | Invalid strategy value → exit 1, error message | Error Handling |
| EC-6 | `--expect-strategy` without `--expect` → silently ignored | Edge Case |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-4)
- Behavioral: 2 tests (EC-2, EC-3)
- Error Handling: 1 test (EC-5)
- Edge Case: 1 test (EC-6)

**Total:** 6 edge cases

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `t02_expect_mismatch_default_fail_exits_3` | `expect_validation_test.rs` |
| EC-2 | `t07_retry_matches_on_second_attempt` | `expect_validation_test.rs` |
| EC-3 | `t08_retry_exhausted_exits_3` | `expect_validation_test.rs` |
| EC-4 | `t09_default_strategy_outputs_fallback_exits_0` | `expect_validation_test.rs` |
| EC-5 | `t10_invalid_strategy_exits_1` | `expect_validation_test.rs` |
| EC-6 | `t12_strategy_without_expect_silently_ignored` | `expect_validation_test.rs` |

---

### EC-1: `fail` strategy → exit 3 on mismatch

- **Given:** claude returns "maybe"; `--expect "yes|no" --expect-strategy fail`
- **When:** mocked to return "maybe"
- **Then:** Exit 3; no retries attempted
- **Exit:** 3
- **Source:** [031_expect_strategy.md](../../../../docs/cli/param/031_expect_strategy.md)
- **Commands:** run, ask

---

### EC-2: `retry` — matches on 2nd attempt → exit 0

- **Given:** claude returns "maybe" then "yes"; `--expect "yes|no" --expect-strategy retry`
- **When:** mocked: 1st call returns "maybe", 2nd returns "yes"
- **Then:** Exit 0; total of 2 invocations
- **Exit:** 0
- **Source:** [031_expect_strategy.md](../../../../docs/cli/param/031_expect_strategy.md)
- **Commands:** run, ask

---

### EC-3: `retry` — all retries exhausted → exit 3

- **Given:** claude always returns "maybe"; `--expect "yes|no" --expect-strategy retry --expect-retries 2`
- **When:** mocked to always return "maybe" (3 total attempts)
- **Then:** Exit 3 after 3 attempts (1 initial + 2 retries)
- **Exit:** 3
- **Source:** [031_expect_strategy.md](../../../../docs/cli/param/031_expect_strategy.md)
- **Commands:** run, ask

---

### EC-4: `default:no` → outputs fallback, exit 0

- **Given:** claude returns "maybe"; `--expect "yes|no" --expect-strategy default:no`
- **When:** mocked to return "maybe"
- **Then:** Exit 0; stdout contains "no" (fallback value)
- **Exit:** 0
- **Source:** [031_expect_strategy.md](../../../../docs/cli/param/031_expect_strategy.md)
- **Commands:** run, ask

---

### EC-5: Invalid strategy value → exit 1

- **Given:** `--expect "yes|no" --expect-strategy bogus`
- **When:** parse time
- **Then:** Exit 1; stderr contains error message about invalid strategy
- **Exit:** 1
- **Source:** [031_expect_strategy.md](../../../../docs/cli/param/031_expect_strategy.md)
- **Commands:** run, ask

---

### EC-6: Without `--expect` → silently ignored

- **Given:** `--expect-strategy fail` but no `--expect`
- **When:** `clr --dry-run --expect-strategy fail "task"`
- **Then:** Exit 0; no error; assembled command unaffected
- **Exit:** 0
- **Source:** [031_expect_strategy.md](../../../../docs/cli/param/031_expect_strategy.md)
- **Commands:** run, ask
