//! Unix-only integration tests.
#![ cfg( unix ) ]
//! Journal Attribution Tests (EC-23..EC-29, task 542)
//!
//! Every journal event emitted by `clr` must answer "which account, which
//! agent": `user`/`host` always set, `dir` defaulting to the process cwd when
//! no explicit `--dir` was given, `agent_id` composed as
//! `{user}@{host}{abs_dir}/` via `claude_journal::compose_agent_id`, and
//! `account` resolved through the hierarchy `CLR_ACCOUNT` env → active-account
//! marker in the credential store → absent.
//!
//! ## Test Layout (task 542 Test Matrix)
//!
//! - EC-23: Print execution without `--dir` → `dir` == process cwd; `agent_id` composed from it
//! - EC-24: Print execution with `--dir Y` → `dir` == Y preserved; `agent_id` uses Y
//! - EC-25: `CLR_ACCOUNT=test.acct` set → `account` == `"test.acct"` (env override wins)
//! - EC-26: Identity unresolvable, no env → `account` absent; `user`/`host`/`agent_id` still set
//! - EC-27: Redirect seat active (marker holds `kimi`) → `account` == `"kimi"` (profile name, no secret)
//! - EC-28: Retry fired → `retry` event carries the same `account`/`agent_id` as its `execution`
//! - EC-29: Interactive session from dir X, no `--dir` → `dir` == X; `agent_id` == `{user}@{host}X/`; `account` set
//!
//! Identity env is pinned (`USER=tester`, `HOSTNAME=testhost`) and the
//! credential store is isolated by pointing `PRO` at a private temp dir, so no
//! test reads the live store and no assertion depends on the host machine.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, read_journal_content };
use std::process::Command;

/// Pinned identity for every test in this file.
const USER : &str = "tester";
const HOST : &str = "testhost";

/// Invoke `clr` with pinned identity env, an isolated `PRO` store root, and a
/// private journal dir; return `(output, journal_dir)`.
///
/// `account_env` = `Some(v)` sets `CLR_ACCOUNT=v`; `None` removes it.
/// `pro_root` is exported as `PRO` — pass an empty temp dir to make the
/// account identity unresolvable, or one seeded with an active marker to
/// resolve a store-derived account.
fn run_attributed(
  args        : &[ &str ],
  cwd         : Option< &std::path::Path >,
  account_env : Option< &str >,
  pro_root    : &std::path::Path,
  fake_body   : &str,
) -> ( std::process::Output, tempfile::TempDir )
{
  let ( _fake, path ) = fake_claude_dir( fake_body );
  let journal_dir = tempfile::TempDir::new().expect( "journal tmpdir" );
  let jd = journal_dir.path().display().to_string();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut cmd = Command::new( bin );
  cmd
    .args( args )
    .args( [ "--max-sessions", "0", "--journal", "full", "--journal-dir", &jd ] )
    .arg( "x" )
    .env( "PATH", &path )
    .env( "HOME", "/tmp/clr-isolated-home" )
    .env( "USER", USER )
    .env( "HOSTNAME", HOST )
    .env( "PRO", pro_root )
    .env_remove( "CLR_JOURNAL" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLR_TIMEOUT" )
    .env_remove( "_CLR_DEFAULT_TIMEOUT" )
    .env_remove( "CLR_DIR" );
  match account_env
  {
    Some( v ) => { cmd.env( "CLR_ACCOUNT", v ); }
    None      => { cmd.env_remove( "CLR_ACCOUNT" ); }
  }
  if let Some( d ) = cwd { cmd.current_dir( d ); }
  let out = cmd.output().expect( "failed to invoke clr binary" );
  ( out, journal_dir )
}

/// Extract the first journal line of the given event type.
fn event_line( journal : &str, event_type : &str ) -> String
{
  let needle = format!( r#""type":"{event_type}""# );
  journal
    .lines()
    .find( | l | l.contains( &needle ) )
    .unwrap_or_else( || panic!( "no {event_type} event in journal:\n{journal}" ) )
    .to_owned()
}

/// Extract a string field's value from a JSONL event line.
fn field_value( line : &str, field : &str ) -> String
{
  let key = format!( r#""{field}":""# );
  let start = line.find( &key )
    .unwrap_or_else( || panic!( "field {field} absent from event line: {line}" ) )
    + key.len();
  let rest = &line[ start.. ];
  let end = rest.find( '"' ).expect( "unterminated string value" );
  rest[ ..end ].to_owned()
}

/// EC-23: a print execution without `--dir` journals `dir` == the process cwd
/// and an `agent_id` composed from it (`{user}@{host}{cwd}/`).
#[ test ]
fn ec23_execution_without_dir_falls_back_to_cwd()
{
  let pro = tempfile::TempDir::new().expect( "pro tmpdir" );
  let work = tempfile::TempDir::new().expect( "work tmpdir" );
  let work_abs = work.path().canonicalize().expect( "canonicalize work dir" );
  let ( out, jdir ) = run_attributed( &[ "-p" ], Some( &work_abs ), None, pro.path(), "exit 0" );
  assert!( out.status.success(), "stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let line = event_line( &read_journal_content( jdir.path() ), "execution" );
  assert_eq!( field_value( &line, "dir" ), work_abs.display().to_string(), "dir must fall back to cwd: {line}" );
  assert_eq!( field_value( &line, "user" ), USER, "{line}" );
  assert_eq!( field_value( &line, "host" ), HOST, "{line}" );
  assert_eq!(
    field_value( &line, "agent_id" ),
    format!( "{USER}@{HOST}{}/", work_abs.display() ),
    "agent_id must be composed from the effective dir: {line}",
  );
}

/// EC-24: an explicit `--dir Y` is preserved verbatim in the event and
/// `agent_id` is composed from Y, not the cwd (AC-04).
#[ test ]
fn ec24_execution_with_explicit_dir_preserved()
{
  let pro = tempfile::TempDir::new().expect( "pro tmpdir" );
  let target = tempfile::TempDir::new().expect( "target tmpdir" );
  let target_abs = target.path().canonicalize().expect( "canonicalize target" ).display().to_string();
  let ( out, jdir ) = run_attributed( &[ "-p", "--dir", &target_abs ], None, None, pro.path(), "exit 0" );
  assert!( out.status.success(), "stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let line = event_line( &read_journal_content( jdir.path() ), "execution" );
  assert_eq!( field_value( &line, "dir" ), target_abs, "explicit --dir must win over cwd fallback: {line}" );
  assert_eq!(
    field_value( &line, "agent_id" ),
    format!( "{USER}@{HOST}{target_abs}/" ),
    "agent_id must use the explicit dir: {line}",
  );
}

/// EC-25: a non-empty `CLR_ACCOUNT` env var overrides every other account
/// source (first rung of the resolution hierarchy).
#[ test ]
fn ec25_clr_account_env_override_wins()
{
  let pro = tempfile::TempDir::new().expect( "pro tmpdir" );
  let ( out, jdir ) = run_attributed( &[ "-p" ], None, Some( "test.acct" ), pro.path(), "exit 0" );
  assert!( out.status.success(), "stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let line = event_line( &read_journal_content( jdir.path() ), "execution" );
  assert_eq!( field_value( &line, "account" ), "test.acct", "CLR_ACCOUNT must win: {line}" );
}

/// EC-26: when no identity is resolvable (no `CLR_ACCOUNT`, empty store),
/// `account` is absent — but `user`/`host`/`agent_id` are still set and the
/// event is still emitted (attribution never blocks journaling).
#[ test ]
fn ec26_unresolvable_account_absent_other_attribution_present()
{
  let pro = tempfile::TempDir::new().expect( "pro tmpdir" );
  let ( out, jdir ) = run_attributed( &[ "-p" ], None, None, pro.path(), "exit 0" );
  assert!( out.status.success(), "stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let line = event_line( &read_journal_content( jdir.path() ), "execution" );
  assert!( !line.contains( r#""account":"# ), "account must be absent when unresolvable: {line}" );
  assert_eq!( field_value( &line, "user" ), USER, "{line}" );
  assert_eq!( field_value( &line, "host" ), HOST, "{line}" );
  assert!( line.contains( r#""agent_id":"# ), "agent_id must still be set: {line}" );
}

/// EC-27: with a redirect seat active (this machine's `_active_{host}_{user}`
/// marker holding a profile name), `account` reports that name — never token
/// material (second rung: store-derived identity).
#[ test ]
fn ec27_store_active_marker_resolves_account()
{
  let pro = tempfile::TempDir::new().expect( "pro tmpdir" );
  let store = pro.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).expect( "create store dirs" );
  std::fs::write( store.join( format!( "_active_{HOST}_{USER}" ) ), "kimi\n" )
    .expect( "write active marker" );
  let ( out, jdir ) = run_attributed( &[ "-p" ], None, None, pro.path(), "exit 0" );
  assert!( out.status.success(), "stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let line = event_line( &read_journal_content( jdir.path() ), "execution" );
  assert_eq!( field_value( &line, "account" ), "kimi", "marker-derived profile name expected: {line}" );
}

/// EC-28: a `retry` event carries the same `account` and `agent_id` as the
/// `execution` event of the run it belongs to — attribution is uniform across
/// event types because stamping is centralized at the append boundary.
#[ test ]
fn ec28_retry_event_carries_same_attribution_as_execution()
{
  let pro = tempfile::TempDir::new().expect( "pro tmpdir" );
  let count_dir = tempfile::TempDir::new().expect( "count tmpdir" );
  let count_path = count_dir.path().join( "count" ).display().to_string();
  let body = format!( "if [ -f \"{count_path}\" ]; then exit 0; fi\ntouch \"{count_path}\"\nexit 2" );
  let ( out, jdir ) = run_attributed(
    &[ "-p", "--retry-on-transient", "1", "--transient-delay", "0" ],
    None,
    Some( "retry.acct" ),
    pro.path(),
    &body,
  );
  assert!( out.status.success(), "stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let journal = read_journal_content( jdir.path() );
  let retry_line = event_line( &journal, "retry" );
  let exec_line  = event_line( &journal, "execution" );
  assert_eq!( field_value( &retry_line, "account" ), "retry.acct", "{retry_line}" );
  assert_eq!(
    field_value( &retry_line, "account" ),
    field_value( &exec_line, "account" ),
    "retry and execution must agree on account",
  );
  assert_eq!(
    field_value( &retry_line, "agent_id" ),
    field_value( &exec_line, "agent_id" ),
    "retry and execution must agree on agent_id",
  );
}

/// EC-29: an interactive session launched from dir X without `--dir` journals
/// `dir` == X, `agent_id` == `{user}@{host}X/`, and a resolved `account`.
#[ test ]
fn ec29_interactive_event_carries_full_attribution()
{
  let pro = tempfile::TempDir::new().expect( "pro tmpdir" );
  let work = tempfile::TempDir::new().expect( "work tmpdir" );
  let work_abs = work.path().canonicalize().expect( "canonicalize work dir" );
  let ( out, jdir ) = run_attributed(
    &[ "--interactive" ],
    Some( &work_abs ),
    Some( "session.acct" ),
    pro.path(),
    "exit 0",
  );
  assert!( out.status.success(), "stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let line = event_line( &read_journal_content( jdir.path() ), "interactive" );
  assert_eq!( field_value( &line, "dir" ), work_abs.display().to_string(), "{line}" );
  assert_eq!(
    field_value( &line, "agent_id" ),
    format!( "{USER}@{HOST}{}/", work_abs.display() ),
    "{line}",
  );
  assert_eq!( field_value( &line, "account" ), "session.acct", "{line}" );
}
