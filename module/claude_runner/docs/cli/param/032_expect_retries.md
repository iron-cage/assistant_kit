# CLI Parameter: --expect-retries

Maximum number of independent re-invocations when `--expect-strategy retry` is
active and the output does not match `--expect`. After all retries are exhausted
without a match, the runner exits with code 3. Only meaningful when
`--expect-strategy retry` is set; silently ignored otherwise.

- **Type:** u8 (0–255)
- **Default:** `0`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
# Try up to 4 times total (1 initial + 3 retries)
clr ask "yes or no?" --expect "yes|no" --expect-strategy retry --expect-retries 3

# Zero retries: one attempt only (same as --expect-strategy fail)
clr ask "yes or no?" --expect "yes|no" --expect-strategy retry --expect-retries 0
```

**Note:** The value is the number of *re-invocations*, not total attempts. A
value of N means the command runs at most N+1 times (1 initial + N retries).

**Note:** Values above 255 are rejected at parse time with exit 1 and an error
message. The valid range is 0–255.

**Note:** Invalid values (non-numeric, negative) are rejected at parse time
with exit 1.

**Env var:** `CLR_EXPECT_RETRIES` — accepts a decimal integer string (0–255);
invalid values are rejected at parse time.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 21 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

### See Also

- [`--expect`](030_expect.md) — defines the expected values
- [`--expect-strategy`](031_expect_strategy.md) — activates retry behavior

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 24 | [024_enum_output_validation.md](../user_story/024_enum_output_validation.md) | Developer |
