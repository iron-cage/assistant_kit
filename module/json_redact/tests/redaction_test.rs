//! Integration tests for `json_redact`'s public API — Test Matrix T01-T20.

use json_redact::RedactionPolicy;
use serde_json::json;

#[ test ]
fn t01_key_name_redaction_top_level()
{
  let policy = RedactionPolicy::default();
  let input = json!( { "token": "abc123" } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result, json!( { "token": "***REDACTED***" } ) );
}

#[ test ]
fn t02_key_name_redaction_nested_object()
{
  let policy = RedactionPolicy::default();
  let input = json!( { "a": { "password": "x" } } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result, json!( { "a": { "password": "***REDACTED***" } } ) );
}

#[ test ]
fn t03_key_name_redaction_case_insensitive()
{
  let policy = RedactionPolicy::default();
  let input = json!( { "API_KEY": "x" } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result, json!( { "API_KEY": "***REDACTED***" } ) );
}

#[ test ]
fn t04_non_sensitive_keys_untouched()
{
  let policy = RedactionPolicy::default();
  let input = json!( { "name": "foo" } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result, input );
}

#[ test ]
fn t05_array_elements_redacted()
{
  let policy = RedactionPolicy::default();
  let input = json!( { "items": [ { "secret": "x" } ] } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result, json!( { "items": [ { "secret": "***REDACTED***" } ] } ) );
}

#[ test ]
fn t06_string_pattern_redaction()
{
  let policy = RedactionPolicy::default();
  let result = json_redact::redact_str( "--token=abc123 --verbose", &policy );
  assert!( result.contains( "***REDACTED***" ) );
  assert!( !result.contains( "abc123" ) );
  assert!( result.contains( "--verbose" ) );
}

#[ test ]
fn t06b_string_pattern_redaction_double_colon()
{
  // This workspace's CLI convention (`clp`, `yrd`) uses `key::value`, not just `key=value`.
  let policy = RedactionPolicy::default();
  let result = json_redact::redact_str( "token::abc123 verbose::true", &policy );
  assert_eq!( result, "token::***REDACTED*** verbose::true" );
}

#[ test ]
fn t07_custom_policy_extension()
{
  let policy = RedactionPolicy::default().with_key( "custom_field" );
  let input = json!( { "custom_field": "x", "token": "y" } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result, json!( { "custom_field": "***REDACTED***", "token": "***REDACTED***" } ) );
}

#[ test ]
fn t08_empty_input()
{
  let policy = RedactionPolicy::default();
  assert_eq!( json_redact::redact_json( &json!( {} ), &policy ), json!( {} ) );
  assert_eq!( json_redact::redact_str( "", &policy ), "" );
}

#[ test ]
fn t09_deeply_nested_json()
{
  let policy = RedactionPolicy::default();
  let input = json!( { "a": { "b": { "c": { "d": { "secret": "x" } } } } } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result, json!( { "a": { "b": { "c": { "d": { "secret": "***REDACTED***" } } } } } ) );
}

#[ test ]
fn default_deny_list_atoms_cover_credential_key_shapes()
{
  // Measurement M1: every default deny-list atom triggers via substring matching,
  // and keys containing no atom survive untouched.
  let policy = RedactionPolicy::default();
  let input = json!( {
    "token": "1", "password": "2", "secret": "3", "authorization": "4",
    "api_key": "5", "apikey": "6", "key": "7", "credential": "8",
    "passwd": "9", "pwd": "10", "bearer": "11",
    "untouched": "12", "name": "13",
  } );
  let result = json_redact::redact_json( &input, &policy );
  let obj = result.as_object().unwrap();
  let redacted_count = obj.values().filter( | v | *v == "***REDACTED***" ).count();
  assert_eq!( redacted_count, 11 );
  assert_eq!( obj.get( "untouched" ).unwrap(), "12" );
  assert_eq!( obj.get( "name" ).unwrap(), "13" );
}

#[ test ]
fn t10_credential_file_key_aliases_redacted()
{
  // The two most important secrets in this system use camelCase names that exact-key
  // matching missed entirely; substring matching must cover them and common variants.
  let policy = RedactionPolicy::default();
  let input = json!( {
    "data":
    {
      "accessToken": "sk-ant-oat01-aaaa", "refreshToken": "sk-ant-ort01-bbbb",
      "access_token": "x1", "refresh_token": "x2", "client_secret": "x3",
      "oauth_token": "x4", "private_key": "x5", "sessionKey": "x6",
      "expiresAt": 123,
    }
  } );
  let result = json_redact::redact_json( &input, &policy );
  let inner = result[ "data" ].as_object().unwrap();
  for field in [ "accessToken", "refreshToken", "access_token", "refresh_token", "client_secret", "oauth_token", "private_key", "sessionKey" ]
  {
    assert_eq!( inner.get( field ).unwrap(), "***REDACTED***", "field {field} must be redacted" );
  }
  assert_eq!( inner.get( "expiresAt" ).unwrap(), 123 );

  // The real credentials.json wraps everything in "claudeAiOauth" — that key itself
  // matches the `auth` atom, so the whole block is redacted as one unit.
  let wrapped = json!( { "claudeAiOauth": { "accessToken": "sk-ant-oat01-aaaa" } } );
  let wrapped_result = json_redact::redact_json( &wrapped, &policy );
  assert_eq!( wrapped_result[ "claudeAiOauth" ], "***REDACTED***" );
}

#[ test ]
fn t11_substring_key_matching_in_free_text()
{
  let policy = RedactionPolicy::default();
  let result = json_redact::redact_str( "accessToken::sk-x --access-token=y verbose::true", &policy );
  assert_eq!( result, "accessToken::***REDACTED*** --access-token=***REDACTED*** verbose::true" );
}

#[ test ]
fn t12_bare_sk_ant_token_positional_redacted()
{
  // A raw token passed as a positional argument has no key name at all — value-pattern
  // matching is the only layer that can catch it.
  let policy = RedactionPolicy::default();
  let result = json_redact::redact_str( "clp .account.add sk-ant-oat01-AbCdEf_123-xyz now", &policy );
  assert!( !result.contains( "sk-ant" ), "raw token leaked: {result}" );
  assert_eq!( result, "clp .account.add ***REDACTED*** now" );
}

#[ test ]
fn t13_jwt_under_unrecognized_key_redacted()
{
  let policy = RedactionPolicy::default();
  let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
  let input = json!( { "blob": jwt } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result[ "blob" ], "***REDACTED***" );
}

#[ test ]
fn t14_bearer_marker_redacts_following_token()
{
  let policy = RedactionPolicy::default();
  // Mid-string, inside a JSON string value.
  let input = json!( { "header_dump": "Authorization: Bearer abc123def456ghi" } );
  let result = json_redact::redact_json( &input, &policy );
  assert_eq!( result[ "header_dump" ], "Authorization: Bearer ***REDACTED***" );
  // Token-level, in free text.
  let text = json_redact::redact_str( "curl -H Bearer abc123def456ghi", &policy );
  assert_eq!( text, "curl -H Bearer ***REDACTED***" );
  // Short ordinary words after "bearer" survive.
  let prose = json_redact::redact_str( "the bearer of news", &policy );
  assert_eq!( prose, "the bearer of news" );
}

#[ test ]
fn t15_depth_guard_fails_closed()
{
  // A redactor that stops redacting on adversarially deep input would turn the
  // stack-safety bound into a bypass — the subtree must be replaced, not passed through.
  let policy = RedactionPolicy::default();
  let mut value = json!( { "secret": "deep-payload" } );
  for _ in 0..70
  {
    value = json!( { "a": value } );
  }
  let result = json_redact::redact_json( &value, &policy );
  let serialized = serde_json::to_string( &result ).unwrap();
  assert!( !serialized.contains( "deep-payload" ), "deep subtree leaked through the depth guard" );
  assert!( serialized.contains( "***REDACTED***" ) );
}

#[ test ]
fn t16_non_space_whitespace_separated_pairs_redacted()
{
  let policy = RedactionPolicy::default();
  let result = json_redact::redact_str( "a\ttoken=x\npassword=y  end", &policy );
  assert_eq!( result, "a\ttoken=***REDACTED***\npassword=***REDACTED***  end" );
}

#[ test ]
fn t17_quoted_value_with_spaces_fully_swallowed()
{
  let policy = RedactionPolicy::default();
  let result = json_redact::redact_str( "--token=\"a b c\" --verbose", &policy );
  assert_eq!( result, "--token=***REDACTED*** --verbose" );
  // Unclosed quote swallows to the end rather than leaking the tail.
  let unclosed = json_redact::redact_str( "--token=\"a b", &policy );
  assert_eq!( unclosed, "--token=***REDACTED***" );
}

#[ test ]
fn t18_over_redaction_bias_is_deliberate()
{
  // Substring matching over-redacts benign keys containing an atom ("monkey" contains
  // "key"). Documented trade-off: scrubbing a benign value beats leaking a credential.
  let policy = RedactionPolicy::default();
  let result = json_redact::redact_str( "monkey=1 name=ok", &policy );
  assert_eq!( result, "monkey=***REDACTED*** name=ok" );
}

#[ test ]
fn t19_secret_used_as_json_key_scrubbed()
{
  let policy = RedactionPolicy::default();
  let input = json!( { "sk-ant-oat01-leaked-as-key": true } );
  let result = json_redact::redact_json( &input, &policy );
  let serialized = serde_json::to_string( &result ).unwrap();
  assert!( !serialized.contains( "sk-ant" ), "secret leaked via key position: {serialized}" );
}

#[ test ]
fn t20_multibyte_text_survives_scrub_scan()
{
  // The value scanner walks bytes; multi-byte characters around and adjacent to
  // patterns must neither panic nor corrupt output.
  let policy = RedactionPolicy::default();
  let result = json_redact::redact_str( "héllo → sk-ant-oat01-abcd €nd", &policy );
  assert_eq!( result, "héllo → ***REDACTED*** €nd" );
  let untouched = json_redact::redact_str( "héllo → wörld €", &policy );
  assert_eq!( untouched, "héllo → wörld €" );
}
