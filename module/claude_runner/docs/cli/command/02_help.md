# CLI Command: help

### Description

Print usage information listing all commands, flags, and their defaults, then exit with code 0. Use `clr help` (or `-h` / `--help`) when onboarding to the CLI or verifying available subcommands without running any task.

-- **Exit Codes:** 0 (always)

### Syntax

```sh
clr help
clr -h
clr --help
```

### Parameters

None — `help` accepts no parameters.

**Algorithm (1 step):** Render the CLI help template via `cli_fmt::CliHelpTemplate`; exit 0.

### Notes

`clr help` is the canonical word-subcommand form. `--help` / `-h` anywhere in argv are parameter aliases that trigger identical behavior. All three forms override any other flags. Empty argv (no arguments) enters interactive mode, not help.

Rendered via `cli_fmt::CliHelpTemplate` using `usage_lines` (8 USAGE forms), `arguments` (one `<COMMAND>` entry), and two `option_groups`: RUNNER OPTIONS (runner-consumed params) and CLAUDE CODE OPTIONS (forwarded). Per-subcommand help (`clr ps --help`, `clr isolated --help`, `clr ask --help`, `clr refresh --help`) uses hand-rolled `println!` output.

### Examples

```sh
clr help
clr -h
clr --help
```

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`run`](01_run.md) | Primary command whose flags appear in help output |
| 2 | [`tools`](08_tools.md) | Complementary discovery: `tools` lists built-in Claude Code tools |

### Referenced Parameter Groups

*None — `help` accepts no parameters.*

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 16 | [016_cli_discoverability.md](../user_story/016_cli_discoverability.md) | New User |

---

**Category:** CLI discoverability
**Complexity:** 0
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
