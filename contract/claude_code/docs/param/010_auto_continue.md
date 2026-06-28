# auto_continue

Automatically continues truncated responses without requiring a user prompt.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_AUTO_CONTINUE` |
| Config Key | — |

### Type

bool

### Default

`false`

### Since

pre-v1.0 (unverified)

### Description

When true, Claude automatically continues long responses that would otherwise be truncated, without requiring a user prompt to proceed. Enables fully unattended automation in `--print` mode. Without this, a truncated response in `--print` mode may hang waiting for input. The `claude_runner_core` builder defaults this to `true` for automation; the binary default is `false`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [051_print.md](051_print.md) | Print mode (enables unattended automation) |
| doc | [076_max_turns.md](076_max_turns.md) | Maximum continuation turns |