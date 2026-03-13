//! Model and budget parameter builder method tests (TSK-076)
//!
//! ## Purpose
//!
//! Verify the three model/budget `with_*()` methods add the correct CLI flags.
//!
//! ## Evidence
//!
//! - `with_effort(EffortLevel::High)` adds `--effort high`
//! - `with_effort(EffortLevel::Max)` adds `--effort max` (NOT `maximum`)
//! - `with_fallback_model("claude-haiku-4-5")` adds `--fallback-model claude-haiku-4-5`
//! - `with_max_budget_usd(0.5)` adds `--max-budget-usd` with clean float string
//!
//! ## Test Coverage Matrix
//!
//! | Method | flag present | float edge cases |
//! |--------|-------------|-----------------|
//! | `with_effort` | ✅ | — |
//! | `with_fallback_model` | ✅ | — |
//! | `with_max_budget_usd` | ✅ | ✅ |
//!
//! ## Float Edge Cases for `with_max_budget_usd`
//!
//! The builder performs no validation — all f64 values pass through to the CLI.
//! Semantically invalid values (NaN, Infinity, negative) produce unusual strings
//! that the Claude CLI will likely reject at runtime, but the builder itself does
//! not error. This is documented behavior, not a bug.
//!
//! | Input | Serialized value | Tested here |
//! |-------|-----------------|-------------|
//! | 0.0 | "0" | ✅ |
//! | 1.0 | "1" | see pattern_e_empty_and_edge_cases_test.rs |
//! | 0.5 | "0.5" | ✅ (clean_float_serialization) |
//! | -1.0 | "-1" | ✅ |
//! | f64::NAN | "NaN" | ✅ |
//! | f64::INFINITY | "inf" | ✅ |
//! | f64::NEG_INFINITY | "-inf" | ✅ |

use claude_runner_core::{ ClaudeCommand, EffortLevel };

fn args_of( cmd: &ClaudeCommand ) -> Vec<String> {
  let c = cmd.build_command_for_test();
  c.get_args().map( |a| a.to_string_lossy().to_string() ).collect()
}

// with_effort

#[test]
fn with_effort_low_adds_flag() {
  let cmd = ClaudeCommand::new().with_effort( EffortLevel::Low );
  let args = args_of( &cmd );
  assert!( args.contains( &"--effort".to_string() ) );
  assert!( args.contains( &"low".to_string() ) );
}

#[test]
fn with_effort_high_adds_flag() {
  let cmd = ClaudeCommand::new().with_effort( EffortLevel::High );
  let args = args_of( &cmd );
  assert!( args.contains( &"--effort".to_string() ) );
  assert!( args.contains( &"high".to_string() ) );
}

#[test]
fn with_effort_max_uses_max_not_maximum() {
  // Fix(issue-effort-level-max): Max maps to "max" not "maximum"
  let cmd = ClaudeCommand::new().with_effort( EffortLevel::Max );
  let args = args_of( &cmd );
  assert!( args.contains( &"max".to_string() ), "must use 'max': {args:?}" );
  assert!( !args.contains( &"maximum".to_string() ), "must NOT use 'maximum'" );
}

// with_fallback_model

#[test]
fn with_fallback_model_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_fallback_model( "claude-haiku-4-5" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--fallback-model".to_string() ) );
  assert!( args.contains( &"claude-haiku-4-5".to_string() ) );
}

// with_max_budget_usd

#[test]
fn with_max_budget_usd_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_max_budget_usd( 1.0 );
  let args = args_of( &cmd );
  assert!( args.contains( &"--max-budget-usd".to_string() ) );
}

#[test]
fn with_max_budget_usd_clean_float_serialization() {
  // Fix(issue-max-budget-float): Clean float serialization — no trailing zeros
  let cmd = ClaudeCommand::new().with_max_budget_usd( 0.5 );
  let args = args_of( &cmd );
  // Should be "0.5" not "0.50000..."
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "flag not found" );
  let value = &args[ pos + 1 ];
  assert!( !value.starts_with( "0.50" ), "float must not have excess trailing zeros: {value}" );
  assert_eq!( value, "0.5" );
}

// ── with_max_budget_usd float edge cases ─────────────────────────────────────

#[test]
fn with_max_budget_usd_zero_serializes_as_zero() {
  // 0.0 → "0" (Rust Display format for whole-number f64)
  let args = args_of( &ClaudeCommand::new().with_max_budget_usd( 0.0 ) );
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "flag not found" );
  assert_eq!( &args[ pos + 1 ], "0" );
}

#[test]
fn with_max_budget_usd_negative_passes_through() {
  // Builder performs no validation — negative values pass through as "-1"
  // Claude CLI will likely reject at runtime; builder itself does not error
  let args = args_of( &ClaudeCommand::new().with_max_budget_usd( -1.0 ) );
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "flag not found" );
  assert_eq!( &args[ pos + 1 ], "-1" );
}

#[test]
fn with_max_budget_usd_nan_passes_through_as_nan_string() {
  // Builder does not validate f64 semantics — NaN passes through as "NaN"
  // Claude CLI will reject this at runtime; builder itself does not error
  let args = args_of( &ClaudeCommand::new().with_max_budget_usd( f64::NAN ) );
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "flag not found" );
  assert_eq!( &args[ pos + 1 ], "NaN" );
}

#[test]
fn with_max_budget_usd_infinity_passes_through_as_inf_string() {
  // Builder does not validate f64 semantics — INFINITY passes through as "inf"
  let args = args_of( &ClaudeCommand::new().with_max_budget_usd( f64::INFINITY ) );
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "flag not found" );
  assert_eq!( &args[ pos + 1 ], "inf" );
}

#[test]
fn with_max_budget_usd_neg_infinity_passes_through_as_neg_inf_string() {
  // Builder does not validate f64 semantics — NEG_INFINITY passes through as "-inf"
  let args = args_of( &ClaudeCommand::new().with_max_budget_usd( f64::NEG_INFINITY ) );
  let pos = args.iter().position( |a| a == "--max-budget-usd" ).expect( "flag not found" );
  assert_eq!( &args[ pos + 1 ], "-inf" );
}
