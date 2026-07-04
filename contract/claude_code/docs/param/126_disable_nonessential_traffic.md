# Parameter: disable_nonessential_traffic

### Forms

| Form | Value |
|------|-------|
| Env Var | `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` |

### Type

boolean (presence-activated)

### Default

Not set (all traffic enabled)

### Description

Convenience variable equivalent to setting `DISABLE_AUTOUPDATER`, `DISABLE_FEEDBACK_COMMAND`, `DISABLE_ERROR_REPORTING`, and `DISABLE_TELEMETRY` all at once. Use this instead of setting all four individually when the goal is to eliminate every non-essential network call.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [099_disable_autoupdater.md](099_disable_autoupdater.md) | One of four vars this subsumes |
| doc | [104_disable_error_reporting.md](104_disable_error_reporting.md) | One of four vars this subsumes |
| doc | [106_disable_feedback_command.md](106_disable_feedback_command.md) | One of four vars this subsumes |
| doc | [118_disable_telemetry.md](118_disable_telemetry.md) | One of four vars this subsumes |
| doc | [../pattern/001_version_pinning.md](../pattern/001_version_pinning.md) | Synthesis: full version-pinning landscape |
