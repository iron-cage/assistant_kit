# CLI Parameter: --expect

Pipe-separated list of expected output values for enum validation. After
capturing stdout from the subprocess, the runner trims whitespace and
lowercases the result, then checks whether it equals any value in the list
(values are also lowercased at parse time). When the output matches, the runner
proceeds normally. When it does not match, the strategy set by
`--expect-strategy` applies.

- **Type:** string (`val1|val2|…`)
- **Default:** — (no output validation)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr ask "Ready to deploy? Answer yes or no" --expect "yes|no"
clr ask "Is this a bug or a feature?" --expect "bug|feature|unclear"
clr ask "Rate 1-5" --expect "1|2|3|4|5" --expect-strategy retry
```

**Note:** Matching is case-insensitive and trims leading/trailing whitespace
from stdout. Both the expected values list and the captured output are
lowercased before comparison.

**Note:** `--expect` is only active in print mode (when a message is given or
`--print` is set). It is silently ignored in interactive mode.

**Note:** `--expect` and `--json-schema` are orthogonal — `--json-schema`
constrains Claude's output structure as a subprocess flag; `--expect` validates
the captured stdout against literal values after execution.

**Env var:** `CLR_EXPECT` — accepts the same `val1|val2|…` syntax.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 21 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 2 | [`isolated`](../command/02_isolated.md) | — | Validates stdout after exit; `retry` strategy unsupported → exit 1 with error (TSK-331) |
| 5 | [`ask`](../command/05_ask.md) | — | — |

### See Also

- [`--expect-strategy`](031_expect_strategy.md) — mismatch handling strategy
- [`--retry-on-validation`](048_retry_on_validation.md) — Validation class retry count (renamed from `--expect-retries`)

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 24 | [024_enum_output_validation.md](../user_story/024_enum_output_validation.md) | Developer |
