# Invariant: Trace Universality

### Scope

- **Purpose**: Enforce that every user-invocable `clr` command that executes a subprocess supports `--trace`.
- **Responsibility**: State which commands must accept `--trace`, what each produces on stderr, and why the invariant exists.
- **In Scope**: `run`, `ask`, `isolated`, `refresh` commands; `--trace` acceptance and stderr diagnostic output contract.
- **Out of Scope**: `help` command (no subprocess — exempt), individual parameter semantics (-> `cli/param/013_trace.md`), default flag injection (-> `invariant/001_default_flags.md`).

### Invariant Statement

Every `clr` command that invokes or manages a subprocess must accept `--trace` and write diagnostic output to stderr before the subprocess is launched.

| Command | Subprocess | Supports `--trace` | Stderr Diagnostic Content |
|---------|-----------|-------------------|--------------------------|
| `run` | `claude` binary | yes | env vars + assembled `claude` command line |
| `ask` | `claude` binary | yes | env vars + assembled `claude` command line (identical to `run` — pure alias) |
| `isolated` | `claude` binary (temp HOME) | yes | credential headers (`# clr isolated`, `# creds: {path}`, `# timeout: 30s`), env vars, assembled `claude --model claude-opus-4-6 --effort max --no-session-persistence [--dangerously-skip-permissions] --print {msg}` |
| `refresh` | `claude` binary (temp HOME, fixed args) | yes | credential headers (`# clr refresh`, `# creds: {path}`, `# timeout: 45s`), env vars, assembled `claude --model claude-sonnet-4-6 --no-chrome --effort low --no-session-persistence --print "."` |
| `help` | — | exempt | no subprocess; `--trace` is not parsed |

`--trace` prints to stderr so it does not pollute captured stdout in print mode. The subprocess is always launched after trace output (unlike `--dry-run`, which suppresses execution).

**Interaction with `--dry-run`** (`run` and `ask` only): when `--dry-run` is set, the process exits before trace fires. Trace output will NOT appear on stderr when combined with `--dry-run`.

### Enforcement Mechanism

- `run` and `ask`: `--trace` is parsed by `parse_args()` into `CliArgs.trace: bool`. When `trace` is `true`, `describe_env()` and `describe()` are written to stderr before `execute()` is called.
- `isolated`: `--trace` is parsed by `parse_isolated_args()`. When set, the `IsolatedArgs` struct carries `trace: true`, and `emit_credential_trace()` writes diagnostic output (credential headers + env vars + assembled command) to stderr before `run_isolated()` is called.
- `refresh`: `--trace` is parsed by `parse_refresh_args()`. When set, `emit_credential_trace()` writes diagnostic output (credential headers + env vars + assembled command) to stderr before `run_isolated()` is called with the fixed `["--print", "."]` args.

Adding a new subprocess-executing command to `clr` requires: (1) including `--trace` in its arg parser, (2) writing diagnostic output to stderr before subprocess invocation.

### Violation Consequences

If a subprocess-executing command does not support `--trace`:
- Users cannot inspect what arguments are being forwarded to the subprocess without adding instrumentation
- Debug parity is broken — some commands are opaque while others are transparent
- CI/automation pipelines cannot conditionally enable diagnostics across all commands uniformly

### Trace Output Format

#### run / ask commands

Emitted via `describe_env()` + `describe()`:
- `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`
- `CLAUDE_CODE_BASH_TIMEOUT=3600000`
- `CLAUDE_CODE_BASH_MAX_TIMEOUT=7200000`
- `CLAUDE_CODE_AUTO_CONTINUE=true`
- `CLAUDE_CODE_TELEMETRY=false`
- Command line: `claude --dangerously-skip-permissions --chrome --effort max -c "msg\nultrathink"` (run and ask — identical output since ask is a pure alias)

#### isolated / refresh commands

Emitted via `emit_credential_trace()`:
- `# clr {label}` (e.g., `# clr isolated`, `# clr refresh`)
- `# creds: {path}`
- `# timeout: {N}s` (isolated default: 30s; refresh default: 45s)
- `describe_env()` block: `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`, `CLAUDE_CODE_BASH_TIMEOUT=3600000`, `CLAUDE_CODE_BASH_MAX_TIMEOUT=7200000`, `CLAUDE_CODE_AUTO_CONTINUE=true`, `CLAUDE_CODE_TELEMETRY=false`
- `describe()` block: `claude --model {model} [injected flags] [args]` (e.g., `--model claude-opus-4-6 --effort max --no-session-persistence --dangerously-skip-permissions --print "Fix bug"` for isolated; `--model claude-sonnet-4-6 --no-chrome --effort low --no-session-persistence --print "."` for refresh)

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Execution modes that launch subprocesses |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/parse.rs` | `parse_args()`, `parse_isolated_args()`, `parse_refresh_args()` — `trace` field in each args struct |
| `../../src/cli/mod.rs` | `dispatch_ask()` — uses `CliArgs.trace` from `parse_args()` |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/docs/invariant/004_trace_universality.md` | IN-1 through IN-5 trace acceptance across all commands |
| `../../tests/cli_args_test.rs` | `--trace` flag parsing via `parse_args()` |
| `../../tests/docs/cli/param/13_trace.md` | EC-1 through EC-8 per-parameter trace edge cases |
