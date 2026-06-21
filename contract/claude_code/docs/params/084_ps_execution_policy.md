# ps_execution_policy

Controls whether PowerShell respects the system execution policy.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_POWERSHELL_RESPECT_EXECUTION_POLICY` |
| Config Key | — |

### Type

bool

### Default

false

### Since

v2.1.143

### Description

When set to true on Windows, Claude Code's Bash tool respects the system
PowerShell execution policy instead of bypassing it. By default, Claude Code
uses `-ExecutionPolicy Bypass` when invoking PowerShell to avoid policy
restrictions. This env var restores the system default policy behavior.

Windows-only; no effect on Linux or macOS.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [013_bash_timeout.md](013_bash_timeout.md) | Bash tool timeout |
