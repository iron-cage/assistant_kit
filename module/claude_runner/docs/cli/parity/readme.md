# CLI Parity

### Scope

- **Purpose**: Cross-command behavioral parity comparisons.
- **Responsibility**: Document behavioral differences and shared characteristics across clr commands, covering param surface, execution modes, session handling, auto-injections, exit codes, and credential lifecycle.
- **In Scope**: Multi-command comparison matrices; behavioral divergences; dimension-by-dimension parity analysis.
- **Out of Scope**: Default injection values with historical traceability (-> `002_command_defaults.md`); individual command reference (-> `command/`); individual parameter docs (-> `param/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `001_run_ask_isolated.md` | Parity matrix: run, ask, isolated across all behavioral dimensions |
| `002_isolated_refresh.md` | Parity matrix: isolated vs refresh (credential operation commands) |

### See Also

- [002_command_defaults.md](../002_command_defaults.md) — default injection values and Plan 009 design traceability
- [command/](../command/readme.md) — individual command reference (8 commands)
