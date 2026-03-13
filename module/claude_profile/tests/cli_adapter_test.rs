//! Unit tests for the adapter and output modules.
//!
//! Covers matrix IDs A-01..A-33 (adapter) and O-01..O-19 (output).
//! All tests call library functions directly — no binary invocation.

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
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ ".account.list" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".account.list" ] );
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

  // A-04: --help → help
  #[ test ]
  fn adapter_double_dash_help()
  {
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ "--help" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".help" ] );
    assert!( needs_help );
  }

  // A-05: -h → help
  #[ test ]
  fn adapter_dash_h_help()
  {
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ "-h" ] ) ).unwrap();
    assert_eq!( tokens, vec![ ".help" ] );
    assert!( needs_help );
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
  // no pre-scan for `.help` anywhere in argv, so `clp .account.list .help`
  // was parsed as a bare token after the command name and rejected with
  // "expected param::value syntax, got: '.help'" instead of showing help.
  //
  // ## Why Not Caught
  //
  // `claude_manager`'s adapter gained a `.help`-anywhere scan (Step 1b) for a
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
  // `argv[0]` checks are too narrow: users type `clp .account.list .help` or
  // `clp .account.list help` — the help token is not in position 0.
  #[ test ]
  fn adapter_dot_help_in_second_position()
  {
    // A-31: `.help` after command name must route to `.help`, not error.
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ ".account.list", ".help" ] ) ).unwrap();
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
    let ( tokens, needs_help ) = argv_to_unilang_tokens( &s( &[ ".account.list", "help" ] ) ).unwrap();
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
