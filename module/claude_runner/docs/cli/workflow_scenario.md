# Workflows

### All Workflows (9 total)

| # | Workflow | Primary Flags | Use Case |
|---|----------|---------------|----------|
| 1 | Interactive REPL | (none) | Exploratory development (continues session) |
| 2 | Print mode with message | `[MESSAGE]` | Default — captured output for scripting |
| 3 | Interactive with message | `--interactive` | TTY passthrough with initial prompt |
| 4 | Dry-run preview | `--dry-run` | Debugging flag combinations |
| 5 | Project-specific execution | `--dir`, `--session-dir` | Multi-project workflows |
| 6 | Verbose debugging | `--verbosity 4` | Troubleshooting runner behavior |
| 7 | Fresh session | `--new-session` | Starting a new unrelated task |
| 8 | Trace execution | `--trace` | See command on stderr then execute |
| 9 | Custom system prompt | `--system-prompt` | Domain-specific behavior injection |

**Total:** 9 workflows

---

### Workflow :: 1. Interactive REPL

Open the interactive REPL with no message. Session continues automatically.

```sh
# Open REPL (continues previous session)
clr

# Open REPL in a specific directory
clr --dir /path/to/project
```

---

### Workflow :: 2. Print Mode With Message

Provide a message and capture Claude's output. Print mode is the default
when a message is given — no `-p` needed.

```sh
# Capture output to variable
result=$(clr "List all TODO comments" --model sonnet)

# Pipe to file
clr "Generate changelog" > changelog.md

# Chain with other tools
clr "List failing tests" | grep FAIL

# Explicit -p (backward compat alias — identical behavior)
clr -p "Explain this function" --model sonnet
```

**Note:** Print mode without a message exits with error code 1.

---

### Workflow :: 3. Interactive With Message

Send an initial prompt and stay in interactive TTY mode. Use `--interactive`
to opt out of the default print behavior.

```sh
# TTY passthrough with initial prompt
clr --interactive "Fix the auth bug"

# Interactive, continue in specific directory
clr --interactive "Now fix the tests" --dir /path/to/project
```

---

### Workflow :: 4. Dry-run Preview

Inspect the assembled command without executing the subprocess.
The output always includes `-c` (automatic session continuation).

```sh
# See what would run (note: -c appears in output by default)
clr --dry-run "test" --model sonnet --max-tokens 50000

# Verify flag combinations before executing
clr --dry-run --verbose "Fix bug" --dir /project
# Then execute for real
clr "Fix bug" --dir /project
```

---

### Workflow :: 5. Project-specific Execution

Target a specific project directory and session storage location.

```sh
# Run in different project
clr "Run tests" --dir /path/to/project

# Custom session storage
clr "Fix bug" --dir /project --session-dir /tmp/sessions

# Interactive work in different project
clr --interactive "Continue fixing" --dir /path/to/project
```

---

### Workflow :: 6. Verbose Debugging

Increase runner diagnostic output to troubleshoot execution issues.

```sh
# Verbose: see command preview on stderr before execution
clr --verbosity 4 "Fix bug"

# Debug: see internal state, timing, paths
clr --verbosity 5 "Fix bug" --dir /project

# Silent: suppress all runner output
clr --verbosity 0 "Fix bug"
```

---

### Workflow :: 7. Fresh Session

Start a genuinely new conversation with no prior context.

```sh
# New unrelated task — discard previous session context
clr --new-session "Analyse the new payments module"

# Fresh start with model selection
clr --new-session "Review this PR from scratch" --model opus

# Fresh start in specific directory
clr --new-session "Onboard to this codebase" --dir /other/project
```

**Note:** Without `--new-session`, all invocations continue the previous session.
Use `--new-session` when switching to a task where prior conversation context
would be irrelevant or misleading.

---

### Workflow :: 8. Trace Execution

Print the assembled command to stderr and then execute. Use `--trace` when you want
to see exactly what `clr` is sending to claude without suppressing execution.
Mirrors shell `set -x` — the command is echoed before it runs.

```sh
# See what runs, then let it run
clr --trace "Fix bug" --model sonnet

# Trace a complex invocation (env vars + command both visible)
clr --trace "Run tests" --dir /path/to/project --max-tokens 50000

# Trace without session continuation
clr --trace --new-session "Analyse this from scratch"
```

**Note:** Output goes to stderr so captured stdout in print mode is unaffected.
For preview-only (no execution), use `--dry-run` instead.

---

### Workflow :: 9. Custom System Prompt

Narrow Claude's behavior for domain-specific automation by replacing or
extending the default system prompt.

`--system-prompt` replaces the built-in system prompt entirely — use when
you need full control over the model's behavioral context.
`--append-system-prompt` adds constraints on top of the default — use when
you want Claude's standard behavior plus domain-specific additions.

```sh
# Replace the default system prompt (strongest override)
clr --system-prompt "You are a Rust expert. Be concise." "Review this PR"

# Extend the default system prompt (lighter touch)
clr --append-system-prompt "Always respond in JSON." "List failing tests"

# Combine: replace then append
clr --system-prompt "You are a Rust expert." \
    --append-system-prompt "Be concise." \
    "Explain this trait"
```
