//! Spawn guard for the `cli` and `cli_args_test` test binaries.
//!
//! Both binaries include this file via `#[ path ]`, and their subprocess
//! helpers call it before every `claude_version` spawn.  It refuses a spawn
//! outside a container, and `mode::history` under the inherited `HOME`.

/// Assert that the current process is inside a container or has bypassed the guard.
///
/// # Panics
///
/// Panics if neither a container environment nor the `VERB_LAYER=l0` bypass is detected.
#[ inline ]
pub fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/claude_version && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

/// Assert that `args` doesn't select `mode::history`, for a spawn that inherits `HOME`.
///
/// `history_runner` names the caller's helper that gives the subprocess a
/// `HOME` the test owns, for the panic message.
///
/// # Panics
///
/// Panics if `args` contains `mode::history`.
#[ inline ]
pub fn assert_no_history_mode( args : &[ &str ], history_runner : &str )
{
  // Fix(BUG-581): refuse `mode::history` under the inherited `HOME`.
  // Root cause: the release fetch writes `{HOME}/.claude/.transient/version_history_cache.json`,
  //   and runbox mounts the developer's real `~/.claude` read-write at that `HOME`.
  // Pitfall: a read command that refreshes a cache writes like any other; it needs a `HOME` the test owns.
  assert!(
    !args.contains( &"mode::history" ),
    "`mode::history` writes a release cache under HOME — run it through {history_runner}() (BUG-581)"
  );
}
