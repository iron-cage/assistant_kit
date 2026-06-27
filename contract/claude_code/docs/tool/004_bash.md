# Tool: Bash

Execute shell commands.

### Category

Shell

### Description

Executes bash commands and returns stdout/stderr output. Working directory persists between calls but shell state does not. Default timeout is 120000ms (2 minutes), configurable up to 600000ms (10 minutes). Supports background execution via `run_in_background` parameter. Shell environment is initialized from the user's profile.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [../params/013_bash_timeout.md](../params/013_bash_timeout.md) | `CLAUDE_CODE_BASH_TIMEOUT` — runner-level default timeout |
| doc | [../params/012_bash_max_timeout.md](../params/012_bash_max_timeout.md) | `CLAUDE_CODE_BASH_MAX_TIMEOUT` — runner-level max timeout |
| doc | [../params/096_bash_default_timeout_ms.md](../params/096_bash_default_timeout_ms.md) | `BASH_DEFAULT_TIMEOUT_MS` — binary-level default timeout |
| doc | [../params/097_bash_max_output_length.md](../params/097_bash_max_output_length.md) | `BASH_MAX_OUTPUT_LENGTH` — output cap before file save |
| doc | [../params/098_bash_max_timeout_ms.md](../params/098_bash_max_timeout_ms.md) | `BASH_MAX_TIMEOUT_MS` — binary-level max timeout cap |
| doc | [029_monitor.md](029_monitor.md) | Background shell execution variant |
| doc | [030_powershell.md](030_powershell.md) | PowerShell alternative on Windows |
