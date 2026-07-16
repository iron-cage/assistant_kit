//! `clr tools` — list all Claude Code built-in tools in a plain-style table.

use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Heading, Format };

/// Static list of 40 Claude Code built-in tools: (name, category, description).
///
/// Source: `contract/claude_code/docs/tool/readme.md`
///
/// Fix(BUG-409): array held only the first 26 of 40 contract-documented tools.
/// Root cause: hand-maintained literal never updated after the contract doc grew;
///   nothing enforced sync between the two, so the array silently fell 14 entries behind.
/// Pitfall: keep this bijective with the contract doc on both name AND category —
///   `tests/tools_command_test.rs`'s TS-1..TS-4 sync-guard tests enforce this on
///   every run; a forward-only check would not have caught this defect (see TS-3).
pub static TOOLS : &[ ( &str, &str, &str ) ] = &[
  ( "Read",            "File Operations", "Read files (text, image, PDF, notebook) with line numbers" ),
  ( "Write",           "File Operations", "Create or overwrite files" ),
  ( "Edit",            "File Operations", "Patch files via exact string replacement" ),
  ( "Bash",            "Shell",           "Execute shell commands with timeout control" ),
  ( "Glob",            "Search",          "Find files by glob patterns" ),
  ( "Grep",            "Search",          "Search file contents with regex (ripgrep)" ),
  ( "Agent",           "Agents",          "Launch specialized subagent processes" ),
  ( "AskUserQuestion", "Interaction",     "Prompt user for input or clarification" ),
  ( "WebFetch",        "Web",             "Fetch content from a URL" ),
  ( "WebSearch",       "Web",             "Search the web" ),
  ( "NotebookEdit",    "File Operations", "Edit Jupyter notebook cells" ),
  ( "LSP",             "Code Intelligence", "Language server protocol queries" ),
  ( "Skill",           "Extensibility",   "Invoke user-defined slash command skills" ),
  ( "TaskCreate",      "Background Tasks", "Create and start background tasks" ),
  ( "TaskGet",         "Background Tasks", "Get information about a background task" ),
  ( "TaskList",        "Background Tasks", "List all background tasks" ),
  ( "TaskOutput",      "Background Tasks", "Read output from a background task" ),
  ( "TaskStop",        "Background Tasks", "Stop a running background task" ),
  ( "TaskUpdate",      "Background Tasks", "Update status of a background task" ),
  ( "CronCreate",      "Scheduling",      "Create recurring scheduled tasks" ),
  ( "CronDelete",      "Scheduling",      "Delete scheduled tasks" ),
  ( "CronList",        "Scheduling",      "List scheduled tasks" ),
  ( "EnterPlanMode",   "Mode",            "Enter plan mode (read-only analysis)" ),
  ( "ExitPlanMode",    "Mode",            "Exit plan mode" ),
  ( "EnterWorktree",   "Mode",            "Enter git worktree isolation" ),
  ( "ExitWorktree",    "Mode",            "Exit git worktree isolation" ),
  ( "TodoWrite",             "Interaction",    "Create and manage structured task lists" ),
  ( "Artifact",              "Publishing",     "Publish HTML/Markdown as shareable artifact" ),
  ( "Monitor",               "Background Tasks", "Run background command, feed output to model" ),
  ( "PowerShell",            "Shell",          "Execute PowerShell commands natively" ),
  ( "PushNotification",      "Notification",   "Send desktop or phone push notification" ),
  ( "ListMcpResourcesTool",  "MCP Resources",  "List resources from connected MCP servers" ),
  ( "ReadMcpResourceTool",   "MCP Resources",  "Read a specific MCP resource by URI" ),
  ( "RemoteTrigger",         "Scheduling",     "Create/run Routines on claude.ai" ),
  ( "ScheduleWakeup",        "Scheduling",     "Reschedule next /loop iteration" ),
  ( "SendMessage",           "Agents",         "Message agent team teammate or subagent" ),
  ( "ShareOnboardingGuide",  "Interaction",    "Upload ONBOARDING.md, return share link" ),
  ( "ToolSearch",            "Extensibility",  "Search/load deferred MCP tool schemas" ),
  ( "WaitForMcpServers",     "Extensibility",  "Wait for background MCP server connections" ),
  ( "Workflow",              "Agents",         "Run dynamic multi-subagent workflow" ),
];

/// Runtime configuration for `clr tools`, assembled from CLI tokens.
struct ToolsConfig
{
  /// Case-insensitive substring filter on tool name from `--name`.
  name     : Option< String >,
  /// Case-insensitive substring filter on tool category from `--category`; combines with `name` via AND.
  category : Option< String >,
  /// Comma-separated column keys from `--columns`; silently ignored when `value` or `inspect` is active.
  columns  : Option< String >,
  /// Single column key from `--value`; mutually exclusive with `inspect`.
  value    : Option< String >,
  /// When `true`: emit key:value inspect blocks instead of a table.
  inspect  : bool,
}

// All 4 column keys in display order, paired with their table header strings.
const COLUMN_KEYS : &[ ( &str, &str ) ] = &[
  ( "idx",      "#" ),
  ( "name",     "Tool" ),
  ( "category", "Category" ),
  ( "desc",     "Description" ),
];

// Look up a single column key against COLUMN_KEYS; returns the canonical 'static key.
fn lookup_column_key( key : &str ) -> Option< &'static str >
{
  COLUMN_KEYS.iter().find( | ( k, _ ) | *k == key ).map( | ( k, _ ) | *k )
}

// Render the valid-keys list for error messages.
fn valid_keys_string() -> String
{
  COLUMN_KEYS.iter().map( | ( k, _ ) | *k ).collect::< Vec< _ > >().join( ", " )
}

// Validate a comma-separated column key string against COLUMN_KEYS.
//
// Mirrors `ps.rs`'s `validate_columns`, adapted to `tools`' 4-key column set.
fn validate_columns( csv : &str ) -> Result< Vec< &'static str >, String >
{
  let mut out = Vec::new();
  for raw in csv.split( ',' )
  {
    let key = raw.trim();
    match lookup_column_key( key )
    {
      Some( k ) => out.push( k ),
      None => return Err( format!( "unknown column key `{key}`; valid keys: {}", valid_keys_string() ) ),
    }
  }
  if out.is_empty()
  {
    return Err( format!( "no column keys given; valid keys: {}", valid_keys_string() ) );
  }
  Ok( out )
}

// Validate a single `--value` key against COLUMN_KEYS.
fn validate_value_key( key : &str ) -> Result< &'static str, String >
{
  lookup_column_key( key ).ok_or_else( || format!( "unknown value key `{key}`; valid keys: {}", valid_keys_string() ) )
}

// Case-insensitive substring match; `lower_filter` is already lowercased by the caller.
// `None` filter matches everything.
fn matches_filter( haystack : &str, lower_filter : &Option< String > ) -> bool
{
  lower_filter.as_ref().map_or( true, | f | haystack.to_lowercase().contains( f.as_str() ) )
}

// Apply --name/--category filters (AND logic) to TOOLS, preserving array order.
fn filter_tools( config : &ToolsConfig ) -> Vec< &'static ( &'static str, &'static str, &'static str ) >
{
  let name_filter = config.name.as_ref().map( | s | s.to_lowercase() );
  let category_filter = config.category.as_ref().map( | s | s.to_lowercase() );
  TOOLS
    .iter()
    .filter( | ( name, cat, _ ) | matches_filter( name, &name_filter ) && matches_filter( cat, &category_filter ) )
    .collect()
}

// Emit a key:value inspect record for each matching tool.
//
// Mirrors `ps.rs`'s `build_inspect_output`, adapted to tools' 4 attributes
// (idx, name, category, desc) and a `Tool <name>` rule line instead of `PID <pid>`.
fn build_inspect_output( tools : &[ &( &str, &str, &str ) ] ) -> String
{
  use core::fmt::Write as _;
  let mut out = String::new();
  for ( idx, ( name, cat, desc ) ) in tools.iter().enumerate()
  {
    if idx > 0 { out.push( '\n' ); }
    let rule = format!( "──── Tool {name} {}", "─".repeat( 50 ) );
    let _ = writeln!( out, "{rule}" );
    let _ = writeln!( out, "{:<10}{}", "idx:", idx + 1 );
    let _ = writeln!( out, "{:<10}{name}", "name:" );
    let _ = writeln!( out, "{:<10}{cat}", "category:" );
    let _ = writeln!( out, "{:<10}{desc}", "desc:" );
  }
  out
}

// Render the filtered tool list as a headed plain-style table restricted to `columns`.
//
// data_fmt ≥0.5.1 fills the heading rule to the rendered table body width
// automatically, so no two-pass probe is required. Zero-row `tools` renders a
// headers-only table (data_fmt supports this directly) rather than a special
// empty-state message — see `docs/cli/param/078_name.md`'s zero-match note.
fn render_tools_table( tools : &[ &( &str, &str, &str ) ], columns : &[ &str ] ) -> String
{
  let headers : Vec< String > = columns
    .iter()
    .map( | key | COLUMN_KEYS.iter()
      .find( | ( k, _ ) | k == key )
      .map_or_else( || ( *key ).to_string(), | ( _, h ) | ( *h ).to_string() ) )
    .collect();
  let mut builder = RowBuilder::new( headers );
  for ( idx, ( name, cat, desc ) ) in tools.iter().enumerate()
  {
    let row : Vec< String > = columns
      .iter()
      .map( | key | match *key
      {
        "idx"      => ( idx + 1 ).to_string(),
        "name"     => ( *name ).to_string(),
        "category" => ( *cat ).to_string(),
        "desc"     => ( *desc ).to_string(),
        _          => String::new(),
      } )
      .collect();
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }
  let heading = Heading::new( "Claude Code Tools" )
    .with_field( format!( "{} built-in", tools.len() ) );
  Format::format(
    &TableFormatter::with_config(
      TableConfig::plain()
        .with_heading( heading ),
    ),
    &builder.build_view(),
  ).unwrap_or_default()
}

/// List all Claude Code built-in tools in a plain-style table.  Never returns.
///
/// Accepts `--name`, `--category`, `--columns`, `--value`, `--inspect` (and `--help`/`-h`).
/// Exits 0 with the table (or bare values, or inspect blocks — including zero matches);
/// exits 1 on unknown arguments, missing flag values, an unknown column/value key, or
/// `--value` combined with `--inspect`.
#[ allow( clippy::too_many_lines ) ] // mechanical dispatch — one arm per CLI flag
pub( crate ) fn dispatch_tools( tokens : &[ String ] ) -> !
{
  let mut config = ToolsConfig { name : None, category : None, columns : None, value : None, inspect : false };

  let mut i = 1_usize; // tokens[0] is "tools"
  while i < tokens.len()
  {
    match tokens[ i ].as_str()
    {
      "--help" | "-h" =>
      {
        super::help::print_tools_help();
      }
      "--name" =>
      {
        i += 1;
        if i >= tokens.len() { eprintln!( "clr tools: `--name` requires a value" ); std::process::exit( 1 ); }
        config.name = Some( tokens[ i ].clone() );
      }
      "--category" =>
      {
        i += 1;
        if i >= tokens.len() { eprintln!( "clr tools: `--category` requires a value" ); std::process::exit( 1 ); }
        config.category = Some( tokens[ i ].clone() );
      }
      "--columns" =>
      {
        i += 1;
        if i >= tokens.len() { eprintln!( "clr tools: `--columns` requires a value" ); std::process::exit( 1 ); }
        config.columns = Some( tokens[ i ].clone() );
      }
      "--value" =>
      {
        i += 1;
        if i >= tokens.len() { eprintln!( "clr tools: `--value` requires a value" ); std::process::exit( 1 ); }
        config.value = Some( tokens[ i ].clone() );
      }
      "--inspect" =>
      {
        config.inspect = true;
      }
      arg =>
      {
        eprintln!( "Error: 'clr tools' does not accept arguments: {arg}" );
        std::process::exit( 1 );
      }
    }
    i += 1;
  }

  if config.value.is_some() && config.inspect
  {
    eprintln!( "clr tools: `--value` and `--inspect` cannot be combined" );
    std::process::exit( 1 );
  }

  // Resolve --value eagerly so an invalid key is reported before filtering runs.
  let value_key : Option< &'static str > = match &config.value
  {
    Some( v ) => match validate_value_key( v )
    {
      Ok( k ) => Some( k ),
      Err( msg ) => { eprintln!( "clr tools: {msg}" ); std::process::exit( 1 ); }
    },
    None => None,
  };

  // --columns is silently ignored (not even validated) when --value or --inspect is active.
  let columns : Vec< &'static str > = if config.inspect || value_key.is_some()
  {
    Vec::new()
  }
  else
  {
    match &config.columns
    {
      Some( csv ) => match validate_columns( csv )
      {
        Ok( keys ) => keys,
        Err( msg ) => { eprintln!( "clr tools: {msg}" ); std::process::exit( 1 ); }
      },
      None => COLUMN_KEYS.iter().map( | ( k, _ ) | *k ).collect(),
    }
  };

  let filtered = filter_tools( &config );

  if let Some( key ) = value_key
  {
    for ( idx, ( name, cat, desc ) ) in filtered.iter().enumerate()
    {
      let val = match key
      {
        "idx"      => ( idx + 1 ).to_string(),
        "name"     => ( *name ).to_string(),
        "category" => ( *cat ).to_string(),
        "desc"     => ( *desc ).to_string(),
        _          => String::new(),
      };
      println!( "{val}" );
    }
    std::process::exit( 0 );
  }

  if config.inspect
  {
    let output = build_inspect_output( &filtered );
    if !output.is_empty()
    {
      println!( "{output}" );
    }
    std::process::exit( 0 );
  }

  println!( "{}", render_tools_table( &filtered, &columns ) );
  std::process::exit( 0 );
}
