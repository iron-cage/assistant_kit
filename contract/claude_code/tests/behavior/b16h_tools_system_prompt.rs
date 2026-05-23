#![ allow( clippy::doc_markdown ) ]
//! B16h: Tool *definitions* (~12k tokens) remain in the assembled system prompt even when
//! `--tools ""` is given — invocation is blocked but the token cost is unchanged.
//!
//! This is the open hypothesis (❓ Uncertain, 60%) from B16. Two mutually exclusive models:
//! - **H1** (60%): `--tools ""` blocks invocation only; tool definitions remain in the
//!   assembled system prompt — token cost is unchanged (~12k tokens still consumed).
//! - **H2** (25%): `--tools ""` strips definitions entirely — saves ~12k tokens AND blocks.
//!
//! Confirmation requires a live Claude API invocation with `--output-format json`
//! to compare `input_tokens` between `--tools ""` and default. Excluded from default test
//! filter because it requires live Anthropic credentials (`ANTHROPIC_API_KEY`).
//!
//! See `docs/behavior/readme.md` B16h / E32 for full evidence record.

/// B16h: compare `input_tokens` between `--tools ""` and default to confirm H1 vs H2.
///
/// Prefixed `lim_it_` so it is excluded by the nextest `default-filter` in
/// `.config/nextest.toml` (pattern `!test(lim_it)`) and only runs when explicitly requested.
///
/// Run manually with:
/// ```bash
/// cargo nextest run -p claude_code --test behavior lim_it_b16h
/// ```
#[ test ]
fn lim_it_b16h_tools_system_prompt_token_comparison()
{
  let Some( claude ) = super::find_claude_binary() else
  {
    eprintln!( "skip: `claude` binary not found on PATH" );
    return;
  };

  // Requires live API credentials — skip if not set.
  if std::env::var( "ANTHROPIC_API_KEY" ).is_err()
  {
    eprintln!( "skip: ANTHROPIC_API_KEY not set; B16h requires live API" );
    return;
  }

  // Baseline: full tools (no --tools flag).
  let baseline = std::process::Command::new( &claude )
    .args( [ "--print", "--output-format", "json", "hi" ] )
    .output()
    .expect( "run baseline claude --print" );

  // With tools disabled.
  let no_tools = std::process::Command::new( &claude )
    .args( [ "--tools", "", "--print", "--output-format", "json", "hi" ] )
    .output()
    .expect( "run claude --tools '' --print" );

  // Extract input_tokens from each JSON response.
  let baseline_tokens = extract_input_tokens( &super::stdout( &baseline ) );
  let no_tools_tokens = extract_input_tokens( &super::stdout( &no_tools ) );

  match ( baseline_tokens, no_tools_tokens )
  {
    ( Some( b ), Some( n ) ) =>
    {
      let diff = b.saturating_sub( n );
      // H2 predicts ~12_000 token reduction; H1 predicts < 1_000 difference.
      // Record the result rather than asserting — this is a measurement test.
      eprintln!( "B16h measurement: baseline={b} no_tools={n} diff={diff}" );
      if diff > 8_000
      {
        eprintln!( "B16h → H2 supported: tool definitions stripped (~{diff} tokens saved)" );
      }
      else
      {
        eprintln!( "B16h → H1 supported: tool definitions present (only {diff} token diff)" );
      }
      // The test itself passes either way — it is a measurement, not a pass/fail assertion.
      // Update behavior/readme.md B16h status based on the measurement result.
    }
    _ =>
    {
      eprintln!(
        "skip: could not parse input_tokens from JSON output.\n\
         baseline stdout: {}\nno_tools stdout: {}",
        super::stdout( &baseline ),
        super::stdout( &no_tools )
      );
    }
  }
}

/// Extract `"input_tokens": N` from a `--output-format json` response.
fn extract_input_tokens( json : &str ) -> Option< u64 >
{
  // Quick text extraction — avoids pulling in a JSON parser dependency.
  let key = "\"input_tokens\":";
  let start = json.find( key )? + key.len();
  let rest = json[ start .. ].trim_start();
  let end = rest.find( |c : char| !c.is_ascii_digit() )?;
  rest[ ..end ].parse().ok()
}
