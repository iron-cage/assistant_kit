//! Integration tests for `.ps` — E6.
//!
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | IT-1 (tc137) | `.ps` exits 0 | P | 0 |
//! | IT-2 (tc141) | `.ps v::0` → no crash | P | 0 |
//! | IT-3 (tc144) | `.ps format::json` → valid JSON | P | 0 |
//! | IT-4 (tc145) | `.ps format::json` no processes → `{"processes":[]}` | P | 0 |
//! | IT-5 | `.ps bogus::x` → exit 1 | N | 1 |
//! | IT-6 | `.ps format::xml` → exit 1 | N | 1 |
//! | IT-7 | `.ps v::3` → exit 1, out of range | N | 1 |
//! | IT-8 | stdout non-empty, stderr empty | P | 0 |
//! | T01 (TSK-463) | `.ps v::1` with ≥1 process (fake, deterministic) → PID column header | P | 0 |
//! | T02 (TSK-463) | `.ps` zero processes (deterministic) → `"no active processes"` | P | 0 |
//! | T05 (TSK-463) | `.ps v::2` → exit 0, non-empty verbose output | P | 0 |

use crate::subprocess_helpers::{ assert_exit, fake_claude_process, run_clv, run_clv_with_env, stderr, stdout };
use tempfile::TempDir;

// ─── E6: ps ────────────────────────────────────────────────────────────

// IT-1 / TC-137
#[ test ]
fn tc137_ps_exits_0()
{
  let out = run_clv( &[ ".ps" ] );
  assert_exit( &out, 0 );
}

// IT-2 / TC-141: v::0 → no crash
#[ test ]
fn tc141_ps_v0_no_crash()
{
  let out = run_clv( &[ ".ps", "v::0" ] );
  assert_exit( &out, 0 );
}

// IT-3 / TC-144: format::json → {"processes":[...]}
#[ test ]
fn tc144_ps_format_json_valid()
{
  let out = run_clv( &[ ".ps", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"processes\"" ), "missing 'processes' key in JSON: {text}" );
  assert!(
    text.trim_start().starts_with( '{' ) || text.contains( '{' ),
    "format::json must produce JSON object: {text}"
  );
}

// IT-4 / TC-145: no processes → {"processes":[]}
#[ test ]
fn tc145_ps_format_json_empty_when_no_processes()
{
  let out = run_clv( &[ ".ps", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"processes\"" ), "format::json must have 'processes' key: {text}" );
}

// IT-5: bogus::x → exit 1 (unknown parameter, rejected by unilang)
#[ test ]
fn it5_ps_bogus_param_exits_1()
{
  let out = run_clv( &[ ".ps", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// IT-6: format::xml → exit 1 (unrecognised format value)
#[ test ]
fn it6_ps_format_xml_exits_1()
{
  let out = run_clv( &[ ".ps", "format::xml" ] );
  assert_exit( &out, 1 );
}

// IT-7: v::3 → exit 1 (verbosity out of the valid 0..=2 range)
#[ test ]
fn it7_ps_v3_out_of_range_exits_1()
{
  let out = run_clv( &[ ".ps", "v::3" ] );
  assert_exit( &out, 1 );
}

// IT-8: output goes to stdout only; stderr is empty
#[ test ]
fn it8_ps_stdout_only_stderr_empty()
{
  let out = run_clv( &[ ".ps" ] );
  assert_exit( &out, 0 );
  assert!( !stdout( &out ).is_empty(), "stdout must be non-empty" );
  assert!( stderr( &out ).is_empty(), "stderr must be empty: {}", stderr( &out ) );
}

// T01 (TSK-463): v::1 table output has a PID column header, given a
// deterministic ≥1-process table (CLR_PROC_DIR fake — see mutation_ps_kill_test.rs
// "Lesson Learned": the real /proc is shared/global and cannot be assumed empty
// or non-empty by a live test).
#[ test ]
fn t01_ps_v1_table_shows_pid_header_with_fake_process()
{
  let fake_proc = TempDir::new().unwrap();
  let fake_pid  = 424_246_u32;
  fake_claude_process( fake_proc.path(), fake_pid );

  let out = run_clv_with_env(
    &[ ".ps", "v::1" ],
    &[ ( "CLR_PROC_DIR", fake_proc.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.to_lowercase().contains( "pid" ), "must contain PID column header: {text}" );
  assert!( text.contains( &fake_pid.to_string() ), "must contain the fake pid: {text}" );
}

// T02 (TSK-463): zero processes (deterministic, CLR_PROC_DIR empty) → the
// table-rendering path's own "no active processes" message, text format.
#[ test ]
fn t02_ps_text_zero_processes_no_active()
{
  let fake_proc = TempDir::new().unwrap();
  let out = run_clv_with_env(
    &[ ".ps" ],
    &[ ( "CLR_PROC_DIR", fake_proc.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  assert!( stdout( &out ).contains( "no active processes" ), "stdout: {}", stdout( &out ) );
}

// T05 (TSK-463): v::2 → exit 0, non-empty verbose output, no crash.
#[ test ]
fn t05_ps_v2_verbose_exit_0()
{
  let out = run_clv( &[ ".ps", "v::2" ] );
  assert_exit( &out, 0 );
  assert!( !stdout( &out ).is_empty(), "v::2 stdout must be non-empty: {}", stdout( &out ) );
}
