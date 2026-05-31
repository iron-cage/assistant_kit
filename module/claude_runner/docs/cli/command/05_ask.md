# CLI Command: ask

Quick single-turn Q&A with lightweight defaults. Facade of `run` — accepts the same
26 parameters but with defaults tuned for read-only, single-turn consultation. No
tools, no session continuation, no extended thinking unless explicitly overridden.

**Syntax:**

```sh
clr ask [OPTIONS] [MESSAGE]
```

**Parameters:**

All 26 parameters from [`run`](01_run.md) are accepted. The following defaults differ:

| Parameter | `run` default | `ask` default | Notes |
|-----------|---------------|---------------|-------|
| [`-p`/`--print`](../param/002_print.md) | auto | **true** | Always on for `ask` |
| [`--no-skip-permissions`](../param/005_no_skip_permissions.md) | false | **true** | No bypass by default |
| [`--new-session`](../param/007_new_session.md) | false | **true** | No continuation by default |
| [`--max-tokens`](../param/009_max_tokens.md) | 200000 | **16384** | Shorter default for Q&A |
| [`--no-ultrathink`](../param/014_no_ultrathink.md) | false | **true** | No suffix by default |
| [`--effort`](../param/017_effort.md) | max | **high** | Lower reasoning default |
| [`--no-chrome`](../param/021_no_chrome.md) | false | **true** | No browser by default |
| [`--no-persist`](../param/022_no_persist.md) | false | **true** | No session state by default |

**Execution Modes:**

| Invocation | Mode | Path |
|------------|------|------|
| `clr ask` | Interactive REPL | `execute_interactive()` (no `-c`) |
| `clr ask "What is X?"` | **Print (default)** | `execute()` + `--print` (no `-c`) |
| `clr ask --interactive "What is X?"` | Interactive | `execute_interactive()` (no `-c`) |
| `clr ask --dry-run "What is X?"` | Preview only | `describe()` / `describe_env()` |
| `clr ask --trace "What is X?"` | Trace (print then execute) | `describe_env()` + `describe()` to stderr, then `execute()` |

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (parse failure, execution error) |
| N | Passthrough from claude subprocess |

**Examples:**

```sh
# Quick Q&A
clr ask "What does the ClaudeCommand builder do?"

# Ask about a specific file
clr ask --file src/lib.rs "Summarize the public API"

# Override effort for complex analysis
clr ask --effort max "Analyze this architectural decision"

# Override token limit for a detailed answer
clr ask --max-tokens 200000 "Explain Rust lifetime rules in detail"
```

**Notes:**

`ask` is a facade of `run` — same parameter set, same execution path, different defaults.
Parameters without a counterpart opt-in flag (e.g., `--no-chrome`, `--no-persist`) cannot
be reversed within `ask`; use `run` when full control is needed.

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | — |
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | — |
| 3 | [System Prompt](../param_group/03_system_prompt.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
| 22 | [022_session_isolation_subdir.md](../user_story/022_session_isolation_subdir.md) | Developer |
