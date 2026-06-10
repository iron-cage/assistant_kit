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
  println!( "  --expect-retries <N>               Retry attempts when --expect-strategy retry (0–255, default: 0)" );
  println!( "  --max-sessions <N>                 Max concurrent claude sessions before blocking (0=unlimited, default: 15)" );
  println!( "  --retry-on-rate-limit <N>          Retry on transient rate limit up to N times (0–255, default: 1; 0 = no retry)" );
  println!( "  --retry-delay <SECS>               Seconds between rate-limit retries (default: 30; 0 = immediate)" );
  println!( "  --timeout <SECS>                   Kill subprocess after N seconds (0 = unlimited, default: 0)" );
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
  println!( "  --expect-retries <N>               Retry attempts when --expect-strategy retry (0–255, default: 0)" );
  println!( "  --retry-on-rate-limit <N>          Retry on transient rate limit up to N times (0–255, default: 1; 0 = no retry)" );
  println!( "  --retry-delay <SECS>               Seconds between rate-limit retries (default: 30; 0 = immediate)" );
  println!( "  --timeout <SECS>                   Kill subprocess after N seconds (0 = unlimited, default: 0)" );
  println!( "  -h, --help                         Show this help" );
  std::process::exit( 0 );
}
