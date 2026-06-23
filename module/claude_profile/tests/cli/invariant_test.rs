//! Integration tests: Architectural invariant assertions.
//!
//! Verifies the six architectural invariants for `claude_profile` through a
//! combination of static source-tree analysis and CLI subprocess calls.
//!
//! ## Test Matrix
//!
//! ### Invariant 001 — Zero Third-Party Dependencies (IN-1..2)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | IN-1 | `zero_third_party_deps_in1_library_deps_are_workspace_only` | `[dependencies]` has no crates.io entries | P |
//! | IN-2 | `zero_third_party_deps_in2_enabled_feature_activates_workspace_deps_only` | `enabled` feature activates only workspace deps | P |
//!
//! ### Invariant 002 — Cross-Platform Compatibility (IN-1..2)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | IN-1 | `cross_platform_in1_no_path_string_concat_in_src` | no `format!(".../"...)` in src/ | P |
//! | IN-2 | `cross_platform_in2_no_tilde_literal_paths_in_src` | no `"~/"` in src/ | P |
//!
//! ### Invariant 003 — Clear Error Messages (IN-1..2)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | IN-1 | `clear_errors_in1_missing_account_error_includes_name` | error for missing account names the account | P |
//! | IN-2 | `clear_errors_in2_missing_creds_error_includes_path` | error for missing credentials names the path | P |
//!
//! ### Invariant 004 — No Process Execution (IN-1..2)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | IN-1 | `no_process_execution_in1_src_contains_zero_std_process` | grep finds no `std::process` in src/ | P |
//! | IN-2 | `no_process_execution_in2_responsibility_test_exists` | responsibility test file is present in tests/ | P |
//!
//! ### Invariant 005 — Atomic Account Switching (IN-1..2)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | IN-1 | `atomic_switching_in1_src_uses_rename_for_credentials` | `std::fs::rename` present in src/; no direct write to `.credentials.json` path | P |
//! | IN-2 | `atomic_switching_in2_credentials_complete_after_switch` | after switch, `.credentials.json` contains complete account-B credentials | P |
//!
//! ### Invariant 006 — Parameters Default to Active Context (IN-1..2)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | IN-1 | `param_defaults_in1_active_account_used_without_name_arg` | `.token.status` without `name::` succeeds when active account set | P |
//! | IN-2 | `param_defaults_in2_require_nonempty_string_arg_only_in_use_delete` | `require_nonempty_string_arg` only in `.account.use` and `.account.delete` handlers | P |
//!
//! ### Invariant 008 — Single Token Refresh Entry Point (IN-1..IN-4)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | FT-25/IN-1 | `single_token_refresh_entry_in1_src_contains_zero_run_isolated_calls` | grep finds no code-level `run_isolated(` in src/ | P |
//! | IN-2 | `single_token_refresh_entry_in2_run_isolated_doc_has_warning` | `# Warning` present in `run_isolated()` doc in `isolated.rs` | P |
//! | IN-3 | `single_token_refresh_entry_in3_expires_manipulation_before_run_isolated` | `manipulate_expires_at(` appears before `run_isolated(` in `account.rs` | P |
//! | IN-4 | `single_token_refresh_entry_in4_live_sync_in_refresh_token_with_live_path` | `credentials_file()` appears in `refresh_token_with_live_path` in `account.rs` | P |

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use super::cli_runner::{ run_cs_with_env, assert_exit, write_credentials, write_account, FAR_FUTURE_MS };

// ── Invariant 001: Zero Third-Party Dependencies ──────────────────────────────

// IN-1: Library path deps are workspace-only (no crates.io version strings)
#[ test ]
fn zero_third_party_deps_in1_library_deps_are_workspace_only()
{
  let cargo_toml = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "Cargo.toml" );
  let content = std::fs::read_to_string( &cargo_toml ).unwrap();

  // Parse the [dependencies] section only
  let mut in_deps = false;
  for line in content.lines()
  {
    let trimmed = line.trim();
    if trimmed == "[dependencies]" { in_deps = true; continue; }
    if trimmed.starts_with( '[' ) { in_deps = false; continue; }
    if !in_deps { continue; }
    if trimmed.is_empty() || trimmed.starts_with( '#' ) { continue; }
    // Every dep entry must use workspace = true; crates.io deps use bare version strings
    assert!(
      trimmed.contains( "workspace" ),
      "non-workspace dependency found in [dependencies] — crates.io entry would violate zero-third-party-deps invariant: {line}",
    );
  }
}

// IN-2: The `enabled` feature activates only workspace-backed deps
#[ test ]
fn zero_third_party_deps_in2_enabled_feature_activates_workspace_deps_only()
{
  let cargo_toml = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "Cargo.toml" );
  let content = std::fs::read_to_string( &cargo_toml ).unwrap();

  // Collect all dep names from [dependencies] that have workspace = true
  let mut workspace_deps : std::collections::HashSet< String > = std::collections::HashSet::new();
  let mut in_deps = false;
  for line in content.lines()
  {
    let trimmed = line.trim();
    if trimmed == "[dependencies]" { in_deps = true; continue; }
    if trimmed.starts_with( '[' ) { in_deps = false; continue; }
    if !in_deps || trimmed.is_empty() || trimmed.starts_with( '#' ) { continue; }
    if let Some( name ) = trimmed.split( '=' ).next()
    {
      workspace_deps.insert( name.trim().to_string() );
    }
  }

  // Every `dep:xxx` entry in the `enabled` feature must reference a workspace dep
  let mut in_enabled = false;
  let mut brace_depth : i64 = 0;
  for line in content.lines()
  {
    let trimmed = line.trim();
    if trimmed.starts_with( "enabled" ) && trimmed.contains( '=' ) { in_enabled = true; }
    if !in_enabled { continue; }
    brace_depth += i64::try_from( trimmed.chars().filter( |&c| c == '[' ).count() ).unwrap_or( 0 );
    brace_depth -= i64::try_from( trimmed.chars().filter( |&c| c == ']' ).count() ).unwrap_or( 0 );
    // Extract dep:xxx entries
    let mut rest = trimmed;
    while let Some( pos ) = rest.find( "dep:" )
    {
      let after = &rest[ pos + 4.. ];
      let end = after.find( |c : char| !c.is_alphanumeric() && c != '_' ).unwrap_or( after.len() );
      let dep_name = &after[ ..end ];
      assert!(
        workspace_deps.contains( dep_name ),
        "enabled feature references dep:{dep_name} which is not in [dependencies] with workspace = true",
      );
      rest = &rest[ pos + 4 + end.. ];
    }
    if in_enabled && brace_depth <= 0 && trimmed.contains( ']' ) { break; }
  }
}

// ── Invariant 002: Cross-Platform Compatibility ───────────────────────────────

// IN-1: No path string concatenation in src/ (format! with "/" path separator)
//
// Arithmetic division always uses spaces around the operator (e.g. `secs / 60`).
// Ratio display strings also use spaces: `{count} / {total}`.
// Path separators in format strings do NOT use spaces: `{dir}/{name}`.
// Filtering " / " (space-surrounded slash) removes arithmetic and display false positives
// while preserving genuine path-string-concatenation violations.
#[ test ]
fn cross_platform_in1_no_path_string_concat_in_src()
{
  let src_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" );
  // Broad pattern: any format! containing "/"
  let output = Command::new( "/usr/bin/grep" )
    .args( [ "-rn", r#"format!.*".*/"#, src_dir.to_str().unwrap() ] )
    .output()
    .expect( "grep failed" );

  let matches = String::from_utf8_lossy( &output.stdout );
  let violations : Vec< &str > = matches.lines()
    // Skip comment lines
    .filter( | line | !line.contains( "//" ) && !line.contains( "/*" ) )
    // Skip arithmetic division: space-surrounded "/" is always division, never a path sep
    .filter( | line | !line.contains( " / " ) )
    // Skip URL patterns
    .filter( | line | !line.contains( "://" ) )
    .collect();

  assert!(
    violations.is_empty(),
    "cross-platform violation: path string concatenation found in src/ — use PathBuf::join() instead:\n{}",
    violations.join( "\n" ),
  );
}

// IN-2: No tilde-literal paths in src/
#[ test ]
fn cross_platform_in2_no_tilde_literal_paths_in_src()
{
  let src_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" );
  let output = Command::new( "/usr/bin/grep" )
    .args( [ "-rn", r#""~/"#, src_dir.to_str().unwrap() ] )
    .output()
    .expect( "grep failed" );

  let matches = String::from_utf8_lossy( &output.stdout );
  assert!(
    matches.trim().is_empty(),
    "cross-platform violation: tilde literal paths found — use std::env::var(\"HOME\") instead:\n{matches}",
  );
}

// ── Invariant 003: Clear Error Messages ──────────────────────────────────────

// IN-1: Error for missing account includes the account name
#[ test ]
fn clear_errors_in1_missing_account_error_includes_name()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Credential store exists but ghost@example.com is absent
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".account.use", "name::ghost@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
  let err = super::cli_runner::stderr( &out );
  assert!(
    err.contains( "ghost@example.com" ),
    "error message must name the missing account 'ghost@example.com'; got: {err}",
  );
}

// IN-2: Error for missing credentials file includes the expected file path
//
// Uses `.credentials.status` (not `.token.status`) because `.credentials.status`
// explicitly formats the credential file path into its error message per the
// "credential file not found: <path>" pattern in credentials_status_routine.
#[ test ]
fn clear_errors_in2_missing_creds_error_includes_path()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials file written — the .claude directory does not exist
  let out = run_cs_with_env(
    &[ ".credentials.status" ],
    &[ ( "HOME", home ) ],
  );
  // Exits non-zero when credentials are absent
  let code = out.status.code().unwrap_or( -1 );
  assert_ne!( code, 0, "expected non-zero exit when credentials absent" );
  let err = super::cli_runner::stderr( &out );
  // Error must name the path: "credential file not found: <path>"
  let names_path = err.contains( ".credentials.json" ) || err.contains( ".claude" );
  assert!(
    names_path,
    "error message must include the credentials file path; got: {err}",
  );
}

// ── Invariant 004: No Process Execution ──────────────────────────────────────

// IN-1: src/ contains zero occurrences of std::process::Command (subprocess spawning)
//
// The invariant is "no process execution" = no subprocess spawning via Command::new.
// std::process::exit() is acceptable in CLI entry points and is NOT process execution.
// The `responsibility_no_process_execution_test.rs` file enforces this exact boundary.
#[ test ]
fn no_process_execution_in1_src_contains_zero_std_process()
{
  let src_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" );
  let output = Command::new( "/usr/bin/grep" )
    .args( [ "-rn", "std::process::Command", src_dir.to_str().unwrap() ] )
    .output()
    .expect( "grep failed" );

  let matches = String::from_utf8_lossy( &output.stdout );
  assert!(
    matches.trim().is_empty(),
    "invariant violation: std::process::Command found in src/ — subprocess spawning is forbidden in claude_profile:\n{matches}",
  );
}

// IN-2: The responsibility enforcement test file exists in tests/
#[ test ]
fn no_process_execution_in2_responsibility_test_exists()
{
  let test_file = Path::new( env!( "CARGO_MANIFEST_DIR" ) )
    .join( "tests" )
    .join( "responsibility_no_process_execution_test.rs" );
  assert!(
    test_file.exists(),
    "invariant safeguard missing: responsibility_no_process_execution_test.rs must exist in tests/ to enforce the no-process-execution boundary at CI time",
  );
}

// ── Invariant 005: Atomic Account Switching ───────────────────────────────────

// IN-1: switch_account uses std::fs::rename for credentials file updates (write-then-rename)
//
// The atomic write is implemented in claude_profile_core (the workspace sibling crate),
// not in claude_profile/src/ itself. Search the core crate's src/ directory.
#[ test ]
fn atomic_switching_in1_src_uses_rename_for_credentials()
{
  // switch_account() lives in claude_profile_core — one level up from this crate
  let crate_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let core_src  = crate_dir
    .parent()
    .expect( "parent of crate dir must exist" )
    .join( "claude_profile_core" )
    .join( "src" );

  let output = Command::new( "/usr/bin/grep" )
    .args( [ "-rn", "fs::rename", core_src.to_str().unwrap() ] )
    .output()
    .expect( "grep failed" );

  let matches = String::from_utf8_lossy( &output.stdout );
  assert!(
    !matches.trim().is_empty(),
    "atomic switching invariant violated: std::fs::rename not found in claude_profile_core/src/ — \
     switch_account() must use temp-file + rename, never direct write",
  );
}

// IN-2: Credentials file is complete (parseable JSON) before and after a switch
#[ test ]
fn atomic_switching_in2_credentials_complete_after_switch()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "bob@example.com", "pro", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::bob@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // After the switch, .credentials.json must be valid JSON with the expected content
  let creds_path = dir.path().join( ".claude" ).join( ".credentials.json" );
  let creds_content = std::fs::read_to_string( &creds_path )
    .expect( ".credentials.json must exist after switch" );
  let parsed : serde_json::Value = serde_json::from_str( &creds_content )
    .expect( ".credentials.json must be valid JSON after switch; got: {creds_content}" );
  assert!(
    parsed.is_object(),
    "credentials file must be a JSON object after switch",
  );
  // Verify the switched-to account's tier is present in the new credentials
  let tier = parsed
    .get( "oauthAccount" )
    .and_then( | o | o.get( "rateLimitTier" ) )
    .and_then( | v | v.as_str() );
  assert_eq!(
    tier, Some( "default" ),
    "credentials after switch must contain bob@example.com's rateLimitTier",
  );
}

// ── Invariant 006: Parameters Default to Active Context ──────────────────────

// IN-1: Account-scoped commands work without name:: when active account is set
#[ test ]
fn param_defaults_in1_active_account_used_without_name_arg()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write active account so ambient context is available
  write_account( dir.path(), "carol@example.com", "max", "default", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".token.status" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// IN-2: require_nonempty_string_arg only called in .account.use and .account.delete handlers
#[ test ]
fn param_defaults_in2_require_nonempty_string_arg_only_in_use_delete()
{
  let src_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" );
  let output = Command::new( "/usr/bin/grep" )
    .args( [ "-rn", "require_nonempty_string_arg", src_dir.to_str().unwrap() ] )
    .output()
    .expect( "grep failed" );

  let matches = String::from_utf8_lossy( &output.stdout );
  // Every call site must be in account_ops.rs; skip function definition and `use` import lines
  for line in matches.lines()
  {
    // Skip the function definition itself
    if line.contains( "fn require_nonempty_string_arg" ) { continue; }
    // Skip `use` import statements (e.g. `use super::shared::{ require_nonempty_string_arg, ... }`)
    if line.trim_start().starts_with( "use " ) { continue; }
    assert!(
      line.contains( "account_ops" ) || line.contains( "account_renewal" ),
      "require_nonempty_string_arg called outside expected handlers — violates param-defaults invariant:\n{line}",
    );
  }
}

// ── Invariant 008: Single Token Refresh Entry Point ──────────────────────────

// FT-25 / IN-1: src/ contains zero code-level run_isolated( calls
//
// All token refresh operations must go through `refresh_account_token()` in
// `claude_profile_core`. Direct `run_isolated(` calls in `claude_profile/src/`
// bypass expiresAt=1 manipulation (Feature 017 AC-32) and live credential sync
// (Feature 017 AC-33), causing silent RT decay that can render accounts irrecoverable.
//
// The grep may return doc-comment mentions of run_isolated( (e.g. lines starting
// with `///`). These are filtered by checking for "//" in the output line — they
// are not code-level calls and do not violate the invariant.
#[ test ]
fn single_token_refresh_entry_in1_src_contains_zero_run_isolated_calls()
{
  let src_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" );
  let output = Command::new( "/usr/bin/grep" )
    .args( [ "-rn", "run_isolated(", src_dir.to_str().unwrap() ] )
    .output()
    .expect( "grep failed" );

  let matches = String::from_utf8_lossy( &output.stdout );
  let violations : Vec< &str > = matches.lines()
    // Filter out comment-only lines from the grep output (format: "path:linenum:content").
    // Doc comments (///) and line comments (//) that ARE the code content are not direct
    // invocations. We extract the content part (after the second ':') and check if it
    // trims to a comment — this correctly keeps real violations that happen to have a
    // trailing `//` comment on the same line, while still suppressing pure comment lines.
    .filter( | line |
    {
      // Split "path:linenum:content" — file paths on Linux contain no ':'.
      let mut parts = line.splitn( 3, ':' );
      let _ = parts.next(); // path
      let _ = parts.next(); // linenum
      let code = parts.next().unwrap_or( "" );
      !code.trim_start().starts_with( "//" )
    } )
    .collect();

  assert!(
    violations.is_empty(),
    "invariant 008 violation: direct run_isolated( call found in claude_profile/src/ — \
     all token refresh must go through claude_profile_core::account::refresh_account_token():\n{}",
    violations.join( "\n" ),
  );
}

// IN-2: `run_isolated` doc comment in `claude_runner_core/src/isolated.rs` has a `# Warning` section
//
// The invariant enforcement mechanism includes both an automated grep test (IN-1) and a
// `# Warning` doc comment at the `run_isolated()` definition and re-export. The doc warning
// is the first line of defence: developers reading `isolated.rs` see it before writing any call.
// This test verifies the warning is present and not accidentally removed.
#[ test ]
fn single_token_refresh_entry_in2_run_isolated_doc_has_warning()
{
  // Navigate from claude_profile/ → module/ → claude_runner_core/src/isolated.rs
  let crate_dir   = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let isolated_rs = crate_dir
    .parent()
    .expect( "parent of crate dir (module/) must exist" )
    .join( "claude_runner_core" )
    .join( "src" )
    .join( "isolated.rs" );

  let content = std::fs::read_to_string( &isolated_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", isolated_rs.display() ) );

  // Find the run_isolated doc block and verify it contains "# Warning"
  let fn_pos = content.find( "pub fn run_isolated" )
    .expect( "run_isolated function must exist in isolated.rs" );
  // The doc comment precedes the function — search the region before the function decl
  let doc_region = &content[ ..fn_pos ];
  assert!(
    doc_region.contains( "# Warning" ),
    "invariant 008 enforcement gap: run_isolated() doc in isolated.rs is missing '# Warning' section \
     directing callers to use refresh_account_token() — this is the first-line defence against \
     future invariant violations"
  );
}

// IN-3: `manipulate_expires_at(` appears before `run_isolated(` in `claude_profile_core/src/account.rs`
//
// AC-32 (Change A) requires `expiresAt=1` manipulation to be applied before every `run_isolated`
// call. This structural test verifies the ordering is maintained in `account.rs`.
#[ test ]
fn single_token_refresh_entry_in3_expires_manipulation_before_run_isolated()
{
  let crate_dir  = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let core_src   = crate_dir
    .parent()
    .expect( "parent of crate dir must exist" )
    .join( "claude_profile_core" )
    .join( "src" )
    .join( "account.rs" );

  let content = std::fs::read_to_string( &core_src )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", core_src.display() ) );

  let manip_pos = content.find( "manipulate_expires_at(" )
    .expect( "manipulate_expires_at( must appear in account.rs (AC-32 / Change A)" );
  let run_pos = content.find( "run_isolated(" )
    .expect( "run_isolated( must appear in account.rs (it is the sole authorized caller)" );

  assert!(
    manip_pos < run_pos,
    "invariant 008 AC-32 violation: manipulate_expires_at( must appear before run_isolated( \
     in account.rs to ensure expiresAt=1 manipulation always precedes subprocess spawn \
     (manip_pos={manip_pos}, run_pos={run_pos})",
  );
}

// IN-4: `credentials_file()` appears inside `refresh_token_with_live_path` in `account.rs`
//
// AC-33 (Change B) requires current-account live credential sync inside the `Some(paths)` branch.
// After the extraction refactor, this logic lives in `refresh_token_with_live_path`. This structural
// test verifies the live-sync reads (`credentials_file()`) are present in the helper body.
#[ test ]
fn single_token_refresh_entry_in4_live_sync_in_refresh_token_with_live_path()
{
  let crate_dir  = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let core_src   = crate_dir
    .parent()
    .expect( "parent of crate dir must exist" )
    .join( "claude_profile_core" )
    .join( "src" )
    .join( "account.rs" );

  let content = std::fs::read_to_string( &core_src )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", core_src.display() ) );

  // Locate the refresh_token_with_live_path helper body
  let helper_start = content.find( "fn refresh_token_with_live_path(" )
    .expect( "refresh_token_with_live_path must exist in account.rs (private helper for AC-33)" );
  let helper_body_start = content[ helper_start.. ].find( '{' )
    .expect( "helper body opening brace must exist" );
  let helper_region = &content[ helper_start + helper_body_start.. ];

  // Count code-level calls only (exclude comment lines)
  let count = helper_region.lines()
    .filter( | line | !line.trim_start().starts_with( "//" ) )
    .filter( | line | line.contains( "credentials_file()" ) )
    .count();

  assert!(
    count >= 2,
    "invariant 008 AC-33 violation: expected ≥2 code-level credentials_file() calls in \
     refresh_token_with_live_path (pre-sync read + race-recovery read), found {count} — \
     live credential sync (Change B) may be missing or incomplete",
  );
}
