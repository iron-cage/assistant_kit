# CLI Type: EventType

Enumeration of the 8 canonical journal event types produced by `clr`.

- **Kind:** Enum
- **Fundamental:** String
- **Key Constraint:** Case-insensitive match against 8 variants

### Variants

| Variant | Produced By | Fields |
|---------|-------------|--------|
| `execution` | `run`, `ask` print-mode completions | exit_code, duration, cost, model, tokens_in, tokens_out, stdout, stderr |
| `credential` | `isolated`, `refresh` credential operations | creds, outcome, duration |
| `gate_wait` | `run`, `ask` concurrency gate blocking | wait_duration, attempt, max_sessions |
| `retry` | Rate-limit/error retry attempts | error_class, attempt, max_retries, delay |
| `timeout` | Subprocess timeout (exit 4) | timeout_secs, partial_stdout |
| `runner_retry` | Runner-level spawn retry | attempt, max_retries, delay |
| `validation_retry` | Expect-validation retry | pattern, strategy, attempt |
| `interactive` | Interactive REPL session start/end | session_duration |

### Validation

- Case-insensitive: `Execution`, `EXECUTION`, `execution` all match
- Invalid variant causes exit 1 with message listing valid options

### Referenced Parameters

| # | Parameter |
|---|-----------|
| 03 | [`type`](../param/03_type.md) |
