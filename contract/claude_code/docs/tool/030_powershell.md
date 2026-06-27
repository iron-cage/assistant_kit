# Tool: PowerShell

Execute PowerShell commands natively.

### Category

Shell

### Permission Required

Yes

### Description

Executes PowerShell commands. On Windows without Git Bash, enabled automatically.
On Windows with Git Bash, rolling out progressively. On Linux/macOS/WSL, opt-in
via `CLAUDE_CODE_USE_POWERSHELL_TOOL=1`.

When the PowerShell tool is active, it replaces the Bash tool for command
execution. The `CLAUDE_CODE_POWERSHELL_RESPECT_EXECUTION_POLICY` env var
controls whether the machine's execution policy is respected.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `command` | string | yes | PowerShell command to execute |
| `timeout` | number | no | Timeout in milliseconds |

### Since

v1.0.51 (2025-07-11) — Windows native support

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [004_bash.md](004_bash.md) | Bash shell execution (alternative) |
| doc | [../params/084_ps_execution_policy.md](../params/084_ps_execution_policy.md) | `CLAUDE_CODE_POWERSHELL_RESPECT_EXECUTION_POLICY` |
