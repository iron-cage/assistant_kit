# Feature: CLI Design

### Scope

- **Purpose**: Document the design rationale behind the `clr` CLI's `--flag value` syntax, flag surface, and parsing behavior.
- **Responsibility**: Describe why the flag-based syntax was chosen over alternatives, how flags and commands are parsed, and the reasoning behind individual flag-level design choices.
- **In Scope**: `--flag value` syntax rationale, hand-rolled parser rationale, flag precedence rules, command-vs-flag namespace separation, print/interactive mode defaults, system prompt flag exposure, binary/crate naming, `refresh` command rationale, `render_summary()` gate field choice.
- **Out of Scope**: Individual command reference (→ `../cli/command/`), individual parameter reference (→ `../cli/param/`), default flag values (→ `invariant/001_default_flags.md`), CLI reference surface — syntax tables, type definitions, parameter defaults (→ `../cli/`).

### Design

**Command syntax:** `clr` uses bare-word subcommands (`run`, `isolated`, `refresh`, `help`, `ps`, `kill`, `tools`, `ask`, `scope`, `query`, `topic`) followed by `--flag value` pairs and an optional positional message. This mirrors POSIX CLI convention (like `git`) rather than a unilang-style `.command param::value` syntax. `topic` delegates to `run`'s handler with an auto-generated `--topic` default — see [command/11_topic.md](../cli/command/11_topic.md).

**Parsing:** A hand-rolled parser (no external CLI framework) validates an explicit whitelist of known flags; unknown flags produce an error with a `--help` hint. Positional arguments are joined with spaces to form the message. Duplicate value-flags resolve last-wins (matches curl/git convention).

**Mode selection:** `clr` defaults to print mode (captured stdout via `execute()` + `--print`) when a message is present, stdin is not a terminal (piped/redirected/non-interactive shell), or `--file`/piped stdin content supplies the prompt. Bare `clr` invoked from a genuine terminal — no message, no `--file`/stdin content — opens the interactive REPL instead. `--interactive` forces TTY passthrough unconditionally, overriding all three print-mode terms alike (message presence, non-TTY stdin, or `--file`/stdin content) — e.g. resuming a prior session with no new message under non-TTY stdin; `-p`/`--print` remains as an explicit alias and takes priority if both flags are set.

**Session handling:** Session continuation (`-c`) is injected by default when a prior session exists for the effective working directory; `--new-session` is the only way to disable it.

### Features

| File | Relationship |
|------|-------------|
| [feature/001_runner_tool.md](001_runner_tool.md) | Runner tool architecture implementing this CLI design |
| [feature/003_retry_hierarchy.md](003_retry_hierarchy.md) | Retry flag surface governed by these parsing rules |
| [feature/004_json_config.md](004_json_config.md) | JSON config precedence layered under the CLI flag parsing described here |

### Design Decisions

Full rationale for each decision — alternatives considered, consequences, and bug history — lives in [`../decision/`](../decision/readme.md), one file per decision. This table is a feature-level index into that collection.

| ID | Decision | Category |
|----|----------|----------|
| [D2](../decision/002_verbose_vs_quiet.md) | `--verbose` vs `--quiet` (supersedes `--verbosity`) | Parameter Conventions |
| [D3](../decision/003_print_mode_requires_content.md) | Requested print mode requires a message, `--file`, or stdin content | Behavior |
| [D4](../decision/004_positional_args_joined.md) | Positional args joined as message | Syntax |
| [D5](../decision/005_unknown_flags_rejected.md) | Unknown flags rejected | Parsing |
| [D6](../decision/006_duplicate_flags_last_wins.md) | Duplicate value-flags: last wins | Parameter Conventions |
| [D7](../decision/007_hand_rolled_parser.md) | Hand-rolled parser over clap/unilang | Parsing |
| [D8](../decision/008_three_layer_cli_docs.md) | Three-layer docs/cli/ replaces 42-file structure | Documentation |
| [D9](../decision/009_session_continuation_default.md) | Session continuation by default | Behavior |
| [D10](../decision/010_binary_named_clr.md) | Binary named `clr`, crate named `claude_runner` | Naming |
| [D11](../decision/011_print_by_default.md) | Print by default when message given, stdin is non-TTY, or file/stdin content is present; `--interactive` to opt into TTY | Behavior |
| [D12](../decision/012_expose_system_prompt.md) | Expose `--system-prompt` (replace) despite capability loss | Parameter Conventions |
| [D13](../decision/013_commands_are_bare_words.md) | Commands are bare words, not `--` flags | Syntax |
| [D14](../decision/014_dedicated_refresh_command.md) | Dedicated `refresh` command vs reusing `isolated` | Behavior |
| [D15](../decision/015_render_summary_gate.md) | `render_summary()` gates on invariant field `type=="result"`, not optional fields | Pipeline |

Decisions by concern area: **Syntax**: D4, D13 | **Parsing**: D5, D7 | **Parameter Conventions**: D2, D6, D12 | **Behavior**: D3, D9, D11, D14 | **Naming**: D10 | **Pipeline**: D15 | **Documentation**: D8

### Sources

| File | Relationship |
|------|-------------|
| `../../src/cli/parse.rs` | Flag parsing, whitelist validation, last-wins resolution |
| `../../src/cli/builder.rs` | `build_claude_command()` — session continuation guard (D9) |
| `../../src/cli/summary.rs` | `render_summary()` — invariant field gate (D15) |
| `../../src/cli/credential.rs` | `run_isolated_command()`, `run_refresh_command()` (D14) |

### Provenance

| File | Notes |
|------|-------|
| [`../decision/`](../decision/readme.md) | Full decision rationale, one file per decision — this feature doc's Design Decisions table is a summary index into that collection |

### Tests

| File | Relationship |
|------|-------------|
| `../../tests/cli_args_test.rs` | Flag parsing, whitelist rejection, last-wins duplicate resolution (D5–D7) |
| `../../tests/cli_args_ext_test.rs` | Extended flag coverage including session continuation guard (D9) |
| `../../tests/refresh_test.rs` | `clr refresh` command behavior (D14) |
| `../../tests/summary_unit_test.rs` | `render_summary()` invariant field gate (D15) |
