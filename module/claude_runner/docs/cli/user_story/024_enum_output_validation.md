# CLI User Story: Enum Output Validation

### Scope

- **Purpose**: Document `--expect` / `--expect-strategy` / `--retry-on-validation` as runner-native
  output validation for automations that require a fixed-option response.
- **Responsibility**: Define acceptance criteria for enum matching, strategy dispatch, retry
  semantics, and exit code behavior.
- **In Scope**: `--expect "val1|val2|..."` case-insensitive matching, `fail`/`retry`/`default:<V>`
  strategies, retry re-invocation semantics, exit code 3, `CLR_EXPECT*` / `CLR_RETRY_ON_VALIDATION` env vars.
- **Out of Scope**: JSON Schema validation (→ 013_structured_json_pipeline.md), free-text
  validation, `--expect` in interactive mode (silently ignored).

### Persona

Developer or CI script that needs Claude to produce one of a finite set of values (yes/no, a
severity level, a status word) and wants the runner to enforce the constraint — exiting non-zero
or retrying automatically on mismatch rather than requiring post-processing in shell.

### Goal

Constrain Claude's print-mode output to a set of expected values so that downstream pipeline
stages receive a validated response without additional shell scripting.

### Acceptance Criteria

- `clr ask "Ready? Answer yes or no" --expect "yes|no"` exits 0 when output matches `yes` or
  `no` (case-insensitive, trimmed); exits 3 when output does not match
- `--expect-strategy fail` (default) exits 3 immediately on first mismatch
- `--expect-strategy retry --retry-on-validation 2` re-invokes independently up to 2 additional times
  before exiting 3; total attempts = 3 (1 initial + 2 retries)
- `--expect-strategy default:no` outputs `no` to stdout and exits 0 on mismatch — fallback value
  is emitted as-is
- Matching is case-insensitive: output `"YES"` matches expected `"yes"`
- Matching trims whitespace: output `"yes\n"` matches expected `"yes"`
- `--expect` is silently ignored in interactive mode (no message, no `--print`)
- `CLR_EXPECT`, `CLR_EXPECT_STRATEGY`, `CLR_RETRY_ON_VALIDATION` env vars apply when the corresponding
  CLI flags are absent
- Invalid strategy string (e.g., `--expect-strategy bogus`) causes `clr` to exit 1 at parse time
- `--retry-on-validation` value > 255 causes `clr` to exit 1 at parse time

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Primary command; validation applies in print mode |
| 5 | [`ask`](../command/05_ask.md) | Also supported; same behavior |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--expect`, `--expect-strategy`, `--retry-on-validation` are Runner Control members |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 30 | [`--expect`](../param/030_expect.md) | Pipe-separated enum values to validate against |
| 31 | [`--expect-strategy`](../param/031_expect_strategy.md) | Mismatch handling strategy |
| 48 | [`--retry-on-validation`](../param/048_retry_on_validation.md) | Re-invocation cap for `retry` strategy |
| 2 | [`--print`](../param/002_print.md) | Activates print mode (capture); required for validation |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 2 | [Print Mode Capture](002_print_mode_capture.md) | `--expect` validates output from print mode capture |
| 13 | [Structured JSON Pipeline](013_structured_json_pipeline.md) | Orthogonal: `--json-schema` constrains structure; `--expect` validates literal values |
| 18 | [Env-var Configuration](018_env_var_configuration.md) | `CLR_EXPECT*` / `CLR_RETRY_ON_VALIDATION` vars are instances of the CLR_* env var system |
