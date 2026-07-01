# enable_auto_mode

Enables the auto-mode permission classifier for automatic tool approval decisions.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_ENABLE_AUTO_MODE` |
| Config Key | — |

### Type

bool

### Default

false

### Since

v2.1.158

### Description

Enables the auto-mode classifier that automatically decides whether to approve
or deny tool calls based on risk assessment. When enabled, the `auto` permission
mode uses a trained classifier to evaluate tool safety rather than prompting the
user for every tool call.

Related to the `claude auto-mode` subcommand which inspects the classifier
configuration.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [046_permission_mode.md](046_permission_mode.md) | Permission mode selection including `auto` |
| doc | [../subcommand/003_auto_mode.md](../subcommand/003_auto_mode.md) | Inspect auto-mode classifier config |
