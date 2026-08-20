//! BUG-537 reproducer: `data_fmt` terminal-width detection compiled out workspace-wide.
//!
//! # Root Cause (BUG-537)
//!
//! `data_fmt`'s `resolve_terminal_width()` resolves width in tiers: explicit config
//! override → `$COLUMNS` env var → runtime TTY probe (`terminal_size` crate) → hard
//! fallback `120`. The TTY probe is gated behind `data_fmt`'s `terminal_size` cargo
//! feature, which is NOT part of its `enabled` feature. Every consumer manifest in
//! this workspace enabled only `data_fmt/enabled`, so the probe was compiled out and
//! every rendered table (`clr ps`, `clr tools`, `clp .usage`/`.accounts`, `clv`
//! config/params) hard-wrapped at 120 columns regardless of the real terminal width —
//! on a 200-column terminal, `clr ps` squeezed `Absolute Path` and `Task` into
//! ~27-character wrapped cells while half the screen stayed blank.
//!
//! # Why Not Caught (BUG-537)
//!
//! Tests run with piped stdout (no TTY), where the probe correctly returns `None` and
//! the 120 fallback is the RIGHT answer — so no rendering test could ever observe the
//! missing feature. BUG-300 even documented "the terminal-width fallback (120
//! columns)" as ambient behavior while fixing an unrelated caption defect on top of
//! it. The defect is visible only interactively, which no automated surface covered.
//!
//! # Fix Applied (BUG-537)
//!
//! Every feature array that enables `data_fmt/enabled` now also enables
//! `data_fmt/terminal_size` (4 manifests: `claude_runner`, `claude_runner_core`,
//! `claude_profile`, `claude_version`). On a TTY the table now fills the detected
//! width; piped/non-TTY output keeps the 120 fallback, so scripted consumers and
//! this test suite see byte-identical rendering.
//!
//! # Prevention (BUG-537)
//!
//! A feature that changes user-visible behavior must be asserted at the manifest
//! level when its effect is unobservable under test harness conditions (no TTY).
//! This test pins the `enabled`→`terminal_size` pairing across all workspace
//! consumers so the next `data_fmt` consumer (or a dependency refactor) cannot
//! silently regress to the 120-column cap — the same one-guard-per-invariant lesson
//! as BUG-292/BUG-324's shared eligibility floor.
//!
//! # Pitfall (BUG-537)
//!
//! `$COLUMNS` (Tier 1) is a bash shell variable that is NOT exported to child
//! processes — it never fires for a spawned binary and cannot substitute for the
//! compiled-in probe. Do not "fix" width issues by documenting a `COLUMNS=` prefix;
//! that is a diagnostic workaround, not a default behavior.

/// Every workspace manifest that enables `data_fmt/enabled` must pair it with
/// `data_fmt/terminal_size`, or its tables silently hard-wrap at the 120-column
/// fallback on every terminal (BUG-537).
#[ doc = "bug_reproducer(BUG-537)" ]
#[ test ]
fn t01_every_data_fmt_consumer_enables_terminal_size()
{
  let own = env!( "CARGO_MANIFEST_DIR" );
  let manifests = [
    format!( "{own}/Cargo.toml" ),
    format!( "{own}/../claude_runner_core/Cargo.toml" ),
    format!( "{own}/../claude_profile/Cargo.toml" ),
    format!( "{own}/../claude_version/Cargo.toml" ),
  ];
  for path in &manifests
  {
    let content = std::fs::read_to_string( path )
      .unwrap_or_else( | e | panic!( "BUG-537: cannot read manifest {path}: {e} — if the workspace layout changed, update this reproducer's manifest list" ) );
    if !content.contains( r#""data_fmt/enabled""# ) { continue; }
    assert!(
      content.contains( r#""data_fmt/terminal_size""# ),
      "BUG-537: {path} enables data_fmt/enabled without data_fmt/terminal_size — \
       the TTY width probe is compiled out and every table in that binary \
       hard-wraps at the 120-column fallback regardless of real terminal width",
    );
  }
}
