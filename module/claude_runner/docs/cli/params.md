# Parameters

### All Parameters (15 total)

| # | Parameter | Type | Default | Valid Values | Description | Used In |
|---|-----------|------|---------|--------------|-------------|---------|
| 1 | `[MESSAGE]` | [`MessageText`](types.md#type--1-messagetext) | — | Any text | Prompt text for Claude | 1 cmd |
| 2 | `-p`/`--print` | bool | auto | present/absent | Explicit print mode (default when message given) | 1 cmd |
| 3 | `--model` | [`ModelName`](types.md#type--4-modelname) | — | Any model name | Claude model to use | 1 cmd |
| 4 | `--verbose` | bool | false | present/absent | Enable Claude verbose output | 1 cmd |
| 5 | `--no-skip-permissions` | bool | false | present/absent | Disable automatic permission bypass | 1 cmd |
| 6 | `--interactive` | bool | false | present/absent | Interactive TTY passthrough when message given | 1 cmd |
| 7 | `--new-session` | bool | false | present/absent | Start fresh session (disables default continuation) | 1 cmd |
| 8 | `--dir` | [`DirectoryPath`](types.md#type--2-directorypath) | cwd | Any path | Working directory | 1 cmd |
| 9 | `--max-tokens` | [`TokenLimit`](types.md#type--3-tokenlimit) | 200000 | 0 to 4294967295 | Max output tokens | 1 cmd |
| 10 | `--session-dir` | [`DirectoryPath`](types.md#type--2-directorypath) | — | Any path | Session storage directory | 1 cmd |
| 11 | `--dry-run` | bool | false | present/absent | Print command without executing | 1 cmd |
| 12 | `--verbosity` | [`VerbosityLevel`](types.md#type--5-verbositylevel) | 3 | 0 to 5 | Runner output gate level | 1 cmd |
| 13 | `--trace` | bool | false | present/absent | Print env+command to stderr then execute | 1 cmd |
| 14 | `--system-prompt` | [`SystemPromptText`](types.md#type--6-systemprompttext) | — | Any text | Set system prompt (replaces the default) | 1 cmd |
| 15 | `--append-system-prompt` | [`SystemPromptText`](types.md#type--6-systemprompttext) | — | Any text | Append text to the default system prompt | 1 cmd |

**Groups:** Parameters 2–4 form [Claude-Native Flags](parameter_groups.md#group--1-claude-native-flags). Parameters 5–13 form [Runner Control](parameter_groups.md#group--2-runner-control). Parameters 14–15 form [System Prompt](parameter_groups.md#group--3-system-prompt).

---

### Parameter :: 1. `[MESSAGE]`

Free-form prompt text sent to Claude Code. Multiple positional words are
joined with spaces. When a message is given, print mode is the default;
use `--interactive` to override to TTY passthrough.

- **Type:** [`MessageText`](types.md#type--1-messagetext)
- **Default:** — (none; interactive REPL when absent)
- **Command:** [`run`](commands.md#command--1-run)

```sh
clr "Fix the bug in auth.rs"
clr Fix the bug       # equivalent — words joined with space
```

---

### Parameter :: 2. `--print`

Explicit print mode flag. When a message is given, print mode is the
default — this flag is a backward-compatible explicit alias.
Captures Claude's stdout and prints it instead of passing through
the TTY.

- **Aliases:** `-p`
- **Type:** bool (standalone flag)
- **Default:** auto (active when message given; inactive for bare REPL)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Claude-Native Flags](parameter_groups.md#group--1-claude-native-flags)

```sh
clr "Explain this function"        # print mode by default
clr -p "Explain this function"     # same — explicit alias
output=$(clr "List files" --model sonnet)
```

**Note:** Print mode without a message exits with error code 1.

---

### Parameter :: 3. `--model`

Select the Claude model for this invocation.

- **Type:** [`ModelName`](types.md#type--4-modelname)
- **Default:** — (Claude Code default)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Claude-Native Flags](parameter_groups.md#group--1-claude-native-flags)
- **Validation:** requires a value; `--model` at end of argv → error

```sh
clr "Explain" --model sonnet
clr --model opus "Fix bug"
```

---

### Parameter :: 4. `--verbose`

Enable Claude Code verbose output. Passed through to the claude
subprocess.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Claude-Native Flags](parameter_groups.md#group--1-claude-native-flags)

```sh
clr --verbose "Debug this"
```

---

### Parameter :: 5. `--no-skip-permissions`

Disable the automatic `--dangerously-skip-permissions` flag that `clr` injects into every
invocation by default. Without this flag, every `clr` call silently passes
`--dangerously-skip-permissions` to the `claude` subprocess, bypassing all tool permission
prompts.

- **Type:** bool (standalone flag)
- **Default:** false (bypass is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)

```sh
clr --no-skip-permissions "Fix bug"   # bypass disabled — claude will prompt for tool approvals
```

**Note:** `--dangerously-skip-permissions` is no longer a user-facing flag. It is injected
automatically unless `--no-skip-permissions` is given. See the
[Default Flags Invariant](../invariant/001_default_flags.md#invariant-statement) in the invariant.

---

### Parameter :: 6. `--interactive`

Opt into interactive TTY passthrough when a message is given. Without
this flag, providing a message defaults to print mode (captured output).
Use `--interactive` when you want live Claude streaming output while
also providing an initial prompt.

- **Type:** bool (standalone flag)
- **Default:** false (print mode when message given)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)

```sh
clr --interactive "Fix bug"               # TTY passthrough with initial prompt
clr --interactive "Continue" --dir /proj  # interactive, specific directory
```

**Note:** No effect when no message is given — bare `clr` is always interactive.

---

### Parameter :: 7. `--new-session`

Disable the default session continuation. Normally `clr` passes
`-c` to claude on every invocation, resuming the most recent conversation.
`--new-session` suppresses that flag, starting a genuinely fresh session
with no prior context.

- **Type:** bool (standalone flag)
- **Default:** false (continuation is automatic)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)

```sh
clr --new-session "Analyse this new codebase from scratch"
clr --new-session "Review this PR fresh" --model opus
```

**Note:** Use when switching to a genuinely unrelated task where prior
conversation context would be misleading or harmful.

---

### Parameter :: 8. `--dir`

Set the working directory for the Claude Code subprocess. The runner
changes to this directory before invoking claude.

- **Type:** [`DirectoryPath`](types.md#type--2-directorypath)
- **Default:** current working directory
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)
- **Validation:** requires a value; `--dir` at end of argv → error

```sh
clr "Fix bug" --dir /path/to/project
```

**Note:** When `--dir` appears multiple times, the last value wins.

---

### Parameter :: 9. `--max-tokens`

Set the maximum number of output tokens for the Claude Code subprocess.
Passed via the `CLAUDE_MAX_OUTPUT_TOKENS` environment variable.

- **Type:** [`TokenLimit`](types.md#type--3-tokenlimit)
- **Default:** 200000
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)
- **Validation:** must be a valid u32 (0–4294967295); non-numeric → error

```sh
clr "Summarize" --max-tokens 50000
```

---

### Parameter :: 10. `--session-dir`

Override the session storage directory. Passed via the
`CLAUDE_SESSION_DIR` environment variable.

- **Type:** [`DirectoryPath`](types.md#type--2-directorypath)
- **Default:** — (Claude Code default)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)
- **Validation:** requires a value

```sh
clr "Fix bug" --session-dir /tmp/my-sessions
```

---

### Parameter :: 11. `--dry-run`

Print the assembled command that would be executed without actually
invoking the Claude Code subprocess. Useful for debugging flag
combinations.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)

```sh
clr --dry-run "test" --model sonnet --max-tokens 50000
# Output includes: claude --dangerously-skip-permissions -c --print --model sonnet "test"
```

---

### Parameter :: 12. `--verbosity`

Control how much diagnostic output the runner itself emits. Does not
affect Claude Code subprocess output.

- **Type:** [`VerbosityLevel`](types.md#type--5-verbositylevel)
- **Default:** 3 (normal)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)
- **Validation:** must be integer 0–5; out of range → error

```sh
clr --verbosity 0 "Silent run"    # suppress runner output
clr --verbosity 4 "Debug"         # verbose command preview
```

### Parameter :: 13. `--trace`

Print the assembled environment variables and command to stderr before executing the
Claude Code subprocess. Unlike `--dry-run`, execution still proceeds — the command is
shown as a diagnostic prefix, then the subprocess is launched. Mirrors shell `set -x`
semantics.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [Runner Control](parameter_groups.md#group--2-runner-control)

```sh
clr --trace "Fix bug"
# Stderr: CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000
# Stderr: claude --dangerously-skip-permissions -c --print "Fix bug"
# Then: subprocess executes normally
```

**Note:** `--trace` prints to stderr so it does not pollute captured stdout in print mode.
Combine with `--dry-run` if you want to preview without executing.

---

### Parameter :: 14. `--system-prompt`

Replace the default system prompt sent to the `claude` subprocess with a
custom text. When omitted, Claude's built-in system prompt remains in effect.

- **Type:** [`SystemPromptText`](types.md#type--6-systemprompttext)
- **Default:** — (built-in system prompt unchanged when absent)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [System Prompt](parameter_groups.md#group--3-system-prompt)
- **Validation:** requires a value; `--system-prompt` at end of argv → error

```sh
clr --system-prompt "You are a Rust expert. Be concise." "Review PR"
clr --dry-run --system-prompt "Be concise." "test"   # preview the flag
```

**What is preserved after replacement:** Tool definitions (~12,000 tokens covering
Bash, Read, Write, Edit, Glob, Grep, WebFetch, etc.) are injected into the assembled
prompt before the replacement is applied and survive intact. Claude can still call
all tools normally.

**What is lost after replacement:** The entire behavioral layer — Claude Code's coding
guidelines, git safety rules, security constraints, output style ("no emojis", conciseness),
CLAUDE.md-handling instructions, environment/project context, and sub-agent coordination
prompts. Claude has raw tool access but no guidance on when or how to use tools safely.

**Use case:** Specialized single-purpose agents that need complete control over behavior
and are prepared to re-specify everything Claude Code normally handles automatically.
For most use cases, `--append-system-prompt` is the correct choice.

---

### Parameter :: 15. `--append-system-prompt`

Append text to the default system prompt. Additive — does not replace the
built-in system prompt. When omitted, nothing is appended.

- **Type:** [`SystemPromptText`](types.md#type--6-systemprompttext)
- **Default:** — (nothing appended when absent)
- **Command:** [`run`](commands.md#command--1-run)
- **Group:** [System Prompt](parameter_groups.md#group--3-system-prompt)
- **Validation:** requires a value; `--append-system-prompt` at end of argv → error

```sh
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```

**Recommended over `--system-prompt` for most use cases.** All built-in Claude Code
behaviors are preserved — safety rules, CLAUDE.md handling, output style, tool usage
policies. The custom text is appended after the full default prompt.

**Precedence vs CLAUDE.md:** `--append-system-prompt` appends directly into the
*system prompt* (highest-priority position). `CLAUDE.md` is injected as the first
*user message* — a different, lower-priority mechanism. When both are active,
`--append-system-prompt` instructions have stronger persistence.

**Note:** Both `--system-prompt` and `--append-system-prompt` may be
given in the same invocation. Both are forwarded to claude in parse order.

---

### Quick Reference

**Required parameters:** `[MESSAGE]` is required for print mode (which is the default when a message is given).

**Most used parameters:** `--model` (model selection), `--dir` (project targeting), `--dry-run` (debugging), `--new-session` (fresh start), `--interactive` (TTY passthrough with prompt).

**Commands by parameter count:** `run` = 15 parameters, `help` = 0 parameters.
