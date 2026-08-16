//! Domain-agnostic redaction of sensitive values from strings and JSON.
//!
//! Scrubs values whose key name matches a configurable deny-list (case-insensitive)
//! out of JSON documents, and scrubs `key=value`/`key::value` pairs out of free text
//! such as CLI invocation strings.

use serde_json::Value;

/// Text substituted for any value matched as sensitive.
pub const REDACTED : &str = "***REDACTED***";

/// Recursion depth guard for `redact_json` — bounds stack usage against adversarial input.
const MAX_DEPTH : usize = 64;

/// Case-insensitive set of key names treated as sensitive.
#[ derive( Debug ) ]
pub struct RedactionPolicy
{
  keys : Vec< String >,
}

impl RedactionPolicy
{
  /// Returns a new policy with an additional sensitive key name (case-insensitive).
  #[ inline ]
  #[ must_use ]
  pub fn with_key( mut self, key : impl Into< String > ) -> Self
  {
    self.keys.push( key.into().to_lowercase() );
    self
  }

  fn is_sensitive( &self, key : &str ) -> bool
  {
    self.keys.contains( &key.to_lowercase() )
  }
}

impl Default for RedactionPolicy
{
  /// Built-in deny-list covering common credential key shapes: `token`, `password`,
  /// `secret`, `authorization`, `api_key`, `apikey`, `key`, `credential`.
  #[ inline ]
  fn default() -> Self
  {
    Self
    {
      keys : [ "token", "password", "secret", "authorization", "api_key", "apikey", "key", "credential" ]
        .into_iter()
        .map( String::from )
        .collect(),
    }
  }
}

/// Recursively redacts values in `value` whose key matches `policy`, at any nesting depth.
///
/// Non-`Object`/`Array` values pass through unchanged. Recursion is bounded by an internal
/// depth guard, so adversarially deep input degrades to a no-op below the guard rather than
/// overflowing the stack.
#[ inline ]
#[ must_use ]
pub fn redact_json( value : &Value, policy : &RedactionPolicy ) -> Value
{
  redact_json_at_depth( value, policy, 0 )
}

fn redact_json_at_depth( value : &Value, policy : &RedactionPolicy, depth : usize ) -> Value
{
  if depth >= MAX_DEPTH
  {
    return value.clone();
  }

  match value
  {
    Value::Object( map ) =>
    {
      let mut out = serde_json::Map::new();
      for ( k, v ) in map
      {
        if policy.is_sensitive( k )
        {
          out.insert( k.clone(), Value::String( REDACTED.to_string() ) );
        }
        else
        {
          out.insert( k.clone(), redact_json_at_depth( v, policy, depth + 1 ) );
        }
      }
      Value::Object( out )
    }
    Value::Array( items ) =>
    {
      Value::Array( items.iter().map( | v | redact_json_at_depth( v, policy, depth + 1 ) ).collect() )
    }
    other => other.clone(),
  }
}

/// Redacts `key=value`/`key::value` pairs in free text whose key matches `policy`.
///
/// Splits `input` on ASCII spaces and inspects each whitespace-delimited token independently;
/// a leading `--` (CLI flag style) is preserved. Tokens with no `=`/`::` separator — including
/// bare flags like `--verbose` — pass through unchanged.
#[ inline ]
#[ must_use ]
pub fn redact_str( input : &str, policy : &RedactionPolicy ) -> String
{
  input
    .split( ' ' )
    .map( | word | redact_word( word, policy ) )
    .collect::< Vec< _ > >()
    .join( " " )
}

fn redact_word( word : &str, policy : &RedactionPolicy ) -> String
{
  let ( prefix, rest ) = if let Some( stripped ) = word.strip_prefix( "--" )
  {
    ( "--", stripped )
  }
  else
  {
    ( "", word )
  };

  for sep in [ "::", "=" ]
  {
    if let Some( ( key, _value ) ) = rest.split_once( sep )
    {
      return if policy.is_sensitive( key )
      {
        format!( "{prefix}{key}{sep}{REDACTED}" )
      }
      else
      {
        word.to_string()
      };
    }
  }

  word.to_string()
}
