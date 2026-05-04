//! Unit tests for the adapter and output modules.
//!
//! Covers matrix IDs A-01..A-37 (adapter), O-01..O-19 (output), D-01..D-12 (`format_duration`).
//! All tests call library functions directly — no binary invocation.
//!
//! ## Test Matrix
//!
//! ### A — Adapter (`argv_to_unilang_tokens`)
//!
//! | ID   | Test Function | Condition | P/N |
//! |------|---------------|-----------|-----|
//! | A-01 | `adapter_empty_argv_returns_help` | empty argv → help | P |
//! | A-02 | `adapter_single_command` | single command word → command token | P |
//! | A-03 | `adapter_dot_returns_help` | `.` → help | P |
//! | A-04 | `adapter_rejects_double_dash_help` | `--help` → unexpected flag error | N |
//! | A-05 | `adapter_rejects_dash_h` | `-h` → unexpected flag error | N |
//! | A-06 | `adapter_rejects_unknown_flag` | `-verbose` (unknown flag) → error | N |
//! | A-07 | `adapter_rejects_param_as_command` | `key::value` as first arg → error | N |
//! | A-08 | `adapter_verbosity_alias_0` | `v::0` → verbosity 0 | P |
//! | A-09 | `adapter_verbosity_alias_1` | `v::1` → verbosity 1 | P |
//! | A-10 | `adapter_verbosity_alias_2` | `v::2` → verbosity 2 | P |
//! | A-11 | `adapter_verbosity_out_of_range` | `v::3` → error | N |
//! | A-12 | `adapter_verbosity_non_numeric` | `v::abc` → error | N |
//! | A-13 | `adapter_verbosity_canonical_key` | `verbosity::1` (canonical) → passes | P |
//! | A-14 | `adapter_dry_true_normalizes` | `dry::true` → `dry::1` | P |
//! | A-15 | `adapter_dry_false_normalizes` | `dry::false` → `dry::0` | P |
//! | A-16 | `adapter_dry_1_passthrough` | `dry::1` → unchanged | P |
//! | A-17 | `adapter_dry_0_passthrough` | `dry::0` → unchanged | P |
//! | A-18 | `adapter_dry_invalid_value` | `dry::maybe` → error | N |
//! | A-19 | `adapter_missing_separator` | bareword without `::` → error | N |
//! | A-20 | `adapter_rejects_flag_in_params` | `--verbose` after command → error | N |
//! | A-21 | `adapter_duplicate_last_wins` | duplicate key → last value wins | P |
//! | A-22 | `adapter_alias_then_canonical_dedup` | alias then canonical → deduped | P |
//! | A-23 | `adapter_multi_separator_split` | `key::val::extra` → split at first `::` | P |
//! | A-24 | `adapter_empty_value_allowed` | `key::` → empty value allowed | P |
//! | A-25 | `adapter_verbosity_negative` | `v::-1` → error (u8 parse) | N |
//! | A-26 | `adapter_verbosity_decimal` | `v::2.5` → error | N |
//! | A-27 | `adapter_verbosity_empty` | `v::` → error (empty) | N |
//! | A-28 | `adapter_multiple_params` | multiple `k::v` pairs → all assembled | P |
//! | A-29 | `adapter_help_as_command` | `.help` as sole arg → help | P |
//! | A-30 | `adapter_verbosity_overflow` | `v::256` → error (u8 overflow) | N |
//! | A-31 | `adapter_dot_help_in_second_position` | `.help` in second position → routes to help | P |
//! | A-32 | `adapter_bare_help_in_second_position` | `help` in second position → routes to help | P |
//! | A-33 | `adapter_bare_help_as_sole_arg` | bare `help` as sole argv → routes to help | P |
//! | A-34 | `adapter_rejects_double_dash_version` | `--version` → unexpected flag error | N |
//! | A-35 | `adapter_rejects_dash_v_version` | `-V` → unexpected flag error | N |
//! | A-36 | `adapter_field_presence_bool_true_normalises` | `account::true` → `account::1` | P |
//! | A-37 | `adapter_field_presence_bool_false_normalises` | `account::false` → `account::0` | P |
//!
//! ### O — Output format / verbosity / `json_escape`
//!
//! | ID   | Test Function | Condition | P/N |
//! |------|---------------|-----------|-----|
//! | O-01 | `output_format_text` | `fmt::text` → `OutputFormat::Text` | P |
//! | O-02 | `output_format_json` | `fmt::json` → `OutputFormat::Json` | P |
//! | O-03 | `output_format_xml_rejected` | `fmt::xml` → error | N |
//! | O-04 | `output_format_default` | no `fmt::` → default `Text` | P |
//! | O-05 | `output_format_case_json_rejected` | `fmt::JSON` (uppercase) → error | N |
//! | O-06 | `output_format_case_text_rejected` | `fmt::Text` (capitalized) → error | N |
//! | O-07 | `output_verbosity_0` | `v::0` → verbosity 0 | P |
//! | O-08 | `output_verbosity_1` | `v::1` → verbosity 1 | P |
//! | O-09 | `output_verbosity_2` | `v::2` → verbosity 2 | P |
//! | O-10 | `output_verbosity_default` | no `v::` → default verbosity 1 | P |
//! | O-11 | `json_escape_plain` | plain string unchanged | P |
//! | O-12 | `json_escape_quote` | `"` → `\"` | P |
//! | O-13 | `json_escape_backslash` | `\` → `\\` | P |
//! | O-14 | `json_escape_newline` | `\n` → `\n` (escaped) | P |
//! | O-15 | `json_escape_tab` | `\t` → `\t` (escaped) | P |
//! | O-16 | `json_escape_cr` | `\r` → `\r` (escaped) | P |
//! | O-17 | `json_escape_empty` | empty string → empty string | P |
//! | O-18 | `json_escape_mixed` | string with multiple special chars → all escaped | P |
//! | O-19 | `json_escape_unicode` | non-ASCII unicode passthrough unchanged | P |
//!
//! ### D — `format_duration_secs`
//!
//! | ID   | Test Function | Condition | P/N |
//! |------|---------------|-----------|-----|
//! | D-01 | `dur_zero_seconds_shows_0m` | 0s → `"0m"` | P |
//! | D-02 | `dur_sub_minute_rounds_to_0m` | 1s → `"0m"` | P |
//! | D-03 | `dur_59s_rounds_to_0m` | 59s → `"0m"` | P |
//! | D-04 | `dur_60s_shows_1m` | 60s → `"1m"` | P |
//! | D-05 | `dur_3599s_shows_59m` | 3599s → `"59m"` | P |
//! | D-06 | `dur_3600s_shows_1h_no_minutes` | 3600s → `"1h"` | P |
//! | D-07 | `dur_3660s_shows_1h_1m` | 3660s → `"1h 1m"` | P |
//! | D-08 | `dur_86400s_shows_1d_no_hours` | 86400s → `"1d"` | P |
//! | D-09 | `dur_86460s_shows_1d_1m` | 86460s → `"1d 1m"` | P |
//! | D-10 | `dur_90000s_shows_1d_1h_no_minutes` | 90000s → `"1d 1h"` | P |
//! | D-11 | `dur_90060s_shows_1d_1h_1m` | 90060s → `"1d 1h 1m"` | P |
//! | D-12 | `dur_max_u64_does_not_panic` | `u64::MAX` → does not panic | P |

// ── Adapter tests ─────────────────────────────────────────────────────────────

#[ cfg( feature = "enabled" ) ]
mod adapter
{
  use claude_profile::adapter::argv_to_unilang_tokens;

  fn s( vals : &[ &str ] ) -> Vec< String >
  {
    vals.iter().map( std::string::ToString::to_string ).collect()
  }

  // A-01: empty argv → help
  #[ test ]
  fn adapter_empty_argv_returns_help()
  {
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &[] ).unwrap();
    assert_eq!( tokens, vec![ ".help" ] );
    assert!( needs_help );
  }

  // A-02: single command passes through
  #[ test ]
  fn adapter_single_command()
  {
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ ".accounts" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".accounts" ] );
    assert!( !needs_help );
  }

  // A-03: dot → help
  #[ test ]
  fn adapter_dot_returns_help()
  {
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ "." ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".help" ] );
    assert!( needs_help );
  }

  // A-04: --help → unexpected flag error (POSIX flags not supported)
  #[ test ]
  fn adapter_rejects_double_dash_help()
  {
    let err = argv_to_unilang_tokens( &s( &[ "--help" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "unexpected flag" ), "got: {msg}" );
  }

  // A-05: -h → unexpected flag error (POSIX flags not supported)
  #[ test ]
  fn adapter_rejects_dash_h()
  {
    let err = argv_to_unilang_tokens( &s( &[ "-h" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "unexpected flag" ), "got: {msg}" );
  }

  // A-06: -verbose → error
  #[ test ]
  fn adapter_rejects_unknown_flag()
  {
    let err = argv_to_unilang_tokens( &s( &[ "-verbose" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "unexpected flag" ), "got: {msg}" );
  }

  // A-07: key::value as first arg → error
  #[ test ]
  fn adapter_rejects_param_as_command()
  {
    let err = argv_to_unilang_tokens( &s( &[ "key::value" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "expected command name" ), "got: {msg}" );
  }

  // A-08: v::0 → verbosity::0
  #[ test ]
  fn adapter_verbosity_alias_0()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "v::0" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "verbosity::0" ] );
  }

  // A-09: v::1 → verbosity::1
  #[ test ]
  fn adapter_verbosity_alias_1()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "v::1" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "verbosity::1" ] );
  }

  // A-10: v::2 → verbosity::2
  #[ test ]
  fn adapter_verbosity_alias_2()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "v::2" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "verbosity::2" ] );
  }

  // A-11: v::3 → error (out of range)
  #[ test ]
  fn adapter_verbosity_out_of_range()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "v::3" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "out of range" ), "got: {msg}" );
  }

  // A-12: v::abc → error (not numeric)
  #[ test ]
  fn adapter_verbosity_non_numeric()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "v::abc" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "verbosity" ), "got: {msg}" );
  }

  // A-13: verbosity::1 (canonical key)
  #[ test ]
  fn adapter_verbosity_canonical_key()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "verbosity::1" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "verbosity::1" ] );
  }

  // A-14: dry::true → dry::1
  #[ test ]
  fn adapter_dry_true_normalizes()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "dry::true" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "dry::1" ] );
  }

  // A-15: dry::false → dry::0
  #[ test ]
  fn adapter_dry_false_normalizes()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "dry::false" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "dry::0" ] );
  }

  // A-16: dry::1 stays
  #[ test ]
  fn adapter_dry_1_passthrough()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "dry::1" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "dry::1" ] );
  }

  // A-17: dry::0 stays
  #[ test ]
  fn adapter_dry_0_passthrough()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "dry::0" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "dry::0" ] );
  }

  // A-18: dry::maybe → error
  #[ test ]
  fn adapter_dry_invalid_value()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "dry::maybe" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "invalid value for dry" ), "got: {msg}" );
  }

  // A-19: bareword → error (no ::)
  #[ test ]
  fn adapter_missing_separator()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "bareword" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "param::value" ), "got: {msg}" );
  }

  // A-20: --verbose after command → error
  #[ test ]
  fn adapter_rejects_flag_in_params()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "--verbose" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "unexpected flag" ), "got: {msg}" );
  }

  // A-21: duplicate → last wins
  #[ test ]
  fn adapter_duplicate_last_wins()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "k::v1", "k::v2" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "k::v2" ] );
  }

  // A-22: alias then canonical → last wins
  #[ test ]
  fn adapter_alias_then_canonical_dedup()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "v::0", "verbosity::2" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "verbosity::2" ] );
  }

  // A-23: key::val::extra → split at first ::
  #[ test ]
  fn adapter_multi_separator_split()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "key::val::extra" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "key::val::extra" ] );
  }

  // A-24: key:: → empty value allowed
  #[ test ]
  fn adapter_empty_value_allowed()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "key::" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "key::" ] );
  }

  // A-25: v::-1 → error (u8 parse)
  #[ test ]
  fn adapter_verbosity_negative()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "v::-1" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "verbosity" ), "got: {msg}" );
  }

  // A-26: v::2.5 → error
  #[ test ]
  fn adapter_verbosity_decimal()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "v::2.5" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "verbosity" ), "got: {msg}" );
  }

  // A-27: v:: → error (empty)
  #[ test ]
  fn adapter_verbosity_empty()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "v::" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "verbosity" ), "got: {msg}" );
  }

  // A-28: multiple params assembled
  #[ test ]
  fn adapter_multiple_params()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".cmd", "v::1", "format::json", "dry::1" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".cmd", "verbosity::1", "format::json", "dry::1" ] );
  }

  // A-29: `.help` as sole arg → tokens=[".help"], needs_help=true
  //
  // Step 1b pre-scan matches `.help` anywhere in argv (including as the
  // sole argument) and returns needs_help=true. Callers discard this flag
  // (`_needs_help`), so the CLI output is identical regardless of its value.
  #[ test ]
  fn adapter_help_as_command()
  {
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ ".help" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".help" ] );
    assert!( needs_help );
  }

  // A-30: v::256 → error (u8 overflow)
  #[ test ]
  fn adapter_verbosity_overflow()
  {
    let err = argv_to_unilang_tokens( &s( &[ ".cmd", "v::256" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "verbosity" ), "got: {msg}" );
  }

  // A-31: `.help` in second position → routes to `.help` (N→P)
  //
  // ## Root Cause
  //
  // `claude_profile`'s adapter only checked `--help`/`-h` as argv[0].  It had
  // no pre-scan for `.help` anywhere in argv, so `clp .accounts .help`
  // was parsed as a bare token after the command name and rejected with
  // "expected param::value syntax, got: '.help'" instead of showing help.
  //
  // ## Why Not Caught
  //
  // `claude_version`'s adapter gained a `.help`-anywhere scan (Step 1b) for a
  // prior bug fix but that fix was never propagated to `claude_profile`.  No
  // test exercised `.help` in a non-first position against `clp`.
  //
  // ## Fix Applied
  //
  // Added Step 1b to `claude_profile::adapter::argv_to_unilang_tokens`: a
  // pre-scan over all argv for `".help"` or `"help"`, routing either form to
  // `".help"` before any other processing.
  //
  // ## Prevention
  //
  // A-31 and A-32 lock the new behaviour.  Any future regression (e.g. removing
  // Step 1b while refactoring the adapter) will be caught immediately.
  //
  // ## Pitfall
  //
  // Adding `--help`/`-h` handling does NOT substitute for the pre-scan.
  // `argv[0]` checks are too narrow: users type `clp .accounts .help` or
  // `clp .accounts help` — the help token is not in position 0.
  // test_kind: bug_reproducer(issue-help-prescan)
  #[ test ]
  fn adapter_dot_help_in_second_position()
  {
    // A-31: `.help` after command name must route to `.help`, not error.
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ ".accounts", ".help" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".help" ] );
    assert!( needs_help );
  }

  // A-32: bare `help` in second position → routes to `.help` (N→P)
  //
  // Same root cause as A-31; bare `help` (no dot) is the form advertised in
  // the unilang help footer ("Use '<command> help'…").
  #[ test ]
  fn adapter_bare_help_in_second_position()
  {
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ ".accounts", "help" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".help" ] );
    assert!( needs_help );
  }

  // A-33: bare `help` as sole argv → routes to `.help`
  #[ test ]
  fn adapter_bare_help_as_sole_arg()
  {
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ "help" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".help" ] );
    assert!( needs_help );
  }

  // A-34: --version → unexpected flag error (POSIX flags not supported)
  #[ test ]
  fn adapter_rejects_double_dash_version()
  {
    let err = argv_to_unilang_tokens( &s( &[ "--version" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "unexpected flag" ), "got: {msg}" );
  }

  // A-35: -V → unexpected flag error (POSIX flags not supported)
  #[ test ]
  fn adapter_rejects_dash_v_version()
  {
    let err = argv_to_unilang_tokens( &s( &[ "-V" ] ) ).unwrap_err();
    let msg = format!( "{err}" );
    assert!( msg.contains( "unexpected flag" ), "got: {msg}" );
  }

  // A-36: account::true → account::1 (field-presence bool normalisation)
  #[ test ]
  fn adapter_field_presence_bool_true_normalises()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".credentials.status", "account::true" ] ) ).unwrap();
    assert!(
      tokens.contains( &"account::1".to_string() ),
      "account::true must normalise to account::1, got: {tokens:?}",
    );
  }

  // A-37: account::false → account::0 (field-presence bool normalisation)
  #[ test ]
  fn adapter_field_presence_bool_false_normalises()
  {
    let ( tokens, _ ) = argv_to_unilang_tokens( &s( &[ ".credentials.status", "account::false" ] ) ).unwrap();
    assert!(
      tokens.contains( &"account::0".to_string() ),
      "account::false must normalise to account::0, got: {tokens:?}",
    );
  }
}

// ── Output tests ──────────────────────────────────────────────────────────────

#[ cfg( feature = "enabled" ) ]
mod output
{
  use claude_profile::output::{ OutputFormat, OutputOptions, json_escape };
  use std::collections::HashMap;
  use unilang::data::CommandDefinition;
  use unilang::semantic::VerifiedCommand;
  use unilang::types::Value;

  fn make_cmd( args : Vec< ( &str, Value ) > ) -> VerifiedCommand
  {
    let mut arguments = HashMap::new();
    for ( k, v ) in args
    {
      arguments.insert( k.to_string(), v );
    }
    let definition = CommandDefinition::former()
      .name( ".test" )
      .description( "test command" )
      .end();
    VerifiedCommand { definition, arguments }
  }

  // O-01: format="text" → Text
  #[ test ]
  fn output_format_text()
  {
    let cmd = make_cmd( vec![ ( "format", Value::String( "text".to_string() ) ) ] );
    let opts = OutputOptions::from_cmd( &cmd ).unwrap();
    assert_eq!( opts.format, OutputFormat::Text );
  }

  // O-02: format="json" → Json
  #[ test ]
  fn output_format_json()
  {
    let cmd = make_cmd( vec![ ( "format", Value::String( "json".to_string() ) ) ] );
    let opts = OutputOptions::from_cmd( &cmd ).unwrap();
    assert_eq!( opts.format, OutputFormat::Json );
  }

  // O-03: format="xml" → error
  #[ test ]
  fn output_format_xml_rejected()
  {
    let cmd = make_cmd( vec![ ( "format", Value::String( "xml".to_string() ) ) ] );
    let err = OutputOptions::from_cmd( &cmd ).unwrap_err();
    assert!( err.message.contains( "unknown format" ), "got: {}", err.message );
  }

  // O-04: no format → Text (default)
  #[ test ]
  fn output_format_default()
  {
    let cmd = make_cmd( vec![] );
    let opts = OutputOptions::from_cmd( &cmd ).unwrap();
    assert_eq!( opts.format, OutputFormat::Text );
  }

  // O-05: format="JSON" (uppercase) → error
  #[ test ]
  fn output_format_case_json_rejected()
  {
    let cmd = make_cmd( vec![ ( "format", Value::String( "JSON".to_string() ) ) ] );
    let err = OutputOptions::from_cmd( &cmd ).unwrap_err();
    assert!( err.message.contains( "unknown format" ), "got: {}", err.message );
  }

  // O-06: format="Text" (capitalized) → error
  #[ test ]
  fn output_format_case_text_rejected()
  {
    let cmd = make_cmd( vec![ ( "format", Value::String( "Text".to_string() ) ) ] );
    let err = OutputOptions::from_cmd( &cmd ).unwrap_err();
    assert!( err.message.contains( "unknown format" ), "got: {}", err.message );
  }

  // O-07: verbosity=0
  #[ test ]
  fn output_verbosity_0()
  {
    let cmd = make_cmd( vec![ ( "verbosity", Value::Integer( 0 ) ) ] );
    let opts = OutputOptions::from_cmd( &cmd ).unwrap();
    assert_eq!( opts.verbosity, 0 );
  }

  // O-08: verbosity=1
  #[ test ]
  fn output_verbosity_1()
  {
    let cmd = make_cmd( vec![ ( "verbosity", Value::Integer( 1 ) ) ] );
    let opts = OutputOptions::from_cmd( &cmd ).unwrap();
    assert_eq!( opts.verbosity, 1 );
  }

  // O-09: verbosity=2
  #[ test ]
  fn output_verbosity_2()
  {
    let cmd = make_cmd( vec![ ( "verbosity", Value::Integer( 2 ) ) ] );
    let opts = OutputOptions::from_cmd( &cmd ).unwrap();
    assert_eq!( opts.verbosity, 2 );
  }

  // O-10: no verbosity → default 1
  #[ test ]
  fn output_verbosity_default()
  {
    let cmd = make_cmd( vec![] );
    let opts = OutputOptions::from_cmd( &cmd ).unwrap();
    assert_eq!( opts.verbosity, 1 );
  }

  // O-11: json_escape plain
  #[ test ]
  fn json_escape_plain()
  {
    assert_eq!( json_escape( "hello" ), "hello" );
  }

  // O-12: json_escape quote
  #[ test ]
  fn json_escape_quote()
  {
    assert_eq!( json_escape( r#"he"lo"# ), r#"he\"lo"# );
  }

  // O-13: json_escape backslash
  #[ test ]
  fn json_escape_backslash()
  {
    assert_eq!( json_escape( "he\\lo" ), "he\\\\lo" );
  }

  // O-14: json_escape newline
  #[ test ]
  fn json_escape_newline()
  {
    assert_eq!( json_escape( "a\nb" ), "a\\nb" );
  }

  // O-15: json_escape tab
  #[ test ]
  fn json_escape_tab()
  {
    assert_eq!( json_escape( "a\tb" ), "a\\tb" );
  }

  // O-16: json_escape CR
  #[ test ]
  fn json_escape_cr()
  {
    assert_eq!( json_escape( "a\rb" ), "a\\rb" );
  }

  // O-17: json_escape empty
  #[ test ]
  fn json_escape_empty()
  {
    assert_eq!( json_escape( "" ), "" );
  }

  // O-18: json_escape mixed
  #[ test ]
  fn json_escape_mixed()
  {
    assert_eq!( json_escape( "a\"b\\c\nd" ), "a\\\"b\\\\c\\nd" );
  }

  // O-19: json_escape unicode passthrough
  #[ test ]
  fn json_escape_unicode()
  {
    assert_eq!( json_escape( "cafe\u{0301}" ), "cafe\u{0301}" );
  }
}

// ── format_duration_secs tests ────────────────────────────────────────────────

/// Unit tests for `format_duration_secs`: human-readable duration from seconds.
///
/// Corner cases covered:
/// - Zero and sub-minute values (no days/hours/mins components)
/// - Boundary at exactly 60s (first minute shows)
/// - Boundary at exactly 3600s (first hour, no minutes shown)
/// - Mixed components: hours+minutes, days+hours, days+minutes
/// - `u64::MAX` does not panic (overflow safety)
#[ cfg( feature = "enabled" ) ]
mod format_duration
{
  use claude_profile::output::format_duration_secs;

  // D-01: 0 seconds → "0m"
  #[ test ]
  fn dur_zero_seconds_shows_0m()
  {
    assert_eq!( format_duration_secs( 0 ), "0m" );
  }

  // D-02: 1 second (sub-minute) → "0m"
  #[ test ]
  fn dur_sub_minute_rounds_to_0m()
  {
    assert_eq!( format_duration_secs( 1 ), "0m" );
  }

  // D-03: 59 seconds → "0m"
  #[ test ]
  fn dur_59s_rounds_to_0m()
  {
    assert_eq!( format_duration_secs( 59 ), "0m" );
  }

  // D-04: exactly 60 seconds → "1m"
  #[ test ]
  fn dur_60s_shows_1m()
  {
    assert_eq!( format_duration_secs( 60 ), "1m" );
  }

  // D-05: 3599 seconds → "59m"
  #[ test ]
  fn dur_3599s_shows_59m()
  {
    assert_eq!( format_duration_secs( 3599 ), "59m" );
  }

  // D-06: exactly 3600 seconds (1 hour, 0 minutes) → "1h"
  // Pitfall: the condition `mins > 0 || parts.is_empty()` means minutes are
  // suppressed when hours or days already appear AND mins == 0.
  #[ test ]
  fn dur_3600s_shows_1h_no_minutes()
  {
    assert_eq!( format_duration_secs( 3600 ), "1h" );
  }

  // D-07: 3660 seconds (1h 1m) → "1h 1m"
  #[ test ]
  fn dur_3660s_shows_1h_1m()
  {
    assert_eq!( format_duration_secs( 3660 ), "1h 1m" );
  }

  // D-08: exactly 86400 seconds (1 day, 0 hours, 0 mins) → "1d"
  #[ test ]
  fn dur_86400s_shows_1d_no_hours()
  {
    assert_eq!( format_duration_secs( 86400 ), "1d" );
  }

  // D-09: 86460 seconds (1d 1m, no hours) → "1d 1m"
  #[ test ]
  fn dur_86460s_shows_1d_1m()
  {
    assert_eq!( format_duration_secs( 86460 ), "1d 1m" );
  }

  // D-10: 90000 seconds (1d 1h, 0 mins) → "1d 1h"
  #[ test ]
  fn dur_90000s_shows_1d_1h_no_minutes()
  {
    assert_eq!( format_duration_secs( 90000 ), "1d 1h" );
  }

  // D-11: 90060 seconds (1d 1h 1m) → "1d 1h 1m"
  #[ test ]
  fn dur_90060s_shows_1d_1h_1m()
  {
    assert_eq!( format_duration_secs( 90060 ), "1d 1h 1m" );
  }

  // D-12: u64::MAX — must not panic (overflow safety)
  #[ test ]
  fn dur_max_u64_does_not_panic()
  {
    let _ = format_duration_secs( u64::MAX );
  }
}
