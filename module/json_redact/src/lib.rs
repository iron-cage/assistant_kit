//! Domain-agnostic redaction of sensitive values from strings and JSON.
//!
//! Two independent redaction layers, both always active:
//!
//! - **Key matching** — any key whose lowercased name *contains* a deny-list atom
//!   (e.g. `accessToken`, `refresh_token`, `sessionKey` all contain atoms) has its
//!   value replaced, in JSON documents and in `key=value`/`key::value` free-text pairs.
//! - **Value patterns** — secret-shaped values (`sk-ant-…` tokens, `eyJ…` JWTs, the
//!   token after a `Bearer` marker) are scrubbed wherever they appear: bare positional
//!   arguments, mid-string, or under keys the deny-list does not recognize.
//!
//! The crate deliberately errs toward over-redaction: a redactor that occasionally
//! scrubs a benign value (`monkey=1` matches the `key` atom) is acceptable; one that
//! leaks a credential is not. The recursion depth guard follows the same doctrine and
//! fails closed — subtrees beyond the depth bound are replaced, never passed through.

use serde_json::Value;

/// Text substituted for any value matched as sensitive.
pub const REDACTED : &str = "***REDACTED***";

/// Recursion depth guard for `redact_json` — bounds stack usage against adversarial input.
const MAX_DEPTH : usize = 64;

/// Minimum length of a token run for `Bearer`-context and bare-token redaction,
/// filtering out ordinary short words while catching real credentials.
const MIN_SECRET_LEN : usize = 8;

/// Case-insensitive set of key-name atoms treated as sensitive.
///
/// A key matches when its lowercased form *contains* any atom — substring matching,
/// not equality — so `accessToken`, `api_key`, and `X-Auth-Header` all match without
/// enumerating every casing and prefix variant. Over-redaction bias is deliberate.
#[ derive( Debug ) ]
pub struct RedactionPolicy
{
  keys : Vec< String >,
}

impl RedactionPolicy
{
  /// Returns a new policy with an additional sensitive key-name atom (case-insensitive,
  /// matched as a substring of the key).
  #[ inline ]
  #[ must_use ]
  pub fn with_key( mut self, key : impl Into< String > ) -> Self
  {
    self.keys.push( key.into().to_lowercase() );
    self
  }

  fn is_sensitive( &self, key : &str ) -> bool
  {
    let lowered = key.to_lowercase();
    self.keys.iter().any( | atom | lowered.contains( atom.as_str() ) )
  }
}

impl Default for RedactionPolicy
{
  /// Built-in deny-list atoms covering common credential key shapes: `token`, `password`,
  /// `passwd`, `pwd`, `secret`, `auth`, `bearer`, `key`, `credential`. Substring matching
  /// makes these cover `accessToken`, `refreshToken`, `api_key`, `authorization`,
  /// `client_secret`, `oauth_token`, `private_key`, `sessionKey`, and the like.
  #[ inline ]
  fn default() -> Self
  {
    Self
    {
      keys : [ "token", "password", "passwd", "pwd", "secret", "auth", "bearer", "key", "credential" ]
        .into_iter()
        .map( String::from )
        .collect(),
    }
  }
}

/// Recursively redacts sensitive content in `value`, at any nesting depth.
///
/// Object values under a key matching `policy` are replaced with [`REDACTED`]; every
/// other string — keys included — is scrubbed for secret-shaped substrings
/// (`sk-ant-…`, JWTs, `Bearer` tokens). Recursion is bounded by an internal depth
/// guard that fails closed: subtrees at the bound are replaced with [`REDACTED`]
/// rather than returned unredacted.
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
    // Fail closed: a redactor that stops redacting on adversarially deep input would
    // turn the stack-safety bound into a redaction bypass.
    return Value::String( REDACTED.to_string() );
  }

  match value
  {
    Value::Object( map ) =>
    {
      let mut out = serde_json::Map::new();
      for ( k, v ) in map
      {
        // Keys are scrubbed too — a secret used as a key is still a leak.
        let key_out = scrub_secrets( k );
        if policy.is_sensitive( k )
        {
          out.insert( key_out, Value::String( REDACTED.to_string() ) );
        }
        else
        {
          out.insert( key_out, redact_json_at_depth( v, policy, depth + 1 ) );
        }
      }
      Value::Object( out )
    }
    Value::Array( items ) =>
    {
      Value::Array( items.iter().map( | v | redact_json_at_depth( v, policy, depth + 1 ) ).collect() )
    }
    Value::String( s ) => Value::String( scrub_secrets( s ) ),
    other => other.clone(),
  }
}

/// Redacts sensitive content in free text such as CLI invocation strings.
///
/// Inspects each whitespace-delimited token (whitespace runs — spaces, tabs,
/// newlines — are preserved verbatim): `key=value`/`key::value` pairs whose key
/// matches `policy` have the value replaced (a leading `--` is preserved); when a
/// redacted value opens a quote it does not close, the quoted continuation across
/// following tokens is swallowed into the same replacement. The token after a
/// standalone `Bearer` word, and secret-shaped substrings (`sk-ant-…`, JWTs)
/// anywhere in any token, are scrubbed regardless of key names.
#[ inline ]
#[ must_use ]
pub fn redact_str( input : &str, policy : &RedactionPolicy ) -> String
{
  let mut out = String::with_capacity( input.len() );
  let mut swallow_quote : Option< char > = None;
  let mut prev_word_bearer = false;
  let mut rest = input;

  while !rest.is_empty()
  {
    let ws_end = rest.find( | c : char | !c.is_whitespace() ).unwrap_or( rest.len() );
    let ( ws, tail ) = rest.split_at( ws_end );
    if swallow_quote.is_none()
    {
      out.push_str( ws );
    }
    if tail.is_empty()
    {
      break;
    }
    let word_end = tail.find( char::is_whitespace ).unwrap_or( tail.len() );
    let ( word, next ) = tail.split_at( word_end );
    rest = next;

    if let Some( q ) = swallow_quote
    {
      // Part of a quoted value already replaced with REDACTED — drop it entirely.
      if word.contains( q )
      {
        swallow_quote = None;
      }
      continue;
    }

    if prev_word_bearer && word.len() >= MIN_SECRET_LEN && word.chars().all( is_token_char )
    {
      out.push_str( REDACTED );
      prev_word_bearer = false;
      continue;
    }
    prev_word_bearer = word.eq_ignore_ascii_case( "bearer" );

    let ( emitted, quote ) = redact_word( word, policy );
    out.push_str( &emitted );
    swallow_quote = quote;
  }

  out
}

/// Redacts one whitespace-delimited token; returns the emitted text plus the quote
/// character to swallow when a redacted value opened a quote it did not close.
fn redact_word( word : &str, policy : &RedactionPolicy ) -> ( String, Option< char > )
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
    if let Some( ( key, value ) ) = rest.split_once( sep )
    {
      if policy.is_sensitive( key )
      {
        return ( format!( "{prefix}{key}{sep}{REDACTED}" ), unclosed_quote( value ) );
      }
      // Key is clean — the value may still be secret-shaped.
      return ( scrub_secrets( word ), None );
    }
  }

  ( scrub_secrets( word ), None )
}

/// Returns the quote character when `value` opens a `"`/`'` quote and never closes it —
/// the signal that the quoted value continues into following whitespace tokens.
fn unclosed_quote( value : &str ) -> Option< char >
{
  let mut chars = value.chars();
  let first = chars.next()?;
  if ( first == '"' || first == '\'' ) && !chars.as_str().contains( first )
  {
    return Some( first );
  }
  None
}

fn is_token_char( c : char ) -> bool
{
  c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
}

/// Replaces secret-shaped substrings anywhere in `s` with [`REDACTED`]:
/// `sk-ant-…` token runs, `eyJ…`-prefixed JWTs, and the token following a
/// standalone `Bearer` marker. Surrounding text is preserved.
fn scrub_secrets( s : &str ) -> String
{
  let bytes = s.as_bytes();
  let mut out = String::with_capacity( s.len() );
  let mut last = 0;
  let mut i = 0;

  while i < bytes.len()
  {
    // All patterns start at ASCII bytes, so a match position is always a char boundary.
    let replaced = match_sk_ant( s, i )
      .or_else( || match_jwt( s, i ) )
      .map( | end | ( i, end ) )
      .or_else( || match_bearer( s, i ) );

    if let Some( ( span_start, span_end ) ) = replaced
    {
      out.push_str( &s[ last..span_start ] );
      out.push_str( REDACTED );
      i = span_end;
      last = span_end;
    }
    else
    {
      i += 1;
    }
  }

  out.push_str( &s[ last.. ] );
  out
}

/// True when the byte before `i` does not extend a token run — the pattern start is
/// a genuine token boundary, not the middle of a longer identifier.
fn at_token_boundary( bytes : &[ u8 ], i : usize ) -> bool
{
  i == 0 || !is_token_char( bytes[ i - 1 ] as char )
}

/// Matches an `sk-ant-` credential run starting at byte `i`; returns the end offset.
fn match_sk_ant( s : &str, i : usize ) -> Option< usize >
{
  let bytes = s.as_bytes();
  // Byte-level prefix check: `i` may sit mid-UTF-8-char during the scan, where a
  // `&s[ i.. ]` slice would panic; byte comparison is boundary-safe.
  if !bytes[ i.. ].starts_with( b"sk-ant-" ) || !at_token_boundary( bytes, i )
  {
    return None;
  }
  let body_start = i + "sk-ant-".len();
  let mut end = body_start;
  while end < bytes.len() && ( bytes[ end ].is_ascii_alphanumeric() || bytes[ end ] == b'_' || bytes[ end ] == b'-' )
  {
    end += 1;
  }
  ( end - body_start >= 4 ).then_some( end )
}

/// Matches an `eyJ…` JWT-shaped run (≥ 2 dots, ≥ 20 chars) starting at byte `i`;
/// returns the end offset.
fn match_jwt( s : &str, i : usize ) -> Option< usize >
{
  let bytes = s.as_bytes();
  // Byte-level prefix check — see `match_sk_ant` for the boundary-safety rationale.
  if !bytes[ i.. ].starts_with( b"eyJ" ) || !at_token_boundary( bytes, i )
  {
    return None;
  }
  let mut end = i;
  while end < bytes.len() && is_token_char( bytes[ end ] as char )
  {
    end += 1;
  }
  let run = &s[ i..end ];
  ( run.len() >= 20 && run.bytes().filter( | b | *b == b'.' ).count() >= 2 ).then_some( end )
}

/// Matches a case-insensitive standalone `bearer` word followed by whitespace and a
/// token run of at least [`MIN_SECRET_LEN`] chars; returns the token run's span.
fn match_bearer( s : &str, i : usize ) -> Option< ( usize, usize ) >
{
  let bytes = s.as_bytes();
  // Byte-level prefix check — see `match_sk_ant` for the boundary-safety rationale.
  if bytes.len() < i + 6 || !bytes[ i..i + 6 ].eq_ignore_ascii_case( b"bearer" ) || !at_token_boundary( bytes, i )
  {
    return None;
  }
  let mut tok_start = i + 6;
  if tok_start >= bytes.len() || !bytes[ tok_start ].is_ascii_whitespace()
  {
    return None;
  }
  while tok_start < bytes.len() && bytes[ tok_start ].is_ascii_whitespace()
  {
    tok_start += 1;
  }
  let mut end = tok_start;
  while end < bytes.len() && is_token_char( bytes[ end ] as char )
  {
    end += 1;
  }
  ( end - tok_start >= MIN_SECRET_LEN ).then_some( ( tok_start, end ) )
}
