# Parameter: disable_telemetry

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_TELEMETRY` |

### Type

boolean (presence-activated)

### Default

Not set (telemetry enabled)

### Description

Opts out of telemetry data collection. Also respects the cross-tool `DO_NOT_TRACK`
convention. For broader opt-out including autoupdater and error reporting, use
`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [104_disable_error_reporting.md](104_disable_error_reporting.md) | Disable error reporting |
| doc | [../endpoint/007_metrics_enabled.md](../endpoint/007_metrics_enabled.md) | Metrics-enabled endpoint (org-level feature) |
