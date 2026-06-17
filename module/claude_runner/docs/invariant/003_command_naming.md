# Invariant: Command Naming

### Scope

- **Purpose**: Enforce the lexical distinction between commands and parameters in the `clr` CLI.
- **Responsibility**: State that all commands are bare words and all parameters are `--`/`-` prefixed flags.
- **In Scope**: Command dispatch convention, bare-word requirement, parameter prefix requirement, `run` explicit subcommand alias, `help` word-subcommand, `--help`/`-h` convenience aliases.
- **Out of Scope**: Default flag injection (-> `invariant/001_default_flags.md`), individual parameter semantics (-> `cli/param/`).

### Invariant Statement

Every `clr` command must be a bare word (no `-` or `--` prefix). Only parameters and flags may use the `--` or `-` prefix.

| Token Type | Prefix | Position | Examples |
|------------|--------|----------|----------|
| command | none (bare word) | first positional token | `run`, `ask`, `isolated`, `refresh`, `help`, `ps`, `kill` |
| parameter | `--` or `-` | anywhere after command | `--model`, `--creds`, `-p`, `--trace` |

**All commands (7):**

| Command | Dispatch | Notes |
|---------|----------|-------|
| `run` | implicit default or explicit first token | Invoked when no subcommand word is given; also accepted as `clr run …` — the `run` token is stripped and execution delegates to default run mode |
| `ask` | explicit first token | `clr ask "question"` — semantic alias for `run` |
| `isolated` | explicit first token | `clr isolated --creds ...` |
| `refresh` | explicit first token | `clr refresh --creds ...` |
| `help` | explicit first token | `clr help` |
| `ps` | explicit first token | `clr ps` — list running Claude Code sessions (Linux only) |
| `kill` | explicit first token | `clr kill <pid>` — terminate a Claude Code session by PID |

**Convenience aliases:** `--help` and `-h` are parameter-form aliases for the `help` command. They trigger identical behavior (`print_help()` + exit 0). The canonical invocation is `clr help`; the flag aliases exist for POSIX convention compliance.

### Enforcement Mechanism

Command dispatch in `run_cli()` uses exact string matching on the first non-flag token:

1. If first token is `"run"` → strip it (tokens become remainder); both `clr run …` and `clr run help` go through this step first.
2. If first token is `"help"` → call `print_help()` and return. Covers `clr help` and (post-strip) `clr run help`.
3. `match` on first token: `"ask"` → `dispatch_ask()`, `"isolated"` → `dispatch_isolated()`, `"refresh"` → `dispatch_refresh()`, `"ps"` → `dispatch_ps()`, `"kill"` → `dispatch_kill()`.
4. `guard_unknown_subcommand()` — rejects token that resembles a known subcommand; exits 1 with "Did you mean" suggestion. Guard fires when: (a) `first.len() >= 4` and `sub.starts_with(first)` (prefix truncation), or (b) `is_close_typo(first, sub)` (1-char insertion/deletion/substitution). Minimum length of 4 prevents false positives from common short words (e.g. "is" sharing a prefix with "isolated").
5. `dispatch_run()` — implicit `run` (no explicit subcommand token).

The `KNOWN_SUBCOMMANDS` guard checks for typos/truncations of all registered subcommands (`run`, `ask`, `isolated`, `refresh`, `help`, `ps`, `kill`) before `parse_args()` is reached.

`--help`/`-h` flag aliases are handled inside `parse_args()` as a pre-scan fast-path (before any flag parsing) for backward compatibility.

### Violation Consequences

If a command were prefixed with `--`:
- It becomes indistinguishable from a parameter in the `parse_args()` flag parser
- The unknown-flag guard rejects it with `Error: unknown option`
- Users cannot reason about whether a token is a mode-selector (command) or a mode-modifier (parameter)
- Shell completion scripts cannot distinguish commands from flags

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Execution modes that consume command dispatch |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/lib.rs` | `run_cli()` command dispatch |
| `../../src/cli/mod.rs` | `guard_unknown_subcommand()` with `KNOWN` subcommand list |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/cli_args_test.rs` | `clr help` word dispatch, `--help`/`-h` flag aliases, unknown subcommand detection |
| `../../tests/cli_args_ext_test.rs` | BUG-212 reproducer: `clr run` stripping; BUG-215: `clr run help` dispatching; BUG-302: guard false-positive on short common words |

### Provenance

| File | Notes |
|------|-------|
| Design decision D13 | Command naming convention rationale |
