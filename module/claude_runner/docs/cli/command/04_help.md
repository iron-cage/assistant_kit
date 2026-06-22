# CLI Command: help

Print usage information listing all commands, flags, and their defaults,
then exit with code 0.

**Syntax:**

```sh
clr help
clr -h
clr --help
```

**Parameters:** none

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Always |

**Rendering:** Help output is rendered via `cli_fmt::CliHelpTemplate` using `usage_lines` (multiple USAGE forms), `option_groups` (COMMANDS, OPTIONS, RETRY OPTIONS, CREDENTIAL OPTIONS sections), and `examples`. Per-subcommand help (`clr ps --help`, `clr isolated --help`, `clr ask --help`, `clr refresh --help`) is each rendered as a separate `CliHelpTemplate` call with subcommand-specific data.

**Notes:** `clr help` is the canonical word-subcommand form. `--help` / `-h`
anywhere in argv are parameter aliases that trigger identical behavior. All
three forms override any other flags. Empty argv (no arguments) enters
interactive mode, not help.

### Examples

```sh
clr help
clr -h
clr --help
```

### Referenced Parameter Groups

*None — `help` accepts no parameters.*

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 16 | [016_cli_discoverability.md](../user_story/016_cli_discoverability.md) | New User |
