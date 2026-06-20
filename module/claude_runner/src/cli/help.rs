/// Print help for the main `clr` command to stdout.
pub( crate ) fn print_help()
{
  println!( "clr — Execute Claude Code with configurable parameters" );
  println!();
  println!( "USAGE:" );
  println!( "  clr [OPTIONS] [MESSAGE]" );
  println!( "  clr run      [OPTIONS] [MESSAGE]" );
  println!( "  clr ask      [OPTIONS] [QUESTION]" );
  println!( "  clr isolated --creds <FILE> [--timeout <SECS>] [--trace] [MESSAGE]" );
  println!( "  clr refresh  --creds <FILE> [--timeout <SECS>] [--trace]" );
  println!( "  clr kill     <PID>" );
  println!( "  clr tools" );
  println!( "  clr help" );
  println!();
  println!( "COMMANDS:" );
  // Fix(BUG-212): `run` was absent from COMMANDS despite being a valid explicit subcommand.
  // Root cause: print_help() only listed ask/isolated/refresh/help; discoverability AC violated.
  // Pitfall: `clr run` must strip the leading token before reaching the parser — see lib.rs.
  println!( "  run                                Execute Claude Code with configurable parameters (default mode)" );
  println!( "  ask                                Semantic alias for `run` (identical behavior)" );
  println!( "  isolated                           Run Claude with credential-isolated temp HOME" );
  println!( "  refresh                            Refresh OAuth credentials without running a task" );
  println!( "  ps                                 List running Claude Code sessions" );
  println!( "  kill                               Terminate a running Claude Code session by PID" );
  println!( "  tools                              List all Claude Code built-in tools" );
  println!( "  help                               Print usage information and exit" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  [MESSAGE]                          Prompt message for Claude" );
  println!();
  println!( "OPTIONS:" );
  println!( "  -p, --print                        Non-interactive mode (capture and print output)" );
  println!( "  --interactive                      Force interactive mode even when a message is given" );
  println!( "  --new-session                      Start a new session (default: continues previous)" );
  println!( "  --model <MODEL>                    Model to use" );
  println!( "  --verbose                          Enable verbose output" );
  println!( "  --no-skip-permissions              Disable automatic permission bypass (on by default)" );
  println!( "  --max-tokens <N>                   Max output tokens (default: 200000)" );
  println!( "  --session-dir <PATH>               Session storage directory" );
  println!( "  --dir <PATH>                       Working directory" );
  println!( "  --subdir <NAME>                    Named subdirectory appended to --dir as /-NAME; . = identity" );
  println!( "  --dry-run                          Print command without executing" );
  println!( "  --trace                            Print command to stderr then execute (like set -x)" );
  println!( "  --system-prompt <TEXT>             Set system prompt (replaces the default)" );
  println!( "  --append-system-prompt <TEXT>      Append text to the default system prompt" );
  println!( "  --no-ultrathink                    Disable automatic \"\\n\\nultrathink\" message suffix" );
  println!( "  --effort <LEVEL>                   Reasoning effort: low, medium, high, max (default: max)" );
  println!( "  --no-effort-max                    Suppress default --effort max injection" );
  println!( "  --no-chrome                        Suppress default --chrome injection" );
  println!( "  --no-persist                       Disable session persistence (--no-session-persistence)" );
  println!( "  --json-schema <SCHEMA>             JSON schema for structured output" );
  println!( "  --mcp-config <PATH>                MCP server config file (repeatable)" );
  println!( "  --output-format <FMT>              Output format: text, json, stream-json, summary" );
  println!( "  --max-turns <N>                    Max agentic turns (0 = unlimited)" );
  println!( "  --allowed-tools <TOOLS>            Comma-separated tool whitelist (e.g. \"Read,Edit\")" );
  println!( "  --disallowed-tools <TOOLS>         Comma-separated tool blacklist" );
  println!( "  --max-budget-usd <AMOUNT>          Max API spend in USD for this session" );
  println!( "  --add-dir <PATH>                   Additional directory Claude may access" );
  println!( "  --fallback-model <MODEL>           Fallback model when primary is unavailable" );
  println!( "  --file <PATH>                      Pipe file content to subprocess stdin" );
  println!( "  --strip-fences                     Strip outermost markdown code fences from stdout" );
  println!( "  --keep-claudecode                  Preserve CLAUDECODE env var in subprocess (default: removed)" );
  println!( "  --verbosity <0-5>                  Runner output verbosity level (default: 3)" );
  println!( "  --output-file <PATH>               Write captured output to file (tee: stdout + file)" );
  println!( "  --expect <VALS>                    Pipe-separated expected values; mismatch → exit 3 (case-insensitive, trimmed)" );
  println!( "  --expect-strategy <STRAT>          Mismatch handling: fail (default), retry, default:<VAL>" );
  println!( "  --max-sessions <N>                 Max concurrent claude sessions before blocking (0=unlimited, default: 30)" );
  println!( "  --timeout <SECS>                   Kill subprocess after N seconds (0 = unlimited, default: 0)" );
  println!();
  println!( "RETRY OPTIONS (3-tier: override > class-specific > fallback):" );
  println!( "  --retry-override <N>               Force retry count for all error classes (0–255; unset = use per-class)" );
  println!( "  --retry-override-delay <SECS>      Force delay for all error classes (unset = use per-class)" );
  println!( "  --retry-default <N>                Fallback retry count for unset classes (0–255, default: 2)" );
  println!( "  --retry-default-delay <SECS>       Fallback delay for unset classes (default: 30; 0 = immediate)" );
  println!( "  --retry-on-transient <N>           Transient (rate limit) retry count (default: auto → fallback)" );
  println!( "  --transient-delay <SECS>           Transient class delay (default: auto → fallback)" );
  println!( "  --retry-on-account <N>             Account (quota exhausted) retry count (default: 0; opt-in only)" );
  println!( "  --account-delay <SECS>             Account class delay (default: auto → fallback)" );
  println!( "  --retry-on-auth <N>                Auth (credential) retry count (default: auto → fallback)" );
  println!( "  --auth-delay <SECS>                Auth class delay (default: auto → fallback)" );
  println!( "  --retry-on-service <N>             Service (API error) retry count (default: auto → fallback)" );
  println!( "  --service-delay <SECS>             Service class delay (default: auto → fallback)" );
  println!( "  --retry-on-process <N>             Process (signal/timeout) retry count (default: auto → fallback)" );
  println!( "  --process-delay <SECS>             Process class delay (default: auto → fallback)" );
  println!( "  --retry-on-validation <N>          Validation (--expect mismatch) retry count (default: auto → fallback)" );
  println!( "  --validation-delay <SECS>          Validation class delay (default: auto → fallback)" );
  println!( "  --retry-on-runner <N>              Runner (infrastructure) retry count (default: auto → fallback)" );
  println!( "  --runner-delay <SECS>              Runner class delay (default: auto → fallback)" );
  println!( "  --retry-on-unknown <N>             Unknown error retry count (default: auto → fallback)" );
  println!( "  --unknown-delay <SECS>             Unknown class delay (default: auto → fallback)" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "CREDENTIAL OPTIONS (isolated, refresh):" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait (default: 30 isolated, 45 refresh)" );
  println!( "  --trace                            Print creds path, timeout, and claude invocation to stderr" );
}

/// Print help for the `isolated` subcommand and exit 0.
///
/// Called when `parse_isolated_args` encounters `-h` or `--help`.
/// Terminates the process via `std::process::exit(0)` so the caller
/// never needs to handle a return value.
pub( crate ) fn print_isolated_help() -> !
{
  println!( "clr isolated — Run Claude Code with credential-isolated temp HOME" );
  println!();
  println!( "USAGE:" );
  println!( "  clr isolated --creds <FILE> [--timeout <SECS>] [MESSAGE] [-- PASSTHROUGH...]" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  [MESSAGE]                          Prompt message for Claude" );
  println!();
  println!( "CREDENTIAL OPTIONS:" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait for subprocess (default: 30)" );
  println!( "  --trace                            Print underlying call details to stderr" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Success" );
  println!( "  1    Error (bad arguments, subprocess failure)" );
  println!( "  2    Timeout — subprocess did not finish within --timeout seconds" );
  std::process::exit( 0 );
}

/// Print help for the `refresh` subcommand and exit 0.
pub( crate ) fn print_refresh_help() -> !
{
  println!( "clr refresh — Refresh OAuth credentials without running a task" );
  println!();
  println!( "USAGE:" );
  println!( "  clr refresh --creds <FILE> [--timeout <SECS>] [--trace]" );
  println!();
  println!( "CREDENTIAL OPTIONS:" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait for refresh (default: 45)" );
  println!( "  --trace                            Print underlying call details to stderr" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Credentials were refreshed and written back" );
  println!( "  1    Error (bad arguments, no refresh occurred, subprocess failure)" );
  println!( "  2    Timeout — subprocess did not finish within --timeout seconds" );
  std::process::exit( 0 );
}

/// Print help for the `ps` subcommand and exit 0.
pub( crate ) fn print_ps_help() -> !
{
  println!( "clr ps — List running Claude Code sessions" );
  println!();
  println!( "USAGE:" );
  println!( "  clr ps [OPTIONS]" );
  println!();
  println!( "OPTIONS:" );
  println!( "  -m, --mode <MODE>                   Filter rows: all (default), print, interactive" );
  println!( "      --columns <COLS>                Comma-separated column keys (overrides --wide)" );
  println!( "  -w, --wide                          Show all 11 columns" );
  println!( "      --pid <PIDs>                    Comma-separated PIDs; restrict active table to matching sessions" );
  println!( "  -i, --inspect                       Show all 12 attributes per session in key:value format" );
  println!( "  -h, --help                          Show this help and exit" );
  println!();
  // Fix(BUG-303): key names must match COLUMN_KEYS constant (idx, cmd — not num, command).
  // Root cause: print_ps_help() was authored with num/command names diverging from
  //   COLUMN_KEYS (idx/cmd); user-visible --help showed invalid --columns keys.
  // Pitfall: COLUMN_KEYS and print_ps_help() are separate sources with no compile-time link —
  //   renaming a column key must update both; EC-10 regression test guards this.
  println!( "COLUMN KEYS (for --columns):" );
  println!( "  idx        #              Row index" );
  println!( "  pid        PID            Process ID" );
  println!( "  elapsed    Elapsed        Time since session started" );
  println!( "  cpu        CPU%           CPU usage percentage" );
  println!( "  ram        RAM            Resident memory (K or M suffix)" );
  println!( "  state      State          Process state (R/S/Z…)" );
  println!( "  path       Absolute Path  Working directory (\\$PRO prefix shortened when PRO is set)" );
  println!( "  task       Task           Last human message extracted from session JSONL (≤35 chars)" );
  println!( "  mode       Mode           Execution mode: print or interactive" );
  println!( "  cmd        Command        Arguments passed after the binary (flags and values)" );
  println!( "  binary     Binary         Full executable path of the claude binary" );
  println!();
  println!( "DEFAULT COLUMNS: idx, pid, elapsed, cpu, ram, state, mode, path, task" );
  println!();
  println!( "QUEUED CLR PROCESSES TABLE (shown when gate files exist in CLR_GATE_DIR):" );
  println!( "  #                                  Row index" );
  println!( "  PID                                Process ID of the waiting clr process" );
  println!( "  CWD                                Working directory of the waiting process" );
  println!( "  Waiting                            Time spent waiting for a session slot" );
  println!( "  Attempt                            Number of polling attempts so far" );
  println!();
  println!( "SESSION FLAGS (auto-inserted Flags column when ≥1 flag fires):" );
  println!( "  👈  This session    Parent of clr ps is a claude process" );
  println!( "  🖨   Print mode      Session cmdline contains --print or -p" );
  println!( "  ⚡  Running         Kernel process state == R" );
  println!( "  🕰   Ancient         elapsed_secs > CLR_PS_ANCIENT_SECS (default: 28800 = 8 h)" );
  println!( "  🐘  High RAM        RAM > CLR_PS_HIGH_RAM_MB (default: 400 MB)" );
  println!( "  ⚠   Dead metrics    /proc stat unreadable (all metric fields show -)" );
  println!( "  🐳  Container       Session cwd does not start with \\$HOME" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Success (table printed, inspect blocks printed, or empty-state message shown)" );
  println!( "  1    Error (unexpected argument, invalid --mode value, unknown column key, or non-numeric --pid value)" );
  println!();
  println!( "ENVIRONMENT:" );
  println!( "  CLR_PS_MODE                        Default value for --mode" );
  println!( "  CLR_PS_COLUMNS                     Default value for --columns" );
  println!( "  CLR_PS_PID                         Default value for --pid (comma-separated PIDs)" );
  println!( "  CLR_PS_ANCIENT_SECS                Seconds threshold for 🕰 Ancient flag (default: 28800)" );
  println!( "  CLR_PS_HIGH_RAM_MB                 RAM-MB threshold for 🐘 High RAM flag (default: 400)" );
  println!( "  CLR_GATE_DIR                       Directory for CLR gate state files (default: /tmp/clr-gate)" );
  std::process::exit( 0 );
}

/// Print help for the `ask` subcommand and exit 0.
///
/// Called when `--help` or `-h` is detected in `dispatch_ask` before delegating to `dispatch_run`.
pub( crate ) fn print_ask_help() -> !
{
  println!( "clr ask — Semantic alias for `clr run`" );
  println!();
  println!( "USAGE:" );
  println!( "  clr ask [OPTIONS] [QUESTION]" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  [QUESTION]                         Question to ask Claude" );
  println!();
  println!( "`clr ask` is a pure semantic alias for `clr run` — all options are identical." );
  println!( "See `clr --help` or `clr run --help` for the full option list." );
  println!();
  println!( "OPTIONS:" );
  println!( "  -p, --print                        Non-interactive mode (capture and print output)" );
  println!( "  --effort <LEVEL>                   Reasoning effort: low, medium, high, max (default: max)" );
  println!( "  --max-tokens <N>                   Max output tokens (default: 200000)" );
  println!( "  --model <MODEL>                    Model to use" );
  println!( "  --dry-run                          Print command without executing" );
  println!( "  --trace                            Print command to stderr then execute" );
  println!( "  --system-prompt <TEXT>             Set system prompt" );
  println!( "  --append-system-prompt <TEXT>      Append to default system prompt" );
  println!( "  --dir <PATH>                       Working directory" );
  println!( "  --subdir <NAME>                    Named subdirectory appended to --dir as /-NAME; . = identity" );
  println!( "  --session-dir <PATH>               Session storage directory" );
  println!( "  --verbosity <0-5>                  Runner output verbosity level" );
  println!( "  --json-schema <SCHEMA>             JSON schema for structured output" );
  println!( "  --mcp-config <PATH>                MCP server config file (repeatable)" );
  println!( "  --file <PATH>                      Pipe file content to subprocess stdin" );
  println!( "  --strip-fences                     Strip outermost markdown code fences" );
  println!( "  --output-file <PATH>               Write captured output to file (tee: stdout + file)" );
  println!( "  --expect <VALS>                    Pipe-separated expected values; mismatch → exit 3 (case-insensitive, trimmed)" );
  println!( "  --expect-strategy <STRAT>          Mismatch handling: fail (default), retry, default:<VAL>" );
  println!( "  --timeout <SECS>                   Kill subprocess after N seconds (0 = unlimited, default: 0)" );
  println!( "  --retry-override <N>               Force retry count for all error classes" );
  println!( "  --retry-default <N>                Fallback retry count (default: 2)" );
  println!( "  --retry-on-transient <N>           Transient class retry count (default: auto → fallback)" );
  println!( "  --retry-on-service <N>             Service class retry count (default: auto → fallback)" );
  println!( "  --retry-on-unknown <N>             Unknown class retry count (default: auto → fallback)" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "See `clr --help` for the full retry option list (20 params, 3-tier hierarchy)." );
  std::process::exit( 0 );
}
