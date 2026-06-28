# sandbox_allow_apple_events

Allows Apple Events in sandbox mode on macOS.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `sandbox.allowAppleEvents` |

### Type

bool

### Default

false

### Since

v2.1.181

### Description

When running in sandbox mode on macOS, this setting allows Apple Events
(AppleScript/osascript) to pass through the sandbox boundary. By default,
sandbox mode blocks Apple Events to prevent untrusted code from controlling
other applications.

macOS-only; no effect on Linux or Windows.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [056_sandbox_mode.md](056_sandbox_mode.md) | Sandbox mode |
