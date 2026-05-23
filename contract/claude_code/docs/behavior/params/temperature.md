# temperature

Controls the randomness of Claude's responses.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_TEMPERATURE` |
| Config Key | — |

### Type

float — valid range: 0.0–1.0

### Default

`1.0`

### Description

Controls the randomness of Claude's responses. Lower values (toward 0.0) make responses more deterministic and focused. Higher values (toward 1.0) increase variability and creativity. The default of 1.0 uses the model's standard sampling behaviour. Most coding and analysis tasks benefit from values between 0.3 and 0.7 for more consistent outputs.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |