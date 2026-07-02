# wheel_scroll_accel

Enables mouse wheel scroll acceleration in the terminal UI.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `wheelScrollAccelerationEnabled` |

### Type

bool

### Default

false

### Since

v2.1.174

### Description

When enabled, mouse wheel scroll events in Claude Code's terminal UI use
acceleration — faster scrolling increases the scroll distance per event.
Improves navigation speed through long outputs.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [066_theme.md](066_theme.md) | UI theme configuration |
