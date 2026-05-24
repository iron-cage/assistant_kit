# CLI User Story: CLI Discoverability

### Scope

- **Purpose**: Document the help command as the CLI's self-documentation entry point.
- **Responsibility**: Define acceptance criteria for discovering available commands, flags, and usage patterns.
- **In Scope**: `clr help`, `clr -h`, `clr --help` invocations and expected output.
- **Out of Scope**: Individual parameter semantics (-> `param/`), runtime behavior (-> `command/`).

### Persona

New user evaluating the `clr` CLI who needs to discover available commands, parameters, and usage patterns before running any task.

### Goal

View a complete usage summary — commands, flags, and invocation syntax — without executing any Claude subprocess or modifying any state.

### Acceptance Criteria

- `clr help` prints usage information and exits with code 0
- `clr -h` and `clr --help` produce identical output to `clr help`
- Help output lists all available subcommands (run, isolated, refresh, ask, help)
- Help output lists available flags with short descriptions
- No Claude subprocess is launched; no credentials are required
- No session state is read or written

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 4 | [`help`](../command/04_help.md) | Prints usage information and exits |

### Referenced Parameter Groups

*None applicable — `help` accepts no parameters and belongs to no group.*

### Referenced Parameters

*None — `help` is a zero-parameter command.*
