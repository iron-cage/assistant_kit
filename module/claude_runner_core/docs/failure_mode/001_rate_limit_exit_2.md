# Failure Mode: Rate-Limit Exit Code 2

### Scope

- **Purpose**: Document that `claude` exits with code 2 and completely empty output when rate-limited, with no error text on any stream.
- **Responsibility**: State the sentinel, detection rule, and handling guidance.
- **In Scope**: Exit-code-2 detection, `ErrorKind::RateLimit` mapping, handling recommendations.
- **Out of Scope**: Rate-limit text patterns in stdout/stderr (→ `failure_mode/002_diagnostic_on_stdout.md`).

### Behavior

When the `claude` binary hits a usage limit, it may exit with code **2** and produce **zero output** on both stdout and stderr. There is no human-readable explanation anywhere in the process output.

This is the canonical rate-limit sentinel for the claude CLI:

| Condition | Value |
|-----------|-------|
| `exit_code` | `2` |
| `stdout` | empty |
| `stderr` | empty |
| `ErrorKind` | `RateLimit` |

A generic non-zero exit handler will treat this as an unknown error, log nothing useful, and retry incorrectly or fail silently.

### Detection Rule

```
if exit_code == 2 { ErrorKind::RateLimit }
```

`classify_error()` applies this rule **after** pattern scanning: if no text pattern matches in stdout/stderr, exit code 2 is treated as `RateLimit` regardless of output content (which is always empty in this case).

### Handling Guidance

- **Do not retry immediately** — exit 2 means the quota is exhausted; retrying without a wait makes it worse.
- **Log with context** — since there is no message from claude, the caller must emit the diagnostic (`ErrorKind::RateLimit` classification + timestamp).
- **Back off** — wait before retrying; the appropriate interval depends on quota type (per-minute vs daily).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [failure_mode/002_diagnostic_on_stdout.md](002_diagnostic_on_stdout.md) | Rate-limit text that does appear but on the wrong channel |
| doc | [failure_mode/004_exit_1_ambiguity.md](004_exit_1_ambiguity.md) | Other exit codes that also indicate rate-limit with text |
| source | `../../src/types.rs` | `ErrorKind::RateLimit` variant, `classify_error()` implementation |
| test | `../../tests/classify_error_test.rs` | T01: exit=2 empty → `RateLimit` |

### Sources

| File | Notes |
|------|-------|
| `tests/classify_error_test.rs` | T01 confirms exit-2 sentinel; BUG-037 root-cause comment |
