# bash_max_timeout

The maximum timeout any individual bash command is permitted to use.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_BASH_MAX_TIMEOUT` |
| Config Key | — |

### Type

integer (milliseconds)

### Default

`600 000` (10 minutes)

### Since

pre-v1.0 (unverified)

### Description

The maximum timeout that any individual bash command is permitted to use, regardless of what Claude requests. Acts as a ceiling on `CLAUDE_CODE_BASH_TIMEOUT`. No single bash command can run longer than this value. For long-running operations (full test suites, large builds), increase both this and `CLAUDE_CODE_BASH_TIMEOUT`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [013_bash_timeout.md](013_bash_timeout.md) | `CLAUDE_CODE_BASH_TIMEOUT` — default timeout (this is its ceiling) |
| doc | [098_bash_max_timeout_ms.md](098_bash_max_timeout_ms.md) | `BASH_MAX_TIMEOUT_MS` — binary-level max timeout equivalent |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool this constrains |