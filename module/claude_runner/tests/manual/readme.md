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
- Prints: `env -u CLAUDECODE claude --dangerously-skip-permissions --chrome --effort max --print "test\n\nultrathink"` (bypass, chrome, effort max, print, ultrathink suffix; `env -u CLAUDECODE` prefix from Feature 006; `-c` omitted because `/tmp` has no session history for this project per BUG-214 fix — `-c` appears only when `$HOME/.claude/projects/{encoded(dir)}/` is non-empty)
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

**Precondition:** Requires fewer than `--max-sessions` live claude sessions on the host. If the gate fires (e.g., 10/10 sessions running), the gate-wait message appears on stderr BEFORE the trace block runs — this is correct gate-before-trace ordering by design. Test in container where session count is 0 for reliable results.

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

### TC-44: `clr run help` Dispatches Help (BUG-215 regression guard)
```sh
cargo run -p claude_runner -- run help
```

**Expected:** Prints USAGE and exits 0 — identical to `clr help`. Does NOT invoke claude. Exit code 0.

**Note:** Before BUG-215 fix, `clr run help` stripped the `run` token but did not re-check for the `help` subcommand, causing "help" to be treated as a positional message and claude to be invoked.

### TC-45: `clr run ask` Routes to ask — Pure Alias Parity (BUG-213 regression guard)
```sh
cargo run -p claude_runner -- ask --dry-run "question"
cargo run -p claude_runner -- run --dry-run "question"
```

**Expected:** Both commands produce **identical** dry-run output — `--effort max`, `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`, `-c` continuation, `--dangerously-skip-permissions`, ultrathink suffix, `--chrome`. `ask` is a pure semantic alias for `run` since plan-007; all old ask-specific overrides (effort high, 16384 tokens, no `-c`, no skip-permissions) were removed. Exit code 0 on both.

### TC-46: Empty Session Dir — No `-c` Injected (BUG-214 regression guard)
```sh
TMPDIR=$(mktemp -d) && cargo run -p claude_runner -- --dry-run --session-dir "$TMPDIR" "test"
```

**Expected:** Dry-run output does NOT contain `-c` (empty session dir means no prior session to continue). Exit code 0.

### TC-47: Non-Empty Session Dir — `-c` Injected
```sh
cargo run -p claude_runner -- --dry-run --session-dir /tmp "test"
```

**Expected:** Dry-run output contains `-c` (non-empty `/tmp` means session history present). Exit code 0.

### TC-48: Output-File — Runner-Internal, Not Forwarded to Claude
```sh
cargo run -p claude_runner -- --dry-run --output-file /tmp/out.txt "test"
```

**Expected:** Dry-run output shows the assembled claude command without any `--output-file` flag (it's a runner option, not a claude flag). Exit code 0.

### TC-49: Expect — Runner-Level Validation Param
```sh
cargo run -p claude_runner -- --dry-run --expect "yes|no" "test"
```

**Expected:** Dry-run output shows assembled claude command without `--expect` forwarded to claude (runner option). Exit code 0.

### TC-50: Expect-Strategy — Valid and Invalid Values
```sh
# Valid: fail, retry, default:<val>
cargo run -p claude_runner -- --dry-run --expect "yes|no" --expect-strategy fail "test"
cargo run -p claude_runner -- --dry-run --expect "yes|no" --expect-strategy retry "test"
cargo run -p claude_runner -- --dry-run --expect "yes|no" --expect-strategy "default:yes" "test"

# Invalid value → exit 1
cargo run -p claude_runner -- --dry-run --expect "yes|no" --expect-strategy bogus "test"
```

**Expected:** First three exit 0. Last exits 1 with `Error: invalid --expect-strategy value: bogus`.

### TC-51: Expect-Retries — Range Validation
```sh
# Valid: 0-255
cargo run -p claude_runner -- --dry-run --expect "yes|no" --expect-strategy retry --expect-retries 3 "test"

# Out of range: 256 → exit 1
cargo run -p claude_runner -- --dry-run --expect "yes|no" --expect-strategy retry --expect-retries 256 "test"
```

**Expected:** First exits 0. Second exits 1 with `Error: invalid --expect-retries value: 256`.

### TC-52: Max-Sessions — Gate Disabled at 0
```sh
cargo run -p claude_runner -- --dry-run --max-sessions 0 "test"
cargo run -p claude_runner -- --dry-run --max-sessions 5 "test"
```

**Expected:** Both exit 0. Neither produces session-gate messages (dry-run bypasses actual execution). When `--max-sessions 0`, the gate is disabled entirely regardless of session count.

### TC-53: Retry-on-Rate-Limit Dry-Run
```sh
cargo run -p claude_runner -- --dry-run --retry-on-rate-limit 3 "test"
```

**Expected:** Exit 0. No retry messages on stderr (dry-run skips subprocess). The flag is parsed and accepted without error.

### TC-54: Retry-Delay Dry-Run
```sh
cargo run -p claude_runner -- --dry-run --retry-delay 30 "test"
```

**Expected:** Exit 0. Flag accepted without error.

### TC-55: Help Lists All Retry, Timeout, and Error Retry Flags
```sh
cargo run -p claude_runner -- --help
```

**Expected:** Help output contains `--retry-on-rate-limit`, `--retry-delay`, `--timeout`, `--retry-on-api-error`, `--api-error-delay`, and `--retry-on-unknown-error`. Exit 0.

### TC-56: CLR_RETRY_ON_RATE_LIMIT Env Var Accepted
```sh
CLR_RETRY_ON_RATE_LIMIT=2 cargo run -p claude_runner -- --dry-run "test"
```

**Expected:** Exit 0. Env var applied silently; no error.

### TC-57: Retry-on-Rate-Limit 0 — Explicit Disable (Overrides Default 1)
```sh
cargo run -p claude_runner -- --dry-run --retry-on-rate-limit 0 "test"
```

**Expected:** Exit 0. No retry logic invoked. `0` explicitly disables retry, overriding the default of `1`.

### TC-58: Timeout 0 (Unlimited Default)
```sh
cargo run -p claude_runner -- --dry-run --timeout 0 "test"
```

**Expected:** Exit 0. Unlimited mode; no watchdog engaged.

### TC-59: Timeout 30 Accepted in Dry-Run
```sh
cargo run -p claude_runner -- --dry-run --timeout 30 "test"
```

**Expected:** Exit 0. Watchdog param parsed but dry-run exits before subprocess is spawned.

### TC-60: CLR_TIMEOUT Env Var Accepted
```sh
CLR_TIMEOUT=10 cargo run -p claude_runner -- --dry-run "test"
```

**Expected:** Exit 0. Env var applied silently; no error.

### TC-61: `clr ps` — No Sessions (Container Only)
```sh
cargo run -p claude_runner -- ps
```

**Expected:** Prints `No active Claude Code sessions.` to stdout. Exit code 0. Must run in container where 0 `claude` processes exist.

### TC-62: `clr ps` — Sessions Present
```sh
cargo run -p claude_runner -- ps
```

**Expected:** Output begins with a titled caption rule line (e.g., `─── Active Sessions · 1 running ──────────────`). The column header row follows: `#`, `PID`, `Elapsed`, `CPU%`, `RAM`, `State`, `Absolute Path`, `Task`. Plain-style (no `┌` border). Exit code 0. Requires ≥1 live `claude` process.

### TC-63: `clr ps` — Self-Exclusion
```sh
cargo run -p claude_runner -- ps
```

**Expected:** The PID of the `clr ps` process itself does not appear as a row in the output table. Exit code 0.

### TC-64: `clr p` — Typo Guard
```sh
cargo run -p claude_runner -- p
```

**Expected:** stderr contains `Did you mean 'ps'?`. Exit code 1.

### TC-65: `clr ps --unknown` — Rejects Arguments
```sh
cargo run -p claude_runner -- ps --unknown
```

**Expected:** stderr error message about unexpected arguments. Exit code 1.

### TC-66: Retry-on-API-Error Dry-Run
```sh
cargo run -p claude_runner -- --dry-run --retry-on-api-error 3 "test"
```

**Expected:** Exit 0. Flag parsed and accepted without error.

### TC-67: API-Error-Delay Dry-Run
```sh
cargo run -p claude_runner -- --dry-run --api-error-delay 10 "test"
```

**Expected:** Exit 0. Flag parsed and accepted without error.

### TC-68: Retry-on-Unknown-Error Dry-Run
```sh
cargo run -p claude_runner -- --dry-run --retry-on-unknown-error 2 "test"
```

**Expected:** Exit 0. Flag parsed and accepted without error.

### TC-69: CLR_RETRY_ON_API_ERROR Env Var Accepted
```sh
CLR_RETRY_ON_API_ERROR=2 cargo run -p claude_runner -- --dry-run "test"
```

**Expected:** Exit 0. Env var applied silently; no error.

### TC-70: CLR_API_ERROR_DELAY Env Var Accepted
```sh
CLR_API_ERROR_DELAY=15 cargo run -p claude_runner -- --dry-run "test"
```

**Expected:** Exit 0. Env var applied silently; no error.

### TC-71: CLR_RETRY_ON_UNKNOWN_ERROR Env Var Accepted
```sh
CLR_RETRY_ON_UNKNOWN_ERROR=1 cargo run -p claude_runner -- --dry-run "test"
```

**Expected:** Exit 0. Env var applied silently; no error.

## Pass Criteria

All TC-1 through TC-71 must pass without unexpected errors or panics.
TC-7 through TC-11, TC-13 through TC-20, TC-23 through TC-65 are runnable without a configured Claude API key (except TC-61 requires container, TC-62/TC-63 require live sessions).
TC-1 through TC-6, TC-12, TC-21, TC-22 require Claude binary and API key for full execution test.

---

## Corner Cases (CC-1 through CC-112) — Automated

These are exhaustively tested by the integration test suite (not manual). Listed here for traceability.

### Parser

- **CC-1/2:** `--help` wins even when unknown flags precede it (BUG-221 regression)
- **CC-3/4:** `--effort invalid_level` → exit 1, error mentions "effort"
- **CC-5/6:** `--effort` without value → exit 1, missing-value error
- **CC-7/8:** `--effort low` and `--effort high` accepted
- **CC-9/10:** `--max-tokens 4294967296` (overflow) → exit 1, mentions "max-tokens"
- **CC-11/12:** `--max-tokens 1.5` and `--max-tokens ""` → exit 1
- **CC-13/14:** `--verbosity 6` → exit 1, mentions "verbosity"
- **CC-15/16:** `--verbosity 5` and `--verbosity 0` → accepted with `--dry-run`
- **CC-17/18:** `--subdir a/b` (slash) → exit 1, mentions "subdir"
- **CC-19:** `--subdir .` → identity (no `-prefix` join)
- **CC-20:** `--subdir ""` → identity (empty string filtered)
- **CC-21:** `--subdir mywork` → path contains `-mywork`
- **CC-22:** `--dir /tmp --subdir mywork` → `/tmp/-mywork`

### Env vars

- **CC-23:** `CLR_MAX_TOKENS=bad` → silently ignored (default preserved)
- **CC-24:** `CLR_VERBOSITY=6` → silently ignored (valid values: 0–5)
- **CC-25:** `CLR_EFFORT=invalid` → silently ignored (default max used)
- **CC-26:** `CLR_SUBDIR=a/b` → silently ignored (slash rejected)
- **CC-27:** `CLR_NEW_SESSION=1` → suppresses `-c`
- **CC-28:** `CLR_PRINT=1` without message → exit 1 ("--print requires a message")
- **CC-29:** `CLR_PRINT=1` with message → `--print` in output
- **CC-30:** `CLR_INTERACTIVE=1` → suppresses auto `--print`
- **CC-31:** `CLR_MCP_CONFIG=...` without CLI flag → used
- **CC-32/32b:** CLI `--mcp-config` wins over `CLR_MCP_CONFIG`

### Empty/whitespace messages

- **CC-33:** `clr ""` → empty arg filtered → no `--print`
- **CC-34:** `clr -- ""` → empty after `--` filtered → no `--print`
- **CC-35:** `clr " "` → whitespace-only IS a valid message → `--print` added

### Flag interactions

- **CC-36:** Message already ending in "ultrathink" → no double suffix (idempotent)
- **CC-37/38:** `--no-effort-max` wins over `--effort medium` regardless of order
- **CC-39/39b:** Duplicate `--system-prompt` → last value wins
- **CC-40:** `--system-prompt` + `--append-system-prompt` together → both appear
- **CC-41:** `--session-dir /nonexistent` → no `-c`
- **CC-42:** `--session-dir /path/to/file` (not a dir) → no `-c`

### Subcommand help

- **CC-43–48:** `isolated/refresh/ask --help` and `-h` each exit 0
- **CC-49–51:** Help output contains expected keywords

### Error cases

- **CC-52–55:** `refresh/isolated --unknown-flag` → exit 1, "unknown option"
- **CC-56–60:** Invalid `--timeout` values (`-1`, `abc`) → exit 1, mentions "timeout"

### Typo guard

- **CC-61:** `rn` (2 chars) → typo guard fires, suggests `run` (first char 'r' matches, Levenshtein 1)
- **CC-62–64:** `isol`, `refre`, `askk` → typo guard fires, suggests correct subcommand
- **CC-65:** `hel` (3 chars) → typo guard fires, suggests `help` (`"help".starts_with("hel")`)
- **CC-65b/65c:** `helpx`, `runn` → typo guard fires

### Subcommand edge cases

- **CC-66:** `clr refresh some_word --creds ...` → positional silently ignored, no parse error
- **CC-67:** `clr ask --dry-run` (no message) → no `--print`
- **CC-68:** `clr ask --dry-run test` == `clr run --dry-run test` stdout-identical (pure alias T01)
- **CC-69:** `clr ask --dry-run test` → has `--dangerously-skip-permissions` (pure alias — no suppression)
- **CC-70:** `clr ask --dry-run test` → does NOT have `--no-session-persistence` (pure alias — no injection)
- **CC-71:** `clr ask --dry-run test` → has ultrathink suffix (pure alias — no suppression)
- **CC-72:** `clr ask --dry-run test` → has `--chrome` (pure alias — no suppression)
- **CC-73:** `clr ask --dry-run test` → `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000` (pure alias — not 16384)
- **CC-74:** `clr ask help` (positional) → shows ask help, exits 0 (BUG-249 regression guard)
- **CC-75:** `clr ask --effort high --dry-run test` → has `--effort high` (explicit override respected)
- Automated in: `ask_command_test.rs` T01–T11

### BUG-245 (CLR_EFFORT/CLR_MAX_TOKENS in ask mode)

- **CC-79:** `CLR_EFFORT=low clr ask` → env var applied (was broken before fix when ask had soft defaults)
- Equivalent test: `CLR_MAX_TOKENS=50000 clr ask` → overrides default 200000
- Automated in: `it_11_clr_effort_env_overrides_ask_default`, `it_12_clr_max_tokens_env_overrides_ask_default`

### New features: output-file, expect, expect-strategy, expect-retries, max-sessions

- **CC-80:** `--output-file /tmp/out.txt --dry-run "test"` → exit 0; runner option not forwarded to claude
- **CC-81:** `--expect "yes|no" --dry-run "test"` → exit 0; expect is runner-level, dry-run exits before validation
- **CC-82:** `--expect-strategy fail --dry-run "test"` → exit 0; runner option, no effect on dry-run
- **CC-83:** `--expect-strategy retry --dry-run "test"` → exit 0
- **CC-84:** `--expect-strategy "default:yes" --dry-run "test"` → exit 0
- **CC-85:** `--expect-strategy bogus --dry-run "test"` → exit 1; error "invalid --expect-strategy value"
- **CC-86:** `--expect-retries 3 --dry-run "test"` → exit 0
- **CC-87:** `--expect-retries 256 --dry-run "test"` → exit 1; error "invalid --expect-retries value"
- **CC-88:** `--max-sessions 5 --dry-run "test"` → exit 0
- **CC-89:** `--max-sessions 0 --dry-run "test"` → exit 0 (gate disabled)
- **CC-90:** `CLR_MAX_SESSIONS=notanumber --dry-run "test"` → exit 0 (silently ignored, default 30 used)
- Automated in: `output_file_test.rs`, `expect_validation_test.rs`, `param_edge_cases_test.rs`, `env_var_ext_test.rs`

### Env vars for expect/output-file params

- **CC-91:** `CLR_OUTPUT_FILE=/tmp/x.txt --dry-run "test"` → exit 0; runner-level, not forwarded to claude command
- **CC-92:** `CLR_EXPECT="yes|no" --dry-run "test"` → exit 0; runner-level, not forwarded
- **CC-93:** `CLR_EXPECT_STRATEGY=fail --dry-run "test"` → exit 0
- **CC-94:** `CLR_EXPECT_STRATEGY=bogus --dry-run "test"` → exit 1 with error "CLR_EXPECT_STRATEGY: invalid"
- **CC-95:** `CLR_EXPECT_RETRIES=5 --dry-run "test"` → exit 0
- **CC-96:** `CLR_EXPECT_RETRIES=256 --dry-run "test"` → exit 1 with error "CLR_EXPECT_RETRIES: invalid"

### expect-strategy edge cases

- **CC-97:** `--expect-strategy "default:" --dry-run "test"` → exit 0; empty-value default is valid (returns `""` on mismatch)
- **CC-98:** `--expect "yes" --expect-strategy fail --expect-retries 3 --dry-run "test"` → exit 0; retries silently ignored when strategy is `fail`

### Runner-level flags not forwarded to claude

- **CC-99:** `--file /etc/hostname --dry-run "test"` → dry-run shows `< /etc/hostname` as stdin redirect, NOT `--file` flag
- **CC-100:** `--strip-fences --dry-run "test"` → dry-run shows no `--strip-fences` in claude command (runner post-processing)
- **CC-101:** `--keep-claudecode --dry-run "test"` → dry-run shows `claude ...` WITHOUT `env -u CLAUDECODE` prefix
- Automated in: `user_story_output_test.rs`, `env_var_ext_test.rs`, `fence_test.rs`

### New features: retry-on-rate-limit, retry-delay, timeout (run/ask)

- **CC-102:** `--retry-on-rate-limit 256 --dry-run "test"` → exit 1; error "invalid --retry-on-rate-limit value: 256" (u8 overflow)
- **CC-103:** `CLR_RETRY_ON_RATE_LIMIT=abc --dry-run "test"` → exit 0 (silently ignored; invalid env var values are non-fatal)
- **CC-104:** `CLR_RETRY_DELAY=abc --dry-run "test"` → exit 0 (silently ignored)
- **CC-105:** `CLR_TIMEOUT=abc --dry-run "test"` → exit 0 (silently ignored)
- **CC-106:** `--retry-on-rate-limit 0 --retry-delay 60 --dry-run "test"` → exit 0 (delay ignored when retry count is 0)
- **CC-107:** `--timeout 4294967295 --dry-run "test"` → exit 0 (u32 max accepted)
- **CC-108:** `--retry-on-rate-limit 255 --dry-run "test"` → exit 0 (u8 max accepted)
- **CC-109:** `--retry-on-rate-limit` (missing value) → exit 1; error "requires a value"
- **CC-110:** `--retry-delay` (missing value) → exit 1; error "requires a value"
- **CC-111:** `--timeout` (missing value, run/ask) → exit 1; error "requires a value"
- **CC-112:** `clr ask --retry-on-rate-limit 3 --dry-run "q"` == `clr run --retry-on-rate-limit 3 --dry-run "q"` (pure alias parity)
- Automated in: `retry_rate_limit_test.rs`, `timeout_test.rs`

### New features: retry-on-api-error, api-error-delay, retry-on-unknown-error

- **CC-113:** `--retry-on-api-error 256 --dry-run "test"` → exit 1; error "invalid --retry-on-api-error value: 256" (u8 overflow)
- **CC-114:** `--retry-on-api-error 255 --dry-run "test"` → exit 0 (u8 max accepted)
- **CC-115:** `--retry-on-api-error` (missing value) → exit 1; error "requires a value"
- **CC-116:** `CLR_RETRY_ON_API_ERROR=abc --dry-run "test"` → exit 0 (silently ignored)
- **CC-117:** `--api-error-delay 4294967296 --dry-run "test"` → exit 1 (u32 overflow)
- **CC-118:** `--api-error-delay 4294967295 --dry-run "test"` → exit 0 (u32 max accepted)
- **CC-119:** `--api-error-delay` (missing value) → exit 1; error "requires a value"
- **CC-120:** `CLR_API_ERROR_DELAY=abc --dry-run "test"` → exit 0 (silently ignored)
- **CC-121:** `--retry-on-unknown-error 256 --dry-run "test"` → exit 1; error "invalid --retry-on-unknown-error value: 256" (u8 overflow)
- **CC-122:** `--retry-on-unknown-error 255 --dry-run "test"` → exit 0 (u8 max accepted)
- **CC-123:** `--retry-on-unknown-error` (missing value) → exit 1; error "requires a value"
- **CC-124:** `CLR_RETRY_ON_UNKNOWN_ERROR=abc --dry-run "test"` → exit 0 (silently ignored)
- **CC-125:** `clr ask --retry-on-api-error 1 --dry-run "q"` == `clr run --retry-on-api-error 1 --dry-run "q"` (pure alias parity)
- Automated in: `retry_api_error_test.rs`, `retry_unknown_error_test.rs`

---

## New Corner Cases (NC-1 through NC-15) — Discovered During Manual Testing

### NC-1: QuotaExhausted Label (Automated)

`clr run` against a fake script that exits 2 with "Usage limit reached" in stdout → stderr contains "quota exhausted" label.
Automated in: `error_classification_test.rs::quota_exhausted_pattern_emits_labeled_message`.

### NC-2: `--keep-claudecode` Warning Suppressed at Low Verbosity

```sh
CLAUDECODE=1 cargo run -p claude_runner -- --keep-claudecode --verbosity 0 --dry-run "test"
```

**Expected:** No warning on stderr (verbosity 0 suppresses BUG-248 warning). Exit code 0.

### NC-3: `--keep-claudecode` Warning Fires at Verbosity ≥ 2

```sh
CLAUDECODE=1 cargo run -p claude_runner -- --keep-claudecode --verbosity 2 --dry-run "test"
```

**Expected:** Warning on stderr: `Warning: CLAUDECODE is set in environment...`. Exit code 0.

### NC-4: `--keep-claudecode` Warning Fires Even in Dry-Run (Verbosity ≥ 2)

```sh
CLAUDECODE=1 cargo run -p claude_runner -- --keep-claudecode --verbosity 2 --dry-run "test"
```

**Expected:** Warning fires on stderr AND dry-run output on stdout. Exit code 0. Confirms BUG-248 fix fires before dry-run short-circuit.

### NC-5: g2cc4 Host Fragility — CLAUDECODE Inherited From Shell

`param_group_test::g2cc4_all_runner_control_flags_no_conflict` uses `--keep-claudecode --verbosity 2`. When run inside a Claude Code session on the host, `CLAUDECODE` is inherited from the outer process environment, causing the BUG-248 warning to fire and breaking the `stderr.is_empty()` assertion.

Fix: test explicitly calls `.env_remove("CLAUDECODE")` to enforce CC-4 "clean environment" precondition. Automated in: `param_group_test.rs::g2cc4`.

### NC-6: Live End-to-End Print Mode (`clr ask hello`)

```sh
clr ask hello
```

**Expected:** Returns a real Claude response (e.g., "Hey. What are we working on?"), exits 0. Confirms full round-trip: arg parsing → env setup → claude spawn → stdout capture → exit propagation. This is the live equivalent of TC-3.

### NC-7: Orphaned Import `use super::VerbosityLevel` in `src/cli/mod.rs`

After `run_interactive` signature changed from `_verbosity: VerbosityLevel` to `cli: &CliArgs`, the import became unused. Clippy fired: `unused import: use super::VerbosityLevel`. Removed the orphaned import. Automated regression: `RUSTFLAGS="-D warnings" cargo nextest run` would have caught it.

### NC-8: Clippy Lints in New Test Files (`retry_rate_limit_test.rs`, `timeout_test.rs`)

Three categories of clippy errors found when running Level 3 (`-D warnings`):
1. `u32 as u64` casts in `src/cli/mod.rs` — 3 occurrences; fixed with `u64::from(x)` (cast_lossless lint)
2. `std::time::Duration` instead of `core::time::Duration` — 5 occurrences in `src/cli/mod.rs` (std_instead_of_core lint)
3. `doc_markdown` errors — 17 in `retry_rate_limit_test.rs`, 8 in `timeout_test.rs`; bare identifiers (`CLR_RETRY_ON_RATE_LIMIT`, `QuotaExhausted`, `classify_error()`, `ERROR_PATTERNS`, `RateLimit`, `CLR_RETRY_DELAY`, `CLR_TIMEOUT`, `spawn_piped`, `try_wait`) in `///` and `//!` doc comments needed backtick wrapping.

Root cause: new test files written without running full clippy sweep. Prevention: run Level 3 immediately after adding doc comments in test files.

### NC-9: `clr isolated` Without `--creds` Auto-Detects Default Credentials

```sh
clr isolated "some task"
```

**Expected:** No "creds required" error. `apply_cred_env_vars` falls back to `ClaudePaths::new().credentials_file()` (`~/.claude/.credentials.json`). If the file exists the subprocess runs; if not, exits 1 with "cannot read credentials file". `--creds` is listed as `(required)` in help, meaning credentials are required in some form — not that the CLI flag is mandatory.

### NC-10: `clr refresh` Without `--creds` On Machine With Default Credentials

```sh
clr refresh
```

**Expected:** If `~/.claude/.credentials.json` exists: subprocess runs with `--print "."`, Claude responds "." (ISOLATED_CLAUDE_MD instruction: single-char input → reply with "."), exits 0. Uses real API credits. Confirms auto-detection path works end-to-end.

### NC-11: `clr isolated --trace --creds /nonexistent "msg"` — Trace Fires Before Error

```sh
clr isolated --trace --creds /nonexistent "test"
```

**Expected:** Trace printed to stderr first (`# clr isolated`, `# creds: /nonexistent`, command preview), THEN `Error: cannot read credentials file '/nonexistent'`. Exit 1. Trace fires before any I/O (from `emit_credential_trace` being called before `read_to_string`).

### NC-12: Gate Waiting Message Format — `X/Y sessions active`

**Precondition:** Requires ≥30 live claude sessions running on the host (or use `--max-sessions N` with N sessions already running). Gate-blocked: cannot be tested in container (0 sessions).

**Expected:** When the gate is triggered, each polling cycle emits to stderr:
`Info: {count}/{max} sessions active; waiting 30s for a slot... (attempt {attempt}/{max_attempts})`

Example with 30 sessions at default limit:
`Info: 30/30 sessions active; waiting 30s for a slot... (attempt 1/100)`

The old format `"X claude session(s) running (limit Y)"` is **not** emitted. The `X/Y` ratio format is the canonical output.

### NC-13: Gate Exhaustion After 100 Attempts

**Precondition:** Same as NC-12. Gate must fire and never find a free slot.

**Expected:** After 100 polling cycles (50 minutes total), `clr` emits to stderr:
`Error: --max-sessions {count}/{max} active; gave up after 100 attempts.`
Then exits with code 1. The old limit of 50 attempts is **not** used.

### NC-14: `clr ps` — Queued CLR Table via `CLR_GATE_DIR`

```sh
mkdir -p /tmp/test-gate
printf '{"cwd":"/tmp/myproject","since":1720000000,"attempt":3,"message":"waiting for session slot"}' \
  > /tmp/test-gate/99999.json
CLR_GATE_DIR=/tmp/test-gate cargo run -p claude_runner -- ps
rm -rf /tmp/test-gate
```

**Expected:** "No active Claude Code sessions." message appears first, then a blank line, then the queued table. The queued table begins with a titled caption rule line (e.g., `─── Queued · 1 waiting ──────────────`), followed by column headers `PID`, `CWD`, `Waiting`, `Attempt`. PID column shows `99999`. `Waiting` shows a large elapsed value (epoch 1720000000 is in 2024, so format is `Xh Ym`). Exit code 0. No live `claude` sessions required — works in container.

### NC-15: `clr kill` — Live Claude Session Termination

**Precondition:** At least one live `clr run` or `clr ask` session must be running. Use `clr ps` to find its PID.

```sh
clr ps                     # note a PID from the output
clr kill <PID>
```

**Expected:** `clr kill <PID>` exits 0; stdout contains `"Sent SIGTERM to Claude Code session <PID>."`. The targeted session terminates (verify with a follow-up `clr ps`). No other sessions are affected. Automated analog: `kill_command_test.rs::it_04_successful_sigterm_delivery` (uses fake `claude` ELF process; confirms same code path).

