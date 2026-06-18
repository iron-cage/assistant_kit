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
  println!( "  clr ps" );
  println!();
  println!( "ACTIVE SESSIONS TABLE:" );
  println!( "  #                                  Row index" );
  println!( "  PID                                Process ID" );
  println!( "  Elapsed                            Time since session started" );
  println!( "  CPU%                               CPU usage percentage" );
  println!( "  RAM                                Resident memory (K or M suffix)" );
  println!( "  State                              Process state (R/S/Z…)" );
  println!( "  Absolute Path                      Working directory (\\$PRO prefix shortened when PRO is set)" );
  println!( "  Task                               Last human message extracted from session JSONL (≤35 chars)" );
  println!();
  println!( "QUEUED CLR PROCESSES TABLE (shown when gate files exist in CLR_GATE_DIR):" );
  println!( "  #                                  Row index" );
  println!( "  PID                                Process ID of the waiting clr process" );
  println!( "  CWD                                Working directory of the waiting process" );
  println!( "  Waiting                            Time spent waiting for a session slot" );
  println!( "  Attempt                            Number of polling attempts so far" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Success (table printed or empty-state message shown)" );
  println!( "  1    Error (unexpected argument)" );
  println!();
  println!( "ENVIRONMENT:" );
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
  println!( "  --retry-on-transient <N>           Transient class retry count" );
  println!( "  --retry-on-service <N>             Service class retry count" );
  println!( "  --retry-on-unknown <N>             Unknown class retry count" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "See `clr --help` for the full retry option list (20 params, 3-tier hierarchy)." );
  std::process::exit( 0 );
}
