# Manual Testing Plan: clr CLI

## Prerequisites

- Claude Code binary in PATH: `which claude` returns a path
- API key configured: `ANTHROPIC_API_KEY` environment variable set
- Build binary: `cargo build -p claude_runner`
- Binary location: `target/debug/clr`

## Test Cases

### TC-1: Interactive REPL (No Args)
```sh
cargo run -p claude_runner
```

**Expected:** Claude opens interactive REPL session. TTY passthrough works — user can type prompts and receive responses. Exit with `/exit` or Ctrl-C.

### TC-2: Interactive with Message
```sh
cargo run -p claude_runner -- "What is 2+2?"
```

**Expected:** Claude starts interactive session with initial prompt. Responds with "4" or equivalent. Exit code 0.

### TC-3: Print Mode
```sh
cargo run -p claude_runner -- -p "What is 2+2?"
```

**Expected:** Claude responds with "4" or equivalent, printed to stdout. No interactive TUI. Exit code 0.

### TC-4: Automatic Session Continuation
```sh
cargo run -p claude_runner -- -p "Remember number 42"
cargo run -p claude_runner -- -p "What number did I tell you?"
```

**Expected:** Second invocation recalls "42" — session continues automatically (no explicit `-c` needed). Exit code 0 on both.

### TC-5: Working Directory
```sh
cargo run -p claude_runner -- -p "List files in this directory" --dir /tmp
```

**Expected:** Claude lists files in `/tmp`. Exit code 0.

### TC-6: Skip Permissions (Default On)
```sh
cargo run -p claude_runner -- -p "Run ls"
```

**Expected:** Claude executes without permission prompts (bypass is on by default). Exit code 0.

To explicitly disable the bypass:
```sh
cargo run -p claude_runner -- -p "Run ls" --no-skip-permissions
```

**Expected:** Claude prompts for tool approvals when needed. Exit code 0.

### TC-7: Dry Run (No Claude Required)
```sh
cargo run -p claude_runner -- --dry-run --dir /tmp "test"
```

**Expected:**
- Prints env var lines (`CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`, etc.)
- Prints: `cd /tmp`
- Prints: `claude --dangerously-skip-permissions --chrome --effort max --print -c "test\n\nultrathink"` (bypass, chrome, effort max, print, and `-c` appear automatically; ultrathink suffix added)
- Does NOT invoke Claude binary
- Exit code 0

### TC-8: Help Output
```sh
cargo run -p claude_runner -- --help
```

**Expected:** Prints USAGE, ARGUMENTS, OPTIONS sections. Exit code 0.

### TC-9: Error on Unknown Flag
```sh
cargo run -p claude_runner -- --nonexistent-flag
```

**Expected:** Prints error to stderr. Exit code 1.

### TC-10: Max Tokens Override
```sh
cargo run -p claude_runner -- --dry-run --max-tokens 50000 "hi"
```

**Expected:** Dry-run output shows `CLAUDE_CODE_MAX_OUTPUT_TOKENS=50000`.

### TC-11: Model Selection
```sh
cargo run -p claude_runner -- --dry-run --model claude-haiku-4-5-20251001 "hi"
```

**Expected:** Dry-run output shows `--model claude-haiku-4-5-20251001` in command.

### TC-12: Verbose Mode
```sh
cargo run -p claude_runner -- -p --verbose "test"
```

**Expected:** `--verbose` flag appears in the command passed to Claude. Exit code depends on Claude availability.

### TC-13: Session Directory
```sh
cargo run -p claude_runner -- --dry-run --session-dir /tmp/sessions "test"
```

**Expected:** Dry-run output shows `CLAUDE_CODE_SESSION_DIR=/tmp/sessions`. Exit code 0.

### TC-14: Bare Dry Run (No Message)
```sh
cargo run -p claude_runner -- --dry-run
```

**Expected:** Dry-run output ends with `claude --dangerously-skip-permissions --chrome --effort max -c` (default bypass, chrome, effort max, automatic continuation; no `--print` since no message). Exit code 0.

### TC-15: Duplicate Dir Flag (Last Wins)
```sh
cargo run -p claude_runner -- --dry-run --dir /tmp --dir /other "test"
```

**Expected:** Dry-run shows `cd /other` (last value wins). Exit code 0.

### TC-16: Negative Max Tokens
```sh
cargo run -p claude_runner -- --max-tokens -1 "test"
```

**Expected:** Error about invalid value. Exit code 1.

### TC-17: Missing Value for Flag
```sh
cargo run -p claude_runner -- --dry-run --dir
```

**Expected:** Error: "--dir requires a value". Exit code 1.

### TC-18: Print Without Message
```sh
cargo run -p claude_runner -- -p
```

**Expected:** Error: "--print requires a message argument". Exit code 1.

### TC-19: Double Dash Separator
```sh
cargo run -p claude_runner -- --dry-run -- --not-a-flag
```

**Expected:** `--not-a-flag` treated as message text, not a flag. Appears quoted in dry-run output.

### TC-20: Verbosity Levels
```sh
cargo run -p claude_runner -- --verbosity 0 --dry-run "test"
cargo run -p claude_runner -- --verbosity 3 --dry-run "test"
cargo run -p claude_runner -- --verbosity 5 --dry-run "test"
```

**Expected:**
- Verbosity 0: dry-run output shown on stdout (verbosity does NOT gate --dry-run output; see fix in dry_run_test.rs)
- Verbosity 3: normal dry-run output on stdout
- Verbosity 5: dry-run output on stdout (no crash)

### TC-21: Verbosity 4 Preview Before Execution
```sh
cargo run -p claude_runner -- --verbosity 4 -p "What is 1+1?"
```

**Expected:** Command preview printed to stderr before execution. Claude response on stdout.

### TC-22: Interactive with Model
```sh
cargo run -p claude_runner -- --model sonnet "Explain what Rust is in one sentence"
```

**Expected:** Interactive session with specified model. Claude responds.

### TC-23: Multiple Positional Words as Message
```sh
cargo run -p claude_runner -- --dry-run Fix the bug
```

**Expected:** Dry-run shows `"Fix the bug"` as the quoted message (all positional args joined).

### TC-24: New Session (No Continuation)
```sh
cargo run -p claude_runner -- --dry-run --new-session "Start fresh"
```

**Expected:** Dry-run output does NOT contain `-c` (automatic continuation suppressed by `--new-session`). Exit code 0.

### TC-25: System Prompt Override
```sh
cargo run -p claude_runner -- --dry-run --system-prompt "You are a Rust expert." "Explain lifetimes"
```

**Expected:** Dry-run output contains `--system-prompt` and `You are a Rust expert.` in the command line. Does NOT contain `--append-system-prompt`. Exit code 0.

### TC-26: Append System Prompt
```sh
cargo run -p claude_runner -- --dry-run --append-system-prompt "Be concise." "Explain lifetimes"
```

**Expected:** Dry-run output contains `--append-system-prompt` and `Be concise.` in the command line. Does NOT contain `--system-prompt`. Exit code 0.

### TC-27: Short Help Flag
```sh
cargo run -p claude_runner -- -h
```

**Expected:** Identical output to `--help`. Exit code 0.

### TC-28: Trace + Dry-Run — Dry-Run Wins; Stderr Empty
```sh
cargo run -p claude_runner -- --dry-run --trace "test" 2>/tmp/trace_err.txt; echo "stderr:"; cat /tmp/trace_err.txt
```

**Expected:** Dry-run output on stdout (env vars + command). Stderr is **empty** — `--dry-run` short-circuits before the `--trace` block fires, so no trace preview is emitted. Exit code 0.

(Note: `--trace` without `--dry-run` echoes the assembled command to stderr before invoking Claude. With `--dry-run` active, the early return means trace never runs.)

### TC-29: Trace Without Dry-Run — Preview on Stderr
```sh
cargo run -p claude_runner -- --trace "test" 2>/tmp/trace29_err.txt; echo "exit:$?"; echo "stderr:"; cat /tmp/trace29_err.txt
```

**Expected:** Command preview (env vars + command) written to stderr. Invocation attempt made (may fail if Claude binary absent). Exit code 0 on success, non-zero if Claude not found.

### TC-30: No-Skip-Permissions in Dry-Run
```sh
cargo run -p claude_runner -- --dry-run --no-skip-permissions "test"
```

**Expected:** Dry-run output does NOT contain `--dangerously-skip-permissions` (bypass disabled). Exit code 0.

### TC-31: All Flags Combined (Dry-Run)
```sh
cargo run -p claude_runner -- --dry-run --model claude-haiku-4-5-20251001 --max-tokens 10000 --dir /tmp --session-dir /tmp/s --system-prompt "Be brief." --new-session --trace "all flags"
```

**Expected:** All flags appear in dry-run output. No crash. `--dangerously-skip-permissions` present (default). Exit code 0.

### TC-32: Print + Dry-Run (With Message)
```sh
cargo run -p claude_runner -- --print --dry-run "test message"
```

**Expected:** Dry-run output on stdout contains `--print` in the command. Exit code 0.

### TC-33: Duplicate Flags — Last Wins (System Prompt)
```sh
cargo run -p claude_runner -- --dry-run --system-prompt "First." --system-prompt "Second." "test"
```

**Expected:** Dry-run output contains `Second.` (last value wins), not `First.`. Exit code 0.

### TC-34: Max Tokens Boundary (Zero)
```sh
cargo run -p claude_runner -- --dry-run --max-tokens 0 "test"
```

**Expected:** Dry-run output shows `CLAUDE_CODE_MAX_OUTPUT_TOKENS=0` (0 is a valid u32; no parse error). Exit code 0.

### TC-35: Max Tokens Boundary (One)
```sh
cargo run -p claude_runner -- --dry-run --max-tokens 1 "test"
```

**Expected:** Dry-run output shows `CLAUDE_CODE_MAX_OUTPUT_TOKENS=1`. Exit code 0.

### TC-36: No-Ultrathink Suppresses Suffix
```sh
cargo run -p claude_runner -- --dry-run --no-ultrathink "do something"
```

**Expected:** Dry-run output message is `do something` with no `ultrathink` suffix. Exit code 0.

### TC-37: Effort Level Override
```sh
cargo run -p claude_runner -- --dry-run --effort medium "test"
```

**Expected:** Dry-run output contains `--effort medium` (not `--effort max`). Exit code 0.

### TC-38: No-Effort-Max Suppresses Effort Flag
```sh
cargo run -p claude_runner -- --dry-run --no-effort-max "test"
```

**Expected:** Dry-run output contains NO `--effort` flag at all. Exit code 0.

Note: combining `--no-effort-max --effort medium` also produces no `--effort` flag — `--no-effort-max` suppresses the entire effort injection block.

### TC-39: No-Chrome Suppresses Chrome Flag
```sh
cargo run -p claude_runner -- --dry-run --no-chrome "test"
```

**Expected:** Dry-run output contains NO `--chrome` flag. Exit code 0.

### TC-40: No-Persist Adds Session-Persistence Flag
```sh
cargo run -p claude_runner -- --dry-run --no-persist "test"
```

**Expected:** Dry-run output contains `--no-session-persistence`. Exit code 0.

### TC-41: JSON Schema
```sh
cargo run -p claude_runner -- --dry-run --json-schema '{"type":"string"}' "test"
```

**Expected:** Dry-run output contains `--json-schema` and `{"type":"string"}`. Exit code 0.

### TC-42: MCP Config (Single)
```sh
cargo run -p claude_runner -- --dry-run --mcp-config /tmp/mcp.json "test"
```

**Expected:** Dry-run output contains `--mcp-config /tmp/mcp.json`. Exit code 0.

### TC-42b: MCP Config (Repeatable)
```sh
cargo run -p claude_runner -- --dry-run --mcp-config /tmp/a.json --mcp-config /tmp/b.json "test"
```

**Expected:** Dry-run output contains both `--mcp-config /tmp/a.json` and `--mcp-config /tmp/b.json`. Exit code 0.

### TC-43: Interactive Flag Suppresses Auto-Print
```sh
cargo run -p claude_runner -- --dry-run --interactive "message"
```

**Expected:** Dry-run output does NOT contain `--print` (interactive mode suppresses auto-print even when a message is given). Exit code 0.

## Pass Criteria

All TC-1 through TC-43 must pass without unexpected errors or panics.
TC-7 through TC-11, TC-13 through TC-20, TC-23 through TC-43 are runnable without a configured Claude API key.
TC-1 through TC-6, TC-12, TC-21, TC-22 require Claude binary and API key for full execution test.
