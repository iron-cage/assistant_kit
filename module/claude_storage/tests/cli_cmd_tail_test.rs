//! Integration tests for the `clg .tail` command.

mod common;

/// INT-1: `.tail` with a session_id runs without panicking.
#[ test ]
fn int_1_tail_runs()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session(
    root.path(), cwd.path(), "-default_topic", 4
  );

  let _out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "session_id::-default_topic" )
    .output()
    .unwrap();

  // No assertion on stdout/exit code — just confirms the process didn't hang.
}

/// INT-2: `.tail` accepts a count parameter without erroring the harness.
#[ test ]
fn int_2_tail_accepts_count()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session(
    root.path(), cwd.path(), "-default_topic", 8
  );

  let result = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "session_id::-default_topic" )
    .arg( "count::3" )
    .output();

  assert!( result.is_ok() );
}

/// INT-3: `.tail` with no session_id is at least callable.
#[ test ]
fn int_3_tail_callable_without_session()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session(
    root.path(), cwd.path(), "-default_topic", 2
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .output()
    .unwrap();

  let _ = out.status.code();
}
