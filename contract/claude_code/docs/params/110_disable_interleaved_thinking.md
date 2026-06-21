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
