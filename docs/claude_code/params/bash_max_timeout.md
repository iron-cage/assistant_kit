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

### Description

The maximum timeout that any individual bash command is permitted to use, regardless of what Claude requests. Acts as a ceiling on `CLAUDE_CODE_BASH_TIMEOUT`. No single bash command can run longer than this value. For long-running operations (full test suites, large builds), increase both this and `CLAUDE_CODE_BASH_TIMEOUT`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |