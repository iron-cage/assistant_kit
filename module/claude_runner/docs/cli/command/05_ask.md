# CLI Command: ask

### Description

Semantic alias for `run` — identical parameter set, identical execution path, identical defaults. Use `ask` in scripts and shell history to signal intent: the invocation is a single-turn question rather than a long-running task.

-- **Parameters:** all parameters from `run` (identical defaults)
-- **Exit Codes:** 0 (success) | 1 (error) | 2 (rate-limit/transient) | 3 (expect mismatch) | 4 (timeout) | N (subprocess passthrough) | 128+signal (signal)

### Syntax

```sh
clr ask [OPTIONS] [MESSAGE]
```

### Parameters

All parameters from [`run`](01_run.md) are accepted with identical defaults. No behavioral differences exist between `ask` and `run`.

**Algorithm (1 step):** Delegate to `run` execution path with identical parameters; no `ask`-specific logic exists.

### Execution Modes

| Invocation | Mode | Path |
|------------|------|------|
| `clr ask` | Interactive REPL | `execute_interactive()` |
| `clr ask "What is X?"` | Print (default) | `execute()` + `--print` |
| `clr ask --interactive "What is X?"` | Interactive | `execute_interactive()` |
| `clr ask --dry-run "What is X?"` | Preview only | `describe()` / `describe_env()` |
| `clr ask --trace "What is X?"` | Trace (print then execute) | `describe_env()` + `describe()` to stderr, then `execute()` |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (parse failure, execution error) |
| 2 | Rate-limit passthrough or Transient retries exhausted |
| 3 | Expect mismatch — output did not match `--expect` values after all retries |
| 4 | CLR-layer watchdog timeout: subprocess exceeded `--timeout`; stderr contains "Error: timeout after Ns" |
| N | Passthrough from claude subprocess |
| 128+signal | Subprocess killed by signal; follows POSIX convention (e.g., SIGTERM → 143, SIGKILL → 137) |

### Examples

```sh
# Quick Q&A
clr ask "What does the ClaudeCommand builder do?"

# Ask about a specific file
clr ask --file src/lib.rs "Summarize the public API"

# Ask with lower effort
clr ask --effort high "What does this function return?"

# Ask in a specific project directory
clr ask --dir ~/project "What is the entry point?"
```

### Notes

`ask` is a pure semantic alias for `run` — identical parameter set, identical execution path, identical defaults. The distinction is documentation only: `ask` communicates that the invocation is a question, not a task.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`run`](01_run.md) | `ask` is a pure semantic alias for `run` with identical behavior |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | — |
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | — |
| 3 | [System Prompt](../param_group/03_system_prompt.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 6 | [006_verbose_debugging.md](../user_story/006_verbose_debugging.md) | Developer |
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
| 17 | [017_model_selection.md](../user_story/017_model_selection.md) | Developer |
| 19 | [019_mcp_config_injection.md](../user_story/019_mcp_config_injection.md) | Developer |
| 22 | [022_session_isolation_subdir.md](../user_story/022_session_isolation_subdir.md) | Developer |
| 23 | [023_output_file_capture.md](../user_story/023_output_file_capture.md) | Developer |
| 24 | [024_enum_output_validation.md](../user_story/024_enum_output_validation.md) | Developer |
| 25 | [025_concurrency_gate.md](../user_story/025_concurrency_gate.md) | Developer |

---

**Category:** Task execution
**Complexity:** 25
**API Requirement:** Write
**Idempotent:** No
**Risk Level:** Low
