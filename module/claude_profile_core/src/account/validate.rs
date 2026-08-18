//! Account-name validation and credential-filename mapping.

use std::path::Path;
use super::types::AccountBackend;

/// Extract the account name from a `{name}.credentials.json` path.
///
/// Returns `None` for anything that is not a `*.credentials.json` file
/// (e.g. the `_active` marker or unrelated files).
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn credential_stem( path : &Path ) -> Option< String >
{
  let filename = path.file_name()?.to_str()?;
  filename
    .strip_suffix( ".credentials.json" )
    .map( std::string::ToString::to_string )
}

#[ doc( hidden ) ]
#[ inline ]
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
pub fn validate_name( name : &str ) -> Result< (), std::io::Error >
{
  // Account names must be valid email addresses (local@domain) so they can be
  // used as filenames and unambiguously identify the Claude account owner.
  let at = name.find( '@' ).ok_or_else( || std::io::Error::new(
    std::io::ErrorKind::InvalidInput,
    format!( "account name '{name}' is not a valid email address: must contain '@'" ),
  ) )?;
  if at == 0
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' is not a valid email address: local part must not be empty" ),
    ) );
  }
  if name[ at + 1.. ].is_empty()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' is not a valid email address: domain must not be empty" ),
    ) );
  }
  // Fix(issue-123): validate_name() passed names like `a/b@c.com` because it only checked
  //   @-presence and non-empty local/domain parts; the local part was never inspected for
  //   path-unsafe chars, so save()/switch_account() hit filesystem errors (exit 2) instead
  //   of returning InvalidInput (exit 1).
  // Root cause: local-part safety check was absent; chars `/`, `\`, `*` create path
  //   traversal when used as a filename prefix (e.g. `{store}/a/b@c.com.credentials.json`).
  // Pitfall: only the local part (before `@`) needs this check; the domain part appears
  //   after `@` in the filename and cannot create sub-directory traversal in practice.
  let local = &name[ ..at ];
  if local.contains( '/' ) || local.contains( '\\' ) || local.contains( '*' )
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' contains path-unsafe characters in the local part" ),
    ) );
  }
  Ok( () )
}

/// Validate a `backend: redirect` account name (Feature 071): filename-safety only, no
/// email-shape requirement. `validate_name()`'s `@`-requirement exists to unambiguously match
/// the Claude account owner's OAuth identity — a redirect account has no such identity (it is
/// an arbitrary local label for a foreign API key), so that requirement does not apply here;
/// only the underlying filesystem-safety constraint (used as a `{name}.json`/
/// `{name}.credentials.json` filename prefix) still does.
#[ doc( hidden ) ]
#[ inline ]
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
pub fn validate_redirect_name( name : &str ) -> Result< (), std::io::Error >
{
  if name.is_empty()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "account name must not be empty".to_string(),
    ) );
  }
  if name.contains( '/' ) || name.contains( '\\' ) || name.contains( '*' )
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "account name '{name}' contains path-unsafe characters" ),
    ) );
  }
  Ok( () )
}

/// Select and apply the correct `.account.save` name-validation rule (Feature 071, AC-15): a
/// brand-new account name must satisfy its requested backend's shape rule
/// (`validate_redirect_name()` for `redirect`, the stricter `validate_name()` for `anthropic`) —
/// but once the account already has a saved credentials file in `credential_store`, the name is
/// an already-established local identifier and is not re-validated against a newly requested
/// backend's stricter rule ("the account name itself is not permanently locked to one backend");
/// only the permissive `validate_redirect_name()` check applies to a re-save regardless of the
/// new backend. Existence is checked via `{name}.credentials.json`, not `{name}.json` — the
/// caller (`account_save_routine()`'s AC-15 rewrite-from-scratch step) may have already deleted
/// the stale `{name}.json` before calling `save()` when the backend is changing, but always
/// leaves `{name}.credentials.json` in place until `save()` itself overwrites it.
#[ doc( hidden ) ]
#[ inline ]
pub fn validate_name_for_save( name : &str, backend : AccountBackend, credential_store : &Path ) -> Result< (), std::io::Error >
{
  let already_exists = credential_store.join( format!( "{name}.credentials.json" ) ).exists();
  match backend
  {
    _ if already_exists       => validate_redirect_name( name ),
    AccountBackend::Redirect  => validate_redirect_name( name ),
    AccountBackend::Anthropic => validate_name( name ),
  }
}
