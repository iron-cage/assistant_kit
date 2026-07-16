//! `clr tools` Subcommand Integration Tests
//!
//! Covers the `tools` command from `docs/cli/command/08_tools.md`.
//! Verifies that `clr tools` lists all Claude Code built-in tools with correct
//! column headers and exit 0, and that the `TOOLS` array stays in sync with
//! `contract/claude_code/docs/tool/readme.md` (see `docs/invariant/015_tools_array_doc_sync.md`).

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;
use claude_runner::TOOLS;
use std::path::Path;

// ── IT-1: `clr tools` exits 0 ─────────────────────────────────────────────────

/// IT-1: `clr tools` exits 0.
#[ test ]
fn it1_tools_exits_zero()
{
  let out = run_cli( &[ "tools" ] );
  assert!(
    out.status.success(),
    "`clr tools` must exit 0. Got: {:?}",
    out.status.code()
  );
}

// ── IT-2: Output contains expected tool names ─────────────────────────────────

/// IT-2: Stdout lists at least "Read", "Write", "Edit", "Bash" and "Agent".
#[ test ]
fn it2_tools_lists_core_tools()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "`clr tools` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for tool in &[ "Read", "Write", "Edit", "Bash", "Agent" ]
  {
    assert!(
      stdout.contains( tool ),
      "`clr tools` stdout must contain '{tool}'. Got:\n{stdout}"
    );
  }
}

// ── IT-3: Output contains expected categories ─────────────────────────────────

/// IT-3: Stdout contains the category headers present in the TOOLS table.
#[ test ]
fn it3_tools_lists_categories()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "`clr tools` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for cat in &[ "File Operations", "Shell", "Search", "Web" ]
  {
    assert!(
      stdout.contains( cat ),
      "`clr tools` stdout must contain category '{cat}'. Got:\n{stdout}"
    );
  }
}

// ── IT-4: Output contains table caption with count ────────────────────────────

/// IT-4: Stdout contains the caption "Claude Code Tools" and the live `TOOLS.len()` count.
///
/// # Fix Applied (BUG-409)
/// Was hardcoded `stdout.contains( "26" )`, which silently kept passing while `TOOLS`
/// held a stale, incomplete count. Reads `TOOLS.len()` dynamically so this test tracks
/// whatever count is actually correct, rather than re-encoding a copy of the bug.
#[ test ]
fn it4_tools_caption_present()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "`clr tools` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Claude Code Tools" ),
    "`clr tools` stdout must contain table caption 'Claude Code Tools'. Got:\n{stdout}"
  );
  let count = TOOLS.len().to_string();
  assert!(
    stdout.contains( &count ),
    "`clr tools` stdout must reference {count} tools (TOOLS.len()). Got:\n{stdout}"
  );
}

// ── IT-5: `clr tools --help` exits 0 ─────────────────────────────────────────

/// IT-5: `clr tools --help` exits 0 and prints usage.
#[ test ]
fn it5_tools_help_exits_zero()
{
  let out = run_cli( &[ "tools", "--help" ] );
  assert!(
    out.status.success(),
    "`clr tools --help` must exit 0. Got: {:?}",
    out.status.code()
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "tools" ),
    "`clr tools --help` stdout must contain 'tools'. Got:\n{stdout}"
  );
}

// ── IT-6: `clr tools` output contains Cron and Mode tools ────────────────────

/// IT-6: Stdout lists Cron and Mode tool names from the Scheduling/Mode categories.
#[ test ]
fn it6_tools_lists_scheduling_and_mode_tools()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "`clr tools` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for tool in &[ "CronCreate", "CronDelete", "CronList", "EnterPlanMode", "ExitPlanMode" ]
  {
    assert!(
      stdout.contains( tool ),
      "`clr tools` stdout must contain '{tool}'. Got:\n{stdout}"
    );
  }
}

// ── IT-7: `clr help` mentions `tools` command ────────────────────────────────

/// IT-7: `clr --help` lists `tools` in the COMMANDS section.
#[ test ]
fn it7_tools_listed_in_main_help()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "`clr --help` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "tools" ),
    "`clr --help` stdout must mention 'tools' command. Got:\n{stdout}"
  );
}

// ── IT-8: `clr tools -h` short help flag ─────────────────────────────────────

/// IT-8: `clr tools -h` exits 0 and prints usage containing "tools".
///
/// Spec: `tests/docs/cli/command/08_tools.md` IT-8
#[ test ]
fn it8_tools_short_help_flag()
{
  let out = run_cli( &[ "tools", "-h" ] );
  assert!(
    out.status.success(),
    "`clr tools -h` must exit 0. Got: {:?}",
    out.status.code()
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "tools" ),
    "`clr tools -h` stdout must contain 'tools'. Got:\n{stdout}"
  );
}

// ── IT-9: `clr tools <unexpected-arg>` rejects arguments ─────────────────────

/// IT-9: `clr tools some-arg` exits 1 with error message — no flags or positional
/// args are accepted (other than --help/-h).
///
/// Spec: `tests/docs/cli/command/08_tools.md` IT-9
///
/// # Root Cause (bug reproducer)
/// `dispatch_tools()` had no unknown-arg guard: any input after the help check
/// fell through to table rendering and exited 0 silently.
///
/// # Fix Applied
/// Unknown-arg guard inserted before table build in `src/cli/tools.rs`.
#[ test ]
fn it9_tools_rejects_unknown_arg()
{
  let out = run_cli( &[ "tools", "some-unknown-arg" ] );
  assert!(
    !out.status.success(),
    "`clr tools some-unknown-arg` must exit non-zero. Got exit 0"
  );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "`clr tools some-unknown-arg` must exit 1. Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "does not accept arguments" ),
    "`clr tools some-unknown-arg` stderr must contain 'does not accept arguments'. Got:\n{stderr}"
  );
}

// ── IT-10: `--name` filters to name-matching tools only ─────────────────────

/// IT-10: `clr tools --name task` shows only tools whose name contains "task"
/// (case-insensitive substring match).
///
/// Spec: `tests/docs/cli/command/08_tools.md` IT-10
#[ test ]
fn it10_tools_name_filter()
{
  let out = run_cli( &[ "tools", "--name", "task" ] );
  assert!( out.status.success(), "`clr tools --name task` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for tool in &[ "TaskCreate", "TaskGet", "TaskList" ]
  {
    assert!( stdout.contains( tool ), "stdout must contain '{tool}'. Got:\n{stdout}" );
  }
  assert!( !stdout.contains( "Bash" ), "stdout must NOT contain 'Bash'. Got:\n{stdout}" );
  // Checks the Read *tool*'s own distinguishing description text, not the bare
  // substring "Read" -- TaskOutput's own description ("Read output from a
  // background task") legitimately contains "Read" as an ordinary English word.
  assert!(
    !stdout.contains( "Read files" ),
    "stdout must NOT contain the Read tool's row. Got:\n{stdout}"
  );
}

// ── IT-11: `--category` filters to category-matching tools only ─────────────

/// IT-11: `clr tools --category Web` shows only tools whose category contains
/// "Web" (case-insensitive substring match).
///
/// Spec: `tests/docs/cli/command/08_tools.md` IT-11
#[ test ]
fn it11_tools_category_filter()
{
  let out = run_cli( &[ "tools", "--category", "Web" ] );
  assert!( out.status.success(), "`clr tools --category Web` must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for tool in &[ "WebFetch", "WebSearch" ]
  {
    assert!( stdout.contains( tool ), "stdout must contain '{tool}'. Got:\n{stdout}" );
  }
  assert!( !stdout.contains( "Bash" ), "stdout must NOT contain 'Bash'. Got:\n{stdout}" );
}

// ── IT-12: `--name` and `--category` combine with AND logic ─────────────────

/// IT-12: `clr tools --name cron --category Scheduling` combines both filters
/// with AND logic -- excludes `RemoteTrigger`/`ScheduleWakeup`, which share the
/// "Scheduling" category but whose names do not contain "cron".
///
/// Requires task 412 (`TOOLS` array completeness) -- see task 413's own
/// Dependency section: `RemoteTrigger`/`ScheduleWakeup` do not exist in the
/// pre-412 26-entry array, which would make this assertion pass trivially.
///
/// Spec: `tests/docs/cli/param_group/07_tool_listing.md` IT-12 / G7-CC3
#[ test ]
fn it12_tools_name_and_category_and_logic()
{
  let out = run_cli( &[ "tools", "--name", "cron", "--category", "Scheduling" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for tool in &[ "CronCreate", "CronDelete", "CronList" ]
  {
    assert!( stdout.contains( tool ), "stdout must contain '{tool}'. Got:\n{stdout}" );
  }
  for tool in &[ "RemoteTrigger", "ScheduleWakeup" ]
  {
    assert!( !stdout.contains( tool ), "stdout must NOT contain '{tool}'. Got:\n{stdout}" );
  }
}

// ── IT-13: zero matches after filtering is not an error ─────────────────────

/// IT-13: `clr tools --name doesnotexist` exits 0 with the table heading present
/// but no tool rows -- filtering to nothing is never an error.
///
/// Spec: `tests/docs/cli/param/078_name.md` IT-13 / EC-3
#[ test ]
fn it13_tools_zero_match_not_error()
{
  let out = run_cli( &[ "tools", "--name", "doesnotexist" ] );
  assert!( out.status.success(), "zero-match must still exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Claude Code Tools" ),
    "stdout must still contain the table heading/caption. Got:\n{stdout}"
  );
  for tool in &[ "Bash", "Read", "TaskCreate" ]
  {
    assert!( !stdout.contains( tool ), "stdout must contain no tool rows -- found '{tool}'. Got:\n{stdout}" );
  }
}

// ── IT-14: `--columns` narrows displayed columns ─────────────────────────────

/// IT-14: `clr tools --columns name,category` shows only the 2 selected columns
/// -- header contains "Tool"/"Category" but not "Description".
///
/// Spec: `tests/docs/cli/param/059_columns.md` IT-14
#[ test ]
fn it14_tools_columns_projection()
{
  let out = run_cli( &[ "tools", "--columns", "name,category" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "Tool" ), "header must contain 'Tool'. Got:\n{stdout}" );
  assert!( stdout.contains( "Category" ), "header must contain 'Category'. Got:\n{stdout}" );
  assert!( !stdout.contains( "Description" ), "header must NOT contain 'Description'. Got:\n{stdout}" );
}

// ── IT-15: unknown `--columns` key rejected ──────────────────────────────────

/// IT-15: `clr tools --columns badkey` exits 1 with stderr listing valid keys.
///
/// Spec: `tests/docs/cli/param/059_columns.md` IT-15
#[ test ]
fn it15_tools_columns_invalid_key_rejected()
{
  let out = run_cli( &[ "tools", "--columns", "badkey" ] );
  assert_eq!( out.status.code(), Some( 1 ), "must exit 1: {out:?}" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  for key in &[ "idx", "name", "category", "desc" ]
  {
    assert!( stderr.contains( key ), "stderr must list valid key '{key}'. Got:\n{stderr}" );
  }
}

// ── IT-16: `--value` prints bare column values ───────────────────────────────

/// IT-16: `clr tools --value name` prints bare tool names, one per line, with
/// zero table decoration (no caption, no column headers).
///
/// Spec: `tests/docs/cli/param/080_value.md` IT-16
#[ test ]
fn it16_tools_value_bare_output()
{
  let out = run_cli( &[ "tools", "--value", "name" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.lines().any( | l | l == "Bash" ), "stdout must contain 'Bash' on its own line. Got:\n{stdout}" );
  assert!( !stdout.contains( "Claude Code Tools" ), "stdout must NOT contain the table caption. Got:\n{stdout}" );
  assert!( !stdout.contains( "Category" ), "stdout must NOT contain table header text. Got:\n{stdout}" );
}

// ── IT-17: `--value` combined with a single-row filter ───────────────────────

/// IT-17: `clr tools --name Bash --value category` narrows to exactly one
/// matching tool and prints exactly one bare value.
///
/// Spec: `tests/docs/cli/param/080_value.md` IT-17
#[ test ]
fn it17_tools_value_single_row_narrowing()
{
  let out = run_cli( &[ "tools", "--name", "Bash", "--value", "category" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert_eq!( stdout, "Shell\n", "stdout must be exactly 'Shell\\n'. Got:\n{stdout:?}" );
}

// ── IT-18: `--inspect` prints key:value record blocks ────────────────────────

/// IT-18: `clr tools --inspect` prints key:value record blocks for every tool
/// -- no table header row.
///
/// Spec: `tests/docs/cli/param/069_inspect.md` IT-18
#[ test ]
fn it18_tools_inspect_record_format()
{
  let out = run_cli( &[ "tools", "--inspect" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for label in &[ "name:", "category:", "desc:" ]
  {
    assert!( stdout.contains( label ), "stdout must contain '{label}'. Got:\n{stdout}" );
  }
  assert!( !stdout.contains( "Claude Code Tools" ), "stdout must NOT contain the table caption. Got:\n{stdout}" );
}

// ── IT-19: `--value` and `--inspect` are mutually exclusive ──────────────────

/// IT-19: `clr tools --value name --inspect` exits 1 -- the two output-format
/// switches cannot be combined.
///
/// Spec: `tests/docs/cli/param_group/07_tool_listing.md` IT-19 / G7-CC5
#[ test ]
fn it19_tools_value_inspect_mutually_exclusive()
{
  let out = run_cli( &[ "tools", "--value", "name", "--inspect" ] );
  assert_eq!( out.status.code(), Some( 1 ), "must exit 1: {out:?}" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "--value" ), "stderr must mention --value. Got:\n{stderr}" );
  assert!( stderr.contains( "--inspect" ), "stderr must mention --inspect. Got:\n{stderr}" );
  assert!( stderr.contains( "cannot" ), "stderr must state the flags cannot combine. Got:\n{stderr}" );
}

// ── IT-20: `--columns` ignored when `--inspect` active ───────────────────────

/// IT-20: `clr tools --columns name --inspect` ignores `--columns` entirely --
/// the full record (all 4 attributes) is shown, not a name-only view.
///
/// Spec: `tests/docs/cli/param/059_columns.md` IT-20
#[ test ]
fn it20_tools_columns_ignored_when_inspect_active()
{
  let out = run_cli( &[ "tools", "--columns", "name", "--inspect" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for label in &[ "name:", "category:", "desc:" ]
  {
    assert!( stdout.contains( label ), "stdout must contain '{label}' -- --columns must be ignored. Got:\n{stdout}" );
  }
}

// ── IT-21: `clr tools` stdout tool count matches contract doc (sync guard) ──

/// IT-21: `clr tools` stdout references the same tool count as the contract
/// doc's Tool Table row count -- integration-level manifestation of the
/// invariant/015 sync guard (see TS-1..TS-4 below for the unit-level guard).
///
/// Spec: `docs/invariant/015_tools_array_doc_sync.md`
#[ test ]
fn it21_tools_count_matches_contract_doc()
{
  let out = run_cli( &[ "tools" ] );
  assert!( out.status.success(), "must exit 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let doc_rows = parse_contract_doc_tool_rows();
  let count = doc_rows.len().to_string();
  assert!(
    stdout.contains( &count ),
    "`clr tools` stdout must reference {count} tools (contract doc row count). Got:\n{stdout}"
  );
}

// ── Sync-guard tests: TOOLS array <-> contract/claude_code/docs/tool/readme.md ──
//
// Spec: tests/docs/invariant/015_tools_array_doc_sync.md (TS-1..TS-4)
//
// # Root Cause (BUG-409)
// `TOOLS` was a hand-maintained literal never updated after the contract doc grew
// past 26 entries; nothing enforced sync between the two, so the array silently
// fell 14 entries behind.
//
// # Why Not Caught
// The only pre-existing check (`it4_tools_caption_present`) hardcoded the expected
// count as a literal ("26"), so it kept passing as the stale count itself, instead
// of comparing against the actual source of truth.
//
// # Fix Applied
// Four tests added below, reading the checked-in contract doc directly and checking
// both bijection directions -- forward (every `TOOLS` entry has a doc match) and,
// critically, reverse (every doc row has a `TOOLS` match). A forward-only check is
// satisfied trivially by a strict subset and would not have caught this defect.
//
// # Prevention
// TS-3 (reverse bijection) is the enforcement mechanism named in
// `docs/invariant/015_tools_array_doc_sync.md` -- any future doc growth without a
// matching `TOOLS` update now fails loudly here instead of silently under-reporting.
//
// # Pitfall
// Comparing on `(name, category)` only, not `description` -- the invariant statement
// in `docs/invariant/015_tools_array_doc_sync.md` scopes the bijection to name+category;
// descriptions are allowed to be paraphrased between the two sources.

/// Parses `contract/claude_code/docs/tool/readme.md`'s Tool Table into `(name, category)` pairs.
///
/// A real data row starts with `| <digits> |`; the tool name is the link text between
/// the first `[` and `]` in the second cell. Header (`| # | ... |`) and separator
/// (`|---|...|`) rows are skipped because their first cell does not parse as a number.
fn parse_contract_doc_tool_rows() -> Vec< ( String, String ) >
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let doc_path = manifest.join( "../../contract/claude_code/docs/tool/readme.md" );
  let content = std::fs::read_to_string( &doc_path )
    .unwrap_or_else( | e | panic!( "failed to read {}: {e}", doc_path.display() ) );

  let mut rows = Vec::new();
  for line in content.lines()
  {
    let trimmed = line.trim();
    if !trimmed.starts_with( '|' )
    {
      continue;
    }
    let cells : Vec< &str > = trimmed.split( '|' ).map( str::trim ).collect();
    if cells.len() < 5 || cells[ 1 ].parse::< u32 >().is_err()
    {
      continue;
    }
    let name_cell = cells[ 2 ];
    let name = match ( name_cell.find( '[' ), name_cell.find( ']' ) )
    {
      ( Some( start ), Some( end ) ) if start < end => name_cell[ start + 1 .. end ].to_string(),
      _ => continue,
    };
    rows.push( ( name, cells[ 3 ].to_string() ) );
  }
  rows
}

/// TS-1: `TOOLS.len()` equals the contract doc's Tool Table row count.
///
/// Pins the exact failure mode that motivated invariant 015 -- `TOOLS` held only
/// 26 entries while the contract doc listed 40.
#[ test ]
fn tools_array_count_matches_contract_doc()
{
  let doc_rows = parse_contract_doc_tool_rows();
  assert_eq!(
    TOOLS.len(),
    doc_rows.len(),
    "TOOLS.len() ({}) must equal contract doc row count ({})",
    TOOLS.len(),
    doc_rows.len()
  );
}

/// TS-2: forward bijection -- every `TOOLS` entry's `(name, category)` pair matches
/// a contract-doc row. Satisfied trivially by a strict subset -- see TS-3 for the
/// direction that actually catches an incomplete array.
#[ test ]
fn tools_array_forward_bijection_with_contract_doc()
{
  let doc_rows = parse_contract_doc_tool_rows();
  let missing : Vec< String > = TOOLS
    .iter()
    .filter( | ( name, cat, _ ) | !doc_rows.iter().any( | ( n, c ) | n == name && c == cat ) )
    .map( | ( name, cat, _ ) | format!( "{name} ({cat})" ) )
    .collect();
  assert!(
    missing.is_empty(),
    "TOOLS entries with no matching contract-doc row: {missing:?}"
  );
}

/// TS-3: reverse bijection -- every contract-doc Tool Table row's name has a
/// matching `TOOLS` entry. This is the exact direction that would have caught
/// BUG-409 -- the forward direction alone (TS-2) is satisfied trivially by a subset.
#[ test ]
fn tools_array_reverse_bijection_with_contract_doc()
{
  let doc_rows = parse_contract_doc_tool_rows();
  let missing : Vec< &str > = doc_rows
    .iter()
    .filter( | ( name, _ ) | !TOOLS.iter().any( | ( n, _, _ ) | n == name ) )
    .map( | ( name, _ ) | name.as_str() )
    .collect();
  assert!(
    missing.is_empty(),
    "Contract-doc tools missing from TOOLS: {missing:?}"
  );
}

/// TS-4: the sync guard reads `contract/claude_code/docs/tool/readme.md` from the
/// checked-out working tree via a relative path -- no external URL fetch, so it
/// stays deterministic under container-only test execution (invariant 010).
#[ test ]
fn tools_array_sync_guard_reads_checked_in_doc()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let doc_path = manifest.join( "../../contract/claude_code/docs/tool/readme.md" );
  assert!(
    doc_path.is_file(),
    "sync guard must read the checked-in file at {}",
    doc_path.display()
  );
  let doc_rows = parse_contract_doc_tool_rows();
  assert!(
    !doc_rows.is_empty(),
    "parsed contract doc must yield at least one row -- got 0, parsing likely broken"
  );
}
