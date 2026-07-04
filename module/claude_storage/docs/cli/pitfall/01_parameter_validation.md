# Pitfall: Parameter Validation Not Implied By Default

### Scope

- **Purpose**: Document the parameter validation pitfall.
- **Responsibility**: Why default values do not substitute for explicit validation.
- **In Scope**: Parameter validation requirement for all constrained parameters.
- **Out of Scope**: Type system guarantees, unilang dispatch behaviour.

### Pitfall

Default parameter values **do not prevent invalid input**. A parameter with a default of `0`
(boolean false) still accepts `"banana"` if the command routine does not explicitly validate it.
This was documented as Finding #010 after observing inconsistent validation across command routines.

### Trigger

Adding a new parameter with a documented default value and assuming unilang will reject
out-of-range input. Unilang dispatches the parameter value as a string — validation is
entirely the command routine's responsibility.

### Required Pattern

Every parameter with value constraints must include explicit validation:

- **Boolean** (`0`/`1`): reject values other than `"0"` and `"1"` with a clear error message
- **Integer** (`min_entries::`, `limit::`): parse as integer, reject negative values or values outside range
- **Enum** (`target::`, `entry_type::`): match arms must be exhaustive against the param/ spec; include a catch-all arm that returns an argument error
- **Non-empty string** (`query::`): trim and reject whitespace-only values

Apply these validation patterns uniformly — copy from an existing command routine that already validates the same type.

### Source Reference

`src/cli/mod.rs` — `## Known Pitfalls § Parameter Validation Consistency (Finding #010)`

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| All 11 | *(all commands)* | Applies to every command accepting constrained parameters |

### Sources

- `src/cli/mod.rs:6` — Finding #010 documented in module doc comment
- `src/cli/search.rs:33` — explicit trim-and-reject pattern example
- `src/cli/search.rs:61` — exhaustive enum match arm pattern example
