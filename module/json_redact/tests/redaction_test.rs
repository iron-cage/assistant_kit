//! Integration tests for `json_redact`'s public API — Test Matrix T01-T09.

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
fn default_deny_list_has_eight_keys()
{
  // Measurement M1: default deny-list size = 8.
  let policy = RedactionPolicy::default();
  let input = json!( {
    "token": "1", "password": "2", "secret": "3", "authorization": "4",
    "api_key": "5", "apikey": "6", "key": "7", "credential": "8",
    "untouched": "9",
  } );
  let result = json_redact::redact_json( &input, &policy );
  let obj = result.as_object().unwrap();
  let redacted_count = obj.values().filter( | v | *v == "***REDACTED***" ).count();
  assert_eq!( redacted_count, 8 );
  assert_eq!( obj.get( "untouched" ).unwrap(), "9" );
}
