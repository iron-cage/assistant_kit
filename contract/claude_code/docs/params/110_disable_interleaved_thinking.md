# Parameter: disable_interleaved_thinking

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_INTERLEAVED_THINKING` |

### Type

boolean (presence-activated)

### Default

Not set (interleaved thinking enabled)

### Description

Prevents sending the `interleaved-thinking` beta header in API requests.
Interleaved thinking allows the model to produce thinking blocks interspersed
with content blocks rather than all thinking at the start.

### Since

v1.0.1 (2025-05-22)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [094_thinking_disabled_display.md](094_thinking_disabled_display.md) | Display behavior when thinking is disabled |
| doc | [023_effort.md](023_effort.md) | Effort level (governs thinking depth) |
| doc | [014_betas.md](014_betas.md) | Beta headers (thinking uses interleaved-thinking header) |
