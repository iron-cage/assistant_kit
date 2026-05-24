//! Guard test: BUG-172 — no bare `ureq::get()`/`ureq::post()` calls without timeout.
//!
//! Static analysis of `src/lib.rs` source text to prevent regression to bare
//! `ureq::get()`/`ureq::post()` calls that use the global agent with
//! `timeout_read = None`.
//!
//! ## Root Cause
//!
//! `ureq::get()` and `ureq::post()` convenience functions create a one-off
//! `Agent` with default `timeout_read = None`. When the server TCP-connects
//! but stalls the response body, the call blocks for the OS TCP timeout
//! (~75–99 seconds).
//!
//! ## Why Not Caught
//!
//! No unit tests exercise the network path; offline tests use `parse_*`
//! functions with string bodies. The missing timeout is invisible in normal
//! operation.
//!
//! ## Fix Applied
//!
//! All call sites replaced with `http_agent().get()`/`.post()` where
//! `http_agent()` builds an `Agent` with explicit `timeout_read(10s)` and
//! `timeout_connect(5s)`.
//!
//! ## Prevention
//!
//! This guard test greps `src/lib.rs` for bare `ureq::get(` and `ureq::post(`
//! patterns, asserting zero occurrences.
//!
//! ## Pitfall
//!
//! New HTTP call sites must use `http_agent()` — bare `ureq::get()`/
//! `ureq::post()` will be caught by this test.

/// bug_reproducer(BUG-172): Assert zero bare `ureq::get()`/`ureq::post()` in
/// `src/lib.rs`.
///
/// # Fix(BUG-172)
///
/// Root cause: `ureq::get()`/`ureq::post()` use global Agent with
/// `timeout_read = None`, causing indefinite blocking on stalled TCP
/// connections.
/// Pitfall: new call sites must use `http_agent()` helper, not bare
/// `ureq::*()`.
#[ test ]
fn test_bug172_mre_no_bare_ureq_get_in_lib()
{
  let manifest_dir = std::env::var( "CARGO_MANIFEST_DIR" )
    .expect( "CARGO_MANIFEST_DIR not set" );
  let source = std::fs::read_to_string( format!( "{manifest_dir}/src/lib.rs" ) )
    .expect( "failed to read src/lib.rs" );

  let bare_get_count = source.matches( "ureq::get(" ).count();
  assert_eq!(
    bare_get_count, 0,
    "BUG-172: found {bare_get_count} bare ureq::get() call(s) — use http_agent().get() instead",
  );

  let bare_post_count = source.matches( "ureq::post(" ).count();
  assert_eq!(
    bare_post_count, 0,
    "BUG-172: found {bare_post_count} bare ureq::post() call(s) — use http_agent().post() instead",
  );
}
