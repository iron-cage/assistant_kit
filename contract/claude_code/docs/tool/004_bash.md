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
| doc | [../params/013_bash_timeout.md](../params/013_bash_timeout.md) | Default bash timeout |
| doc | [../params/012_bash_max_timeout.md](../params/012_bash_max_timeout.md) | Max bash timeout |
