# Invariant: Trace Universality

### Scope

- **Purpose**: Enforce that every user-invocable `clr` command that executes a subprocess supports `--trace`.
- **Responsibility**: State which commands must accept `--trace`, what each produces on stderr, and why the invariant exists.
- **In Scope**: `run`, `ask`, `topic`, `isolated`, `refresh` commands; `--trace` acceptance and stderr diagnostic output contract.
- **Out of Scope**: `help` command (no subprocess — exempt), individual parameter semantics (-> `cli/param/013_trace.md`), default flag injection (-> `invariant/001_default_flags.md`).

### Invariant Statement

Every `clr` command that invokes or manages a subprocess must accept `--trace` and write diagnostic output to stderr before the subprocess is launched.

| Command | Subprocess | Supports `--trace` | Stderr Diagnostic Content |
|---------|-----------|-------------------|--------------------------|
| `run` | `claude` binary | yes | env vars + assembled `claude` command line |
| `ask` | `claude` binary | yes | env vars + assembled `claude` command line (identical to `run` — pure alias) |
| `topic` | `claude` binary | yes | env vars + assembled `claude` command line (identical to `run`/`ask` — delegates to `run`'s handler; `--subdir`'s default never appears in the traced line itself) |
| `isolated` | `claude` binary (temp HOME) | yes | credential headers (`# clr isolated`, `# creds: {path}`, `# timeout: 30s`), env vars, assembled `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude --chrome --model claude-opus-4-8 --effort max --no-session-persistence [--dangerously-skip-permissions] --print {msg}` |
| `refresh` | `claude` binary (temp HOME, fixed args) | yes | credential headers (`# clr refresh`, `# creds: {path}`, `# timeout: 45s`), env vars, assembled `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude --model claude-sonnet-5 --no-chrome --effort low --no-session-persistence --print "."` |
| `help` | — | exempt | no subprocess; `--trace` is not parsed |

`--trace` prints to stderr so it does not pollute captured stdout in print mode. The subprocess is always launched after trace output (unlike `--dry-run`, which suppresses execution).

**Interaction with `--dry-run`** (`run`, `ask`, and `topic` only): when `--dry-run` is set, the process exits before trace fires. Trace output will NOT appear on stderr when combined with `--dry-run`.

### Enforcement Mechanism

- `run`, `ask`, and `topic`: `--trace` is parsed by `parse_args()` into `CliArgs.trace: bool`. When `trace` is `true`, `describe_full()` is written to stderr before `execute()` is called (the single source-of-truth preview function that combines `describe_env()` env-var block + blank line + `describe()` invocation line).
- `isolated`: `--trace` is parsed by `parse_isolated_args()`. When set, the `IsolatedArgs` struct carries `trace: true`, and `emit_credential_trace()` writes diagnostic output (credential headers + env vars + assembled command) to stderr before `run_isolated()` is called.
- `refresh`: `--trace` is parsed by `parse_refresh_args()`. When set, `emit_credential_trace()` writes diagnostic output (credential headers + env vars + assembled command) to stderr before `run_isolated()` is called with the fixed `["--print", "."]` args.

Adding a new subprocess-executing command to `clr` requires: (1) including `--trace` in its arg parser, (2) writing diagnostic output to stderr before subprocess invocation.

### Violation Consequences

If a subprocess-executing command does not support `--trace`:
- Users cannot inspect what arguments are being forwarded to the subprocess without adding instrumentation
- Debug parity is broken — some commands are opaque while others are transparent
- CI/automation pipelines cannot conditionally enable diagnostics across all commands uniformly

### Trace Output Format

#### run / ask / topic commands

Emitted via `describe_full()` (env-var block, blank line, then invocation line):
- `export CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000`
- `export CLAUDE_CODE_BASH_TIMEOUT=3600000`
- `export CLAUDE_CODE_BASH_MAX_TIMEOUT=7200000`
- `export CLAUDE_CODE_AUTO_CONTINUE=true`
- `export CLAUDE_CODE_TELEMETRY=false`
- (blank line separating env block from invocation line)
- Command line: `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude --dangerously-skip-permissions --effort max --print --output-format json [-c] "msg\n\nultrathink"` (run, ask, and topic — identical output; ask and topic both delegate to `run`'s handler with no Claude-native flag divergence, so this trace format applies unchanged to all three — topic's only difference from ask is its runner-side `--subdir` default, which controls the subprocess's working directory and never appears in the traced command line itself; `--chrome` absent in print mode per BUG-304 auto-suppression; `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION` prefix per BUG-246 WYSIWYG fix — `CLAUDE_CODE_CHILD_SESSION` always stripped unconditionally to prevent spurious child-session transcript warnings in the spawned process; `[-c]` present when a session file exists in the session dir; `--output-format json` auto-injected when output style is `summary` — the default)

#### isolated / refresh commands

Emitted via `emit_credential_trace()`, which calls `describe_full()` internally:
- `# clr {label}` (e.g., `# clr isolated`, `# clr refresh`)
- `# creds: {path}`
- `# timeout: {N}s` (isolated default: 30s; refresh default: 45s)
- `describe_full()` env block: `export CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000`, `export CLAUDE_CODE_BASH_TIMEOUT=3600000`, `export CLAUDE_CODE_BASH_MAX_TIMEOUT=7200000`, `export CLAUDE_CODE_AUTO_CONTINUE=true`, `export CLAUDE_CODE_TELEMETRY=false` (plus `export HOME=/tmp/claude_isolated_{pid}` — the isolated temp HOME)
- (blank line separating env block from invocation line)
- Command line: `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude --chrome --model {model} [injected flags] [args]` for isolated (e.g., `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude --chrome --model claude-opus-4-8 --effort max --no-session-persistence --dangerously-skip-permissions --print "Fix bug"`); `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude --model claude-sonnet-5 --no-chrome --effort low --no-session-persistence --print "."` for refresh (`env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION` prefix per BUG-246; `--chrome` present in isolated because `emit_credential_trace()` uses `ClaudeCommand::new()` default, not the builder.rs BUG-304 print-mode suppression path)

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
