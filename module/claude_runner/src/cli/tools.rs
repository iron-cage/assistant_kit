//! `clr tools` — list all Claude Code built-in tools in a plain-style table.

use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Heading, Format };

/// Static list of 26 Claude Code built-in tools: (name, category, description).
///
/// Source: `contract/claude_code/docs/tool/readme.md`
static TOOLS : &[ ( &str, &str, &str ) ] = &[
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
];

/// List all Claude Code built-in tools in a plain-style table.  Never returns.
pub( crate ) fn dispatch_tools( tokens : &[ String ] ) -> !
{
  // Handle help flags — show brief help then exit 0.
  if tokens.iter().skip( 1 ).any( | t | t == "--help" || t == "-h" )
  {
    println!( "clr tools — List all Claude Code built-in tools" );
    println!();
    println!( "USAGE:" );
    println!( "  clr tools" );
    println!();
    println!( "Prints a table of all 26 Claude Code built-in tools with name, category," );
    println!( "and description.  No flags or arguments are accepted." );
    std::process::exit( 0 );
  }

  // Reject any arguments — 'clr tools' accepts no flags or positional args.
  let unexpected : Vec< &str > = tokens.iter().skip( 1 ).map( String::as_str ).collect();
  if !unexpected.is_empty()
  {
    eprintln!(
      "Error: 'clr tools' does not accept arguments: {}",
      unexpected.join( ", " )
    );
    std::process::exit( 1 );
  }

  let headers = vec![
    "#".to_string(),
    "Tool".to_string(),
    "Category".to_string(),
    "Description".to_string(),
  ];
  let mut builder = RowBuilder::new( headers );
  for ( idx, ( name, cat, desc ) ) in TOOLS.iter().enumerate()
  {
    let row : Vec< String > = vec![
      ( idx + 1 ).to_string(),
      ( *name ).to_string(),
      ( *cat ).to_string(),
      ( *desc ).to_string(),
    ];
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }

  let heading = Heading::new( "Claude Code Tools" )
    .with_field( format!( "{} built-in", TOOLS.len() ) );

  // data_fmt ≥0.5.1 fills the heading rule to the rendered table body width
  // automatically, so no two-pass probe is required.
  let table = Format::format(
    &TableFormatter::with_config(
      TableConfig::plain()
        .auto_wrap( false )
        .with_heading( heading ),
    ),
    &builder.build_view(),
  ).unwrap_or_default();

  println!( "{table}" );
  std::process::exit( 0 );
}
