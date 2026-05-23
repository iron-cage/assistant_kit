# auto_approve_tools

Automatically approves all tool invocations without user confirmation prompts.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_AUTO_APPROVE_TOOLS` |
| Config Key | — |

### Type

bool

### Default

`false`

### Description

When true, all tool invocations are automatically approved without user confirmation prompts. Equivalent in effect to `--dangerously-skip-permissions` but applied via env var. Intended for fully automated pipelines in sandboxed environments. Setting this in an interactive or network-connected context removes a key safety layer.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |