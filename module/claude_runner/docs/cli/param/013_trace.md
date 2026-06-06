# CLI Parameter: --trace

Print diagnostic details to stderr before executing the subprocess. Unlike `--dry-run`,
execution still proceeds — the trace is shown as a diagnostic prefix, then the
subprocess is launched. Mirrors shell `set -x` semantics.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`isolated`](../command/02_isolated.md), [`refresh`](../command/03_refresh.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md) (for `run` and `ask`), [Credential Operations](../param_group/04_credential_operations.md) (for `isolated` and `refresh`)

What `--trace` shows depends on the command:

- **`run`** / **`ask`**: assembled env vars + full `claude` subprocess command (printed to stderr before execution)
- **`isolated`**: header lines (`# clr isolated`, `# creds:`, `# timeout:`), then env vars, then assembled `claude` invocation (including `--model claude-sonnet-4-6`)
- **`refresh`**: header lines (`# clr refresh`, `# creds:`, `# timeout:`), then env vars, then assembled `claude` invocation with fixed args and `--model claude-sonnet-4-6`

```sh
# Trace on run
clr --trace "Fix bug"
# Stderr: CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
# Stderr: claude --dangerously-skip-permissions --chrome -c --print "Fix bug\n\nultrathink"
# Then: subprocess executes normally

# Trace on ask (pure alias — identical output to run)
clr ask --trace "What is X?"
# Stderr: CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
# Stderr: claude --dangerously-skip-permissions --chrome --effort max --print -c "What is X?\n\nultrathink"
# Then: subprocess executes normally

# Trace on isolated
clr isolated --creds creds.json --trace "Fix bug"
# Stderr: # clr isolated
# Stderr: # creds: /path/to/creds.json
# Stderr: # timeout: 30s
# Stderr: CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
# Stderr: CLAUDE_CODE_BASH_TIMEOUT=3600000
# Stderr: CLAUDE_CODE_BASH_MAX_TIMEOUT=7200000
# Stderr: CLAUDE_CODE_AUTO_CONTINUE=true
# Stderr: CLAUDE_CODE_TELEMETRY=false
# Stderr: claude --chrome --model claude-sonnet-4-6 --print "Fix bug"
# Then: run_isolated() executes

# Trace on refresh
clr refresh --creds creds.json --trace
# Stderr: # clr refresh
# Stderr: # creds: /path/to/creds.json
# Stderr: # timeout: 45s
# Stderr: CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
# Stderr: CLAUDE_CODE_BASH_TIMEOUT=3600000
# Stderr: CLAUDE_CODE_BASH_MAX_TIMEOUT=7200000
# Stderr: CLAUDE_CODE_AUTO_CONTINUE=true
# Stderr: CLAUDE_CODE_TELEMETRY=false
# Stderr: claude --chrome --model claude-sonnet-4-6 --print "."
# Then: run_isolated() executes
```

**Note:** `--trace` prints to stderr so it does not pollute captured stdout in print mode.
Combine with `--dry-run` if you want to preview without executing (`run` and `ask` only — trace fires after dry-run exits for those commands).

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | `--creds`, `--timeout` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | Emits env vars + claude command |
| 2 | [`isolated`](../command/02_isolated.md) | false | Emits creds path, temp HOME, timeout |
| 3 | [`refresh`](../command/03_refresh.md) | false | Emits creds path, fixed args |
| 5 | [`ask`](../command/05_ask.md) | false | Emits env vars + claude command |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [004_dry_run_preview.md](../user_story/004_dry_run_preview.md) | Developer |
| 6 | [006_verbose_debugging.md](../user_story/006_verbose_debugging.md) | Developer |
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |
| 14 | [014_credential_refresh.md](../user_story/014_credential_refresh.md) | Developer |
