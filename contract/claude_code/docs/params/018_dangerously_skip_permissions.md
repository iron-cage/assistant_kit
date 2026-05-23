# dangerously_skip_permissions

Bypasses all permission checks for every tool invocation without prompting.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--dangerously-skip-permissions` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Bypasses all permission checks for every tool invocation — file writes, bash commands, network access — without prompting. Intended only for fully sandboxed environments with no internet access. Using this flag in an untrusted or network-connected environment is a security risk. Prefer `--allow-dangerously-skip-permissions` which enables skip-permissions as an option without making it the default.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |