/// Print help for the main `clr` command to stdout.
pub( crate ) fn print_help()
{
  use cli_fmt::help::*;

  let mut data         = CliHelpData::default();
  data.binary          = "clr".to_string();
  data.tagline         = "Execute Claude Code with configurable parameters".to_string();
  data.usage_lines     = vec!
  [
    "clr <command>".to_string(),
    "clr run [OPTIONS] [<MSG>]".to_string(),
    "clr ask [OPTIONS] [<MSG>]".to_string(),
    "clr ps [OPTIONS]".to_string(),
    "clr kill <PID>".to_string(),
    "clr tools".to_string(),
    "clr isolated [OPTIONS]".to_string(),
    "clr refresh [OPTIONS]".to_string(),
  ];
  data.groups          = vec!
  [
    CommandGroup
    {
      name    : String::new(),
      entries : vec!
      [
        CommandEntry { name : "run".to_string(),      desc : "Execute Claude Code (default mode)".to_string() },
        CommandEntry { name : "ask".to_string(),      desc : "Semantic alias for run (identical behavior)".to_string() },
        CommandEntry { name : "isolated".to_string(), desc : "Run with credential-isolated temp HOME".to_string() },
        CommandEntry { name : "refresh".to_string(),  desc : "Refresh OAuth credentials without a task".to_string() },
        CommandEntry { name : "ps".to_string(),       desc : "List running Claude Code sessions".to_string() },
        CommandEntry { name : "kill".to_string(),     desc : "Terminate a Claude Code session by PID".to_string() },
        CommandEntry { name : "tools".to_string(),    desc : "List all Claude Code built-in tools".to_string() },
        CommandEntry { name : "help".to_string(),     desc : "Print this help and exit".to_string() },
      ],
    },
  ];
  data.option_groups   = vec![ runner_option_group(), claude_code_option_group() ];
  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
}

fn runner_option_group() -> cli_fmt::help::OptionGroup
{
  use cli_fmt::help::{ OptionEntry, OptionGroup };
  OptionGroup
  {
    name    : "RUNNER OPTIONS".to_string(),
    entries : vec!
    [
      OptionEntry { name : "-p, --print".into(),                    desc : "Non-interactive mode (capture and print output)".into() },
      OptionEntry { name : "--interactive".into(),                   desc : "Force interactive mode even when a message is given".into() },
      OptionEntry { name : "--new-session".into(),                   desc : "Start a new session (default: continues previous)".into() },
      OptionEntry { name : "--no-skip-permissions".into(),           desc : "Disable automatic permission bypass (on by default)".into() },
      OptionEntry { name : "--no-ultrathink".into(),                 desc : "Disable automatic \"\\n\\nultrathink\" message suffix".into() },
      OptionEntry { name : "--no-effort-max".into(),                 desc : "Suppress default --effort max injection".into() },
      OptionEntry { name : "--no-chrome".into(),                     desc : "Suppress default --chrome injection".into() },
      OptionEntry { name : "--no-persist".into(),                    desc : "Disable session persistence (--no-session-persistence)".into() },
      OptionEntry { name : "--keep-claudecode".into(),               desc : "Preserve CLAUDECODE env var in subprocess (default: removed)".into() },
      OptionEntry { name : "--verbose".into(),                       desc : "Enable verbose output".into() },
      OptionEntry { name : "--verbosity <0-5>".into(),               desc : "Runner output verbosity level (default: 3)".into() },
      OptionEntry { name : "--dir <PATH>".into(),                    desc : "Working directory".into() },
      OptionEntry { name : "--subdir <NAME>".into(),                 desc : "Named subdirectory appended to --dir as /-NAME; . = identity".into() },
      OptionEntry { name : "--session-dir <PATH>".into(),            desc : "Session storage directory".into() },
      OptionEntry { name : "--dry-run".into(),                       desc : "Print command without executing".into() },
      OptionEntry { name : "--trace".into(),                         desc : "Print command to stderr then execute (like set -x)".into() },
      OptionEntry { name : "--file <PATH>".into(),                   desc : "Pipe file content to subprocess stdin".into() },
      OptionEntry { name : "--strip-fences".into(),                  desc : "Strip outermost markdown code fences from stdout".into() },
      OptionEntry { name : "--output-file <PATH>".into(),            desc : "Write captured output to file (tee: stdout + file)".into() },
      OptionEntry { name : "--output-style <MODE>".into(),           desc : "Rendering mode: summary (default) or raw [env: CLR_OUTPUT_STYLE]".into() },
      OptionEntry { name : "--summary-fields <FIELDS>".into(),       desc : "Summary field selection: minimal, standard, full (default), or comma-separated [env: CLR_SUMMARY_FIELDS]".into() },
      OptionEntry { name : "--journal <LEVEL>".into(),               desc : "Journal level: full (default), meta (no output), or off [env: CLR_JOURNAL]".into() },
      OptionEntry { name : "--journal-dir <PATH>".into(),            desc : "Journal directory (default: ~/.clr/journal/) [env: CLR_JOURNAL_DIR]".into() },
      OptionEntry { name : "--expect <VALS>".into(),                 desc : "Pipe-separated expected values; mismatch → exit 3".into() },
      OptionEntry { name : "--expect-strategy <STRAT>".into(),       desc : "Mismatch handling: fail (default), retry, default:<VAL>".into() },
      OptionEntry { name : "--max-sessions <N>".into(),              desc : "Max concurrent sessions before blocking (0=unlimited, default: 30)".into() },
      OptionEntry { name : "--timeout <SECS>".into(),                desc : "Kill subprocess after N seconds (0 = unlimited, default: 3600)".into() },
      // Retry tier 1: override
      OptionEntry { name : "--retry-override <N>".into(),            desc : "Force retry count for all error classes (unset = per-class)".into() },
      OptionEntry { name : "--retry-override-delay <SECS>".into(),   desc : "Force delay for all error classes (unset = per-class)".into() },
      // Retry tier 2: class-specific
      OptionEntry { name : "--retry-on-transient <N>".into(),        desc : "Transient (rate limit) retry count (default: auto → fallback)".into() },
      OptionEntry { name : "--transient-delay <SECS>".into(),        desc : "Transient class delay (default: auto → fallback)".into() },
      OptionEntry { name : "--retry-on-account <N>".into(),          desc : "Account (quota exhausted) retry count (default: auto → fallback)".into() },
      OptionEntry { name : "--account-delay <SECS>".into(),          desc : "Account class delay (default: auto → fallback)".into() },
      OptionEntry { name : "--retry-on-auth <N>".into(),             desc : "Auth (credential) retry count (default: auto → fallback)".into() },
      OptionEntry { name : "--auth-delay <SECS>".into(),             desc : "Auth class delay (default: auto → fallback)".into() },
      OptionEntry { name : "--retry-on-service <N>".into(),          desc : "Service (API error) retry count (default: auto → fallback)".into() },
      OptionEntry { name : "--service-delay <SECS>".into(),          desc : "Service class delay (default: auto → fallback)".into() },
      OptionEntry { name : "--retry-on-process <N>".into(),          desc : "Process (signal/timeout) retry count (default: auto → fallback)".into() },
      OptionEntry { name : "--process-delay <SECS>".into(),          desc : "Process class delay (default: auto → fallback)".into() },
      OptionEntry { name : "--retry-on-validation <N>".into(),       desc : "Validation (--expect mismatch) retry count (default: auto → fallback)".into() },
      OptionEntry { name : "--validation-delay <SECS>".into(),       desc : "Validation class delay (default: auto → fallback)".into() },
      OptionEntry { name : "--retry-on-runner <N>".into(),           desc : "Runner (infrastructure) retry count (default: auto → fallback)".into() },
      OptionEntry { name : "--runner-delay <SECS>".into(),           desc : "Runner class delay (default: auto → fallback)".into() },
      OptionEntry { name : "--retry-on-unknown <N>".into(),          desc : "Unknown error retry count (default: auto → fallback)".into() },
      OptionEntry { name : "--unknown-delay <SECS>".into(),          desc : "Unknown class delay (default: auto → fallback)".into() },
      // Retry tier 3: fallback
      OptionEntry { name : "--retry-default <N>".into(),             desc : "Fallback retry count for unset classes (default: 2)".into() },
      OptionEntry { name : "--retry-default-delay <SECS>".into(),    desc : "Fallback delay for unset classes (default: 30; 0 = immediate)".into() },
      OptionEntry { name : "-h, --help".into(),                      desc : "Show this help".into() },
    ],
  }
}

fn claude_code_option_group() -> cli_fmt::help::OptionGroup
{
  use cli_fmt::help::{ OptionEntry, OptionGroup };
  OptionGroup
  {
    name    : "CLAUDE CODE OPTIONS (forwarded)".to_string(),
    entries : vec!
    [
      OptionEntry { name : "--model <MODEL>".into(),                 desc : "Model to use".into() },
      OptionEntry { name : "--max-tokens <N>".into(),                desc : "Max output tokens (default: 200000)".into() },
      OptionEntry { name : "--effort <LEVEL>".into(),                desc : "Reasoning effort: low, medium, high, max (default: max)".into() },
      OptionEntry { name : "--output-format <FMT>".into(),           desc : "Output format: text, json, stream-json".into() },
      OptionEntry { name : "--max-turns <N>".into(),                 desc : "Max agentic turns (0 = unlimited)".into() },
      OptionEntry { name : "--allowed-tools <TOOLS>".into(),         desc : "Comma-separated tool whitelist (e.g. \"Read,Edit\")".into() },
      OptionEntry { name : "--disallowed-tools <TOOLS>".into(),      desc : "Comma-separated tool blacklist".into() },
      OptionEntry { name : "--max-budget-usd <AMOUNT>".into(),       desc : "Max API spend in USD for this session".into() },
      OptionEntry { name : "--add-dir <PATH>".into(),                desc : "Additional directory Claude may access".into() },
      OptionEntry { name : "--fallback-model <MODEL>".into(),        desc : "Fallback model when primary is unavailable".into() },
      OptionEntry { name : "--json-schema <SCHEMA>".into(),          desc : "JSON schema for structured output".into() },
      OptionEntry { name : "--mcp-config <PATH>".into(),             desc : "MCP server config file (repeatable)".into() },
      OptionEntry { name : "--system-prompt <TEXT>".into(),          desc : "Set system prompt (replaces the default)".into() },
      OptionEntry { name : "--append-system-prompt <TEXT>".into(),   desc : "Append text to the default system prompt".into() },
    ],
  }
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
  println!( "  clr isolated --creds <FILE> [OPTIONS] [MESSAGE] [-- PASSTHROUGH...]" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  [MESSAGE]                          Prompt message for Claude" );
  println!();
  println!( "CREDENTIAL OPTIONS:" );
  println!( "  --creds <FILE>                     Credentials JSON file (required) [env: CLR_CREDS]" );
  println!( "  --timeout <SECS>                   Max seconds to wait for subprocess (default: 30) [env: CLR_TIMEOUT]" );
  println!( "  --trace                            Print underlying call details to stderr [env: CLR_TRACE]" );
  println!( "  --journal <LEVEL>                  Journal level: full (default), meta, or off [env: CLR_JOURNAL]" );
  println!( "  --journal-dir <PATH>               Journal directory (default: ~/.clr/journal/) [env: CLR_JOURNAL_DIR]" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "ISOLATION OPTIONS:" );
  println!( "  --dry-run                          Print subprocess command without executing; exit 0" );
  println!( "  --dir <PATH>                       Working directory for the subprocess [env: CLR_DIR]" );
  println!( "  --add-dir <PATH>                   Additional directory Claude may access (repeatable) [env: CLR_ADD_DIR]" );
  println!( "  --file <PATH>                      Pipe file content to subprocess stdin" );
  println!( "  --expect <VALS>                    Pipe-separated expected values; mismatch triggers strategy (case-insensitive, trimmed)" );
  println!( "  --expect-strategy <STRAT>          Mismatch handling: fail (default) → exit 3; default:<VAL> → print VAL, exit 0" );
  println!( "                                     Note: retry is NOT supported for isolated (one-shot semantics)" );
  println!( "  --output-file <PATH>               Write output to file (also prints to stdout) [env: CLR_OUTPUT_FILE]" );
  println!( "  --strip-fences                     Strip outermost markdown code fences from output [env: CLR_STRIP_FENCES]" );
  println!( "  --output-style <MODE>              Output rendering: raw (default), summary [env: CLR_OUTPUT_STYLE]" );
  println!( "  --summary-fields <PROFILE>         Summary field selection: full, standard, minimal, or comma-separated [env: CLR_SUMMARY_FIELDS]" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Success" );
  println!( "  1    Error (bad arguments, unsupported --expect-strategy retry, subprocess failure)" );
  println!( "  2    Timeout — subprocess did not finish within --timeout seconds" );
  println!( "  3    Expect mismatch with fail strategy" );
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
  println!( "  --journal <LEVEL>                  Journal level: full (default), meta, or off [env: CLR_JOURNAL]" );
  println!( "  --journal-dir <PATH>               Journal directory (default: ~/.clr/journal/) [env: CLR_JOURNAL_DIR]" );
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
  println!( "  ⚡  Active          CPU delta >= 3 ticks in 1-second sample window" );
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
  println!( "  --timeout <SECS>                   Kill subprocess after N seconds (0 = unlimited, default: 3600)" );
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
