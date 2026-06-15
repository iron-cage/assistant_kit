# CLI Parameter: --expect-strategy

Mismatch handling strategy applied when `--expect` validation fails. Only
meaningful when `--expect` is set; silently ignored otherwise.

- **Type:** enum (`fail` | `retry` | `default:<VALUE>`)
- **Default:** `fail`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

**Strategies:**

| Value | Behavior |
|-------|----------|
| `fail` | Exit 3 immediately on first mismatch |
| `retry` | Re-invoke the same command up to `--retry-on-validation` times; exit 3 after all retries exhausted |
| `default:<VALUE>` | On mismatch, output `<VALUE>` as stdout and exit 0 |

```sh
# Hard fail on mismatch (explicit; same as omitting --expect-strategy)
clr ask "yes or no?" --expect "yes|no" --expect-strategy fail

# Retry up to 3 times before giving up
clr ask "yes or no?" --expect "yes|no" --expect-strategy retry --retry-on-validation 3

# Fall back to conservative default on mismatch
clr ask "safe to proceed?" --expect "yes|no" --expect-strategy default:no
```

**Exit codes when `--expect` is active:**

| Condition | Exit Code |
|-----------|-----------|
| Output matched | 0 |
| All retries exhausted without match (`retry`) | 3 |
| Mismatch (`fail`) | 3 |
| Fallback used or output matched (`default:<VALUE>`) | 0 |

**Note:** The `retry` strategy re-invokes independently — new session, same
arguments — it does not continue the prior conversation.

**Note:** The `default:<VALUE>` strategy always exits 0. The fallback value is
emitted to stdout as-is (no trimming, no suffix).

**Note:** Invalid strategy strings (anything other than `fail`, `retry`, or
`default:…`) are rejected at parse time with exit 1.

**Env var:** `CLR_EXPECT_STRATEGY` — accepts `fail`, `retry`, or
`default:<VALUE>`; applied when `--expect-strategy` is absent from the CLI.

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
- [`--retry-on-validation`](048_retry_on_validation.md) — Validation class retry count for the `retry` strategy

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 24 | [024_enum_output_validation.md](../user_story/024_enum_output_validation.md) | Developer |
