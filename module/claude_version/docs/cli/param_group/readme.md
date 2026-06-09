# param_group/ — cm Parameter Groups

### Scope

- **Purpose**: Logical groupings of cm parameters by shared purpose.
- **Responsibility**: Group membership, semantics, and per-group parameter and command tables.
- **In Scope**: 4 parameter groups — Output Control, Execution Control, Settings Identity, Config Identity.
- **Out of Scope**: Individual parameter details (→ `../param/`), cross-parameter constraints (→ `../004_parameter_interactions.md`).

**Boundary note:** This directory documents group *membership* (which parameters belong together and why). It does not document what happens when parameters from different groups are combined — that is the responsibility of `../004_parameter_interactions.md`.

### Responsibility Table

| File | Responsibility |
|------|---------------|
| readme.md | Index and navigation for parameter group files |
| procedure.md | Steps for adding, updating, or removing parameter group instances |
| 01_output_control.md | Output Control group — v::, format::, count:: |
| 02_execution_control.md | Execution Control group — dry::, force:: |
| 03_settings_identity.md | Settings Identity group — key::, value:: |
| 04_config_identity.md | Config Identity group — key::, value::, scope::, unset:: |

### All Groups (4 total)

| # | Group | Parameters | Purpose |
|---|-------|------------|---------|
| 1 | [Output Control](01_output_control.md) | 3 | Control output appearance and volume |
| 2 | [Execution Control](02_execution_control.md) | 2 | Control mutation behavior |
| 3 | [Settings Identity](03_settings_identity.md) | 2 | Identify settings target (deprecated commands) |
| 4 | [Config Identity](04_config_identity.md) | 4 | Identify config target and operation |

### See Also

- [Parameters](../param/readme.md) — parameter reference
- [Commands](../command/readme.md) — command reference
- [Parameter Interactions](../004_parameter_interactions.md) — cross-group constraints
