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

**Rendering:** Main help (`clr --help`, `clr -h`, `clr help`) is rendered via `cli_fmt::CliHelpTemplate` using `usage_lines` (8 USAGE forms), `arguments` (one `<COMMAND>` entry), and two `option_groups`: RUNNER OPTIONS (runner-consumed params) and CLAUDE CODE OPTIONS (forwarded) (params passed through to claude). Per-subcommand help (`clr ps --help`, `clr isolated --help`, `clr ask --help`, `clr refresh --help`) uses hand-rolled `println!` output.

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
