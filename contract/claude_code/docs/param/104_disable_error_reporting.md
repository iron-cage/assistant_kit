# Parameter: disable_error_reporting

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_ERROR_REPORTING` |

### Type

boolean (presence-activated)

### Default

Not set (error reporting enabled)

### Description

Opts out of Sentry error reporting. Prevents crash reports and error telemetry
from being sent to Anthropic.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [118_disable_telemetry.md](118_disable_telemetry.md) | Disable telemetry |
| doc | [126_disable_nonessential_traffic.md](126_disable_nonessential_traffic.md) | Combined opt-out including this var |
