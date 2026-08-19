//! Credential-store CRUD — listing, the store-wide mutation lock, `save()`, and `delete()`.

use std::path::Path;
use claude_core::ClaudePaths;
use claude_core::file_io::{ atomic_write, atomic_write_secret };
use super::types::{ Account, AccountBackend };
use super::validate::{ credential_stem, validate_name, validate_name_for_save };
use super::ownership::{ active_marker_filename, all_marker_files, current_identity, read_active_marker };
use super::json_field::{ parse_bool_field, parse_string_array_field, parse_string_field, parse_u64_field };
use super::tags::{ TagOp, apply_tag_write, normalize_tag_set };

/// List all accounts in `credential_store`.
///
/// Returns an empty `Vec` if the credential store does not exist yet — not an error.
///
/// An individual account whose credential or metadata file cannot be read is still
/// listed, with the affected fields defaulted (empty strings / zero expiry) — a
/// single corrupt file must not hide the rest of the store from rendering.
///
/// # Errors
///
/// Returns an error only if the credential store directory itself cannot be
/// enumerated (`read_dir` failure); per-file read errors never propagate.
#[ inline ]
#[ must_use = "check the returned accounts list" ]
pub fn list( credential_store : &Path ) -> Result< Vec< Account >, std::io::Error >
{
  if !credential_store.exists() { return Ok( Vec::new() ); }

  let active   = read_active_marker( credential_store );
  // Pre-compute once — current_identity() reads env vars + resolves hostname.
  let identity = current_identity();
  let mut accounts = Vec::new();

  for entry in std::fs::read_dir( credential_store )?.flatten()
  {
    let path = entry.path();
    let Some( name ) = credential_stem( &path ) else { continue };
    let content = std::fs::read_to_string( &path ).unwrap_or_default();
    let subscription_type = parse_string_field( &content, "subscriptionType" )
      .unwrap_or_default();
    let rate_limit_tier = parse_string_field( &content, "rateLimitTier" )
      .unwrap_or_default();
    let expires_at_ms = parse_u64_field( &content, "expiresAt" )
      .unwrap_or( 0 );
    let is_active = active.as_deref() == Some( name.as_str() );

    // Read unified per-account metadata from {name}.json — best-effort, empty when absent.
    let meta_json = std::fs::read_to_string(
      credential_store.join( format!( "{name}.json" ) )
    ).unwrap_or_default();
    let email        = parse_string_field( &meta_json, "emailAddress"      ).unwrap_or_default();
    let display_name = parse_string_field( &meta_json, "displayName"      ).unwrap_or_default();
    let billing      = parse_string_field( &meta_json, "billingType"      ).unwrap_or_default();
    let model        = parse_string_field( &meta_json, "model"            ).unwrap_or_default();
    let tagged_id    = parse_string_field( &meta_json, "taggedId"         ).unwrap_or_default();
    let uuid         = parse_string_field( &meta_json, "uuid"             ).unwrap_or_default();
    let capabilities = parse_string_array_field( &meta_json, "capabilities" );
    let organization_uuid = parse_string_field( &meta_json, "organization_uuid" ).unwrap_or_default();
    let organization_name = parse_string_field( &meta_json, "organization_name" ).unwrap_or_default();
    let org_role          = parse_string_field( &meta_json, "organization_role" ).unwrap_or_default();
    let workspace_uuid    = parse_string_field( &meta_json, "workspace_uuid"    ).unwrap_or_default();
    let workspace_name    = parse_string_field( &meta_json, "workspace_name"    ).unwrap_or_default();
    let host         = parse_string_field( &meta_json, "host"       ).unwrap_or_default();
    let role         = parse_string_field( &meta_json, "role"       ).unwrap_or_default();
    let tags         = parse_string_array_field( &meta_json, "tags" );
    let owner        = parse_string_field( &meta_json, "owner"      ).unwrap_or_default();
    let is_owned     = owner.is_empty() || owner == identity;
    let claim_lock   = parse_bool_field( &meta_json, "claim_lock" ).unwrap_or( false );
    let reserve      = parse_bool_field( &meta_json, "reserve" ).unwrap_or( false );
    let renewal_at   = parse_string_field( &meta_json, "_renewal_at" );
    let backend        = AccountBackend::parse( &parse_string_field( &meta_json, "backend" ).unwrap_or_default() );
    let base_url       = parse_string_field( &meta_json, "base_url" );
    let redirect_model = parse_string_field( &meta_json, "redirect_model" );
    let inference_provider = parse_string_field( &meta_json, "inference_provider" ).unwrap_or_default();

    accounts.push( Account
    {
      name,
      subscription_type,
      rate_limit_tier,
      expires_at_ms,
      is_active,
      email,
      display_name,
      billing,
      model,
      tagged_id,
      uuid,
      capabilities,
      organization_uuid,
      organization_name,
      org_role,
      workspace_uuid,
      workspace_name,
      host,
      role,
      tags,
      owner,
      is_owned,
      claim_lock,
      reserve,
      renewal_at,
      backend,
      base_url,
      redirect_model,
      inference_provider,
    } );
  }

  accounts.sort_by( | a, b | a.name.cmp( &b.name ) );
  Ok( accounts )
}

/// Guard for the exclusive store-wide mutation lock; the lock releases on drop.
///
/// Returned by [`lock_store()`]. Holds the open lock-file descriptor — the kernel
/// releases the `flock` automatically when the descriptor closes (drop, panic
/// unwind, or process death), so a crashed holder can never wedge the store.
#[ derive( Debug ) ]
pub struct StoreLock
{
  _file : std::fs::File,
}

/// Take the exclusive cross-process mutation lock for `credential_store`.
///
/// Serializes the multi-file store mutations — `save()`, `switch_account()`,
/// `delete()` — via a blocking exclusive `flock(2)` on `{store}/-store.lock`
/// (hyphen prefix keeps the lock file out of the store's tracked git tree).
/// Two concurrent switches, or a switch racing a save/delete, could otherwise
/// interleave their credentials/marker/metadata writes and leave the `_active`
/// marker naming one account while another account's credentials are live (each
/// individual write is atomic, but the SEQUENCE was not); under the lock each
/// mutation's whole file sequence lands before the next begins.
///
/// Blocks until the current holder (if any) releases. Advisory only: nothing
/// stops a writer that bypasses this function, and `flock` semantics over NFS
/// mounts are historically unreliable — the store is expected on local disk.
/// The token refresh path deliberately does NOT hold this lock across its
/// multi-second OAuth subprocess window; only its final store writes serialize,
/// via the `save()` call inside — so its marker re-read guard (Fix(BUG-485))
/// stays load-bearing rather than being subsumed by this lock.
///
/// On non-unix targets the lock file is created but no lock is taken.
///
/// # Errors
///
/// Returns an error if the store directory or lock file cannot be created, or
/// the `flock` call itself fails.
// `extern "C"` decl + unsafe call are scoped to this one fn — same idiom as the
// signal-FFI in clp's usage/live.rs execute_live_mode() (workspace denies unsafe
// globally; std's own File::lock would need MSRV 1.89 vs the declared 1.74).
#[ inline ]
#[ allow( unsafe_code ) ]
pub fn lock_store( credential_store : &Path ) -> Result< StoreLock, std::io::Error >
{
  std::fs::create_dir_all( credential_store )?;
  let file = std::fs::OpenOptions::new()
    .create( true )
    .write( true )
    .truncate( false ) // a lock file's content is irrelevant — never disturb a held lock's file
    .open( credential_store.join( "-store.lock" ) )?;
  #[ cfg( unix ) ]
  {
    use std::os::raw::c_int;
    use std::os::unix::io::AsRawFd as _;
    extern "C"
    {
      fn flock( fd : c_int, operation : c_int ) -> c_int;
    }
    const LOCK_EX : c_int = 2;
    // SAFETY: flock takes only an owned open fd and an integer op flag; no pointers cross.
    let rc = unsafe { flock( file.as_raw_fd(), LOCK_EX ) };
    if rc != 0
    {
      return Err( std::io::Error::last_os_error() );
    }
  }
  Ok( StoreLock { _file : file } )
}

/// Save credentials as a named account in `credential_store`.
///
/// Writes two files per account:
/// - `{name}.credentials.json` — OAuth tokens and expiry
/// - `{name}.json` — unified metadata (identity, model, roles, profile)
///
/// When `creds` is `Some(bytes)`, writes `bytes` directly to the credential file.
/// When `creds` is `None`, copies from `paths.credentials_file()` (the live session file).
///
/// `host` / `role` are profile display metadata; pass `None` from background callers
/// to preserve existing values via merge.
///
/// `owner` sets the `owner` field in `{name}.json`:
/// - `Some(s)` — writes `s` as the owner (use `current_identity()` for CLI saves, `""` for unclaim).
/// - `None` — preserves existing `owner` field unchanged (background callers: refresh, touch paths).
///
/// `backend` selects the write path (Feature 071): `Anthropic` (default) captures the live
/// OAuth session exactly as before; `Redirect` skips that capture entirely and instead writes
/// only `accessToken` (from `creds`, the caller-supplied static API key) to `{name}.credentials.json`,
/// plus `backend`/`base_url`/`redirect_model` to `{name}.json`. `base_url`/`redirect_model` are
/// only meaningful when `backend` is `Redirect`.
///
/// `inference_provider` sets the `inference_provider` field in `{name}.json` — the manually
/// selected provider used by Gate 10 rotation eligibility (Feature 072):
/// - `Some(s)` — writes `s` as the selected provider (`.provider.select` only).
/// - `None` — preserves existing `inference_provider` field unchanged (all other callers).
///
/// `tags` sets the `tags` array in `{name}.json` (Feature 075):
/// - `Some(list)` — a tag write with replace semantics: the list is normalized
///   (lowercased, validated, deduplicated, sorted) and overwrites the stored set;
///   the lazy `role` migration applies in the same merge (see `tags::apply_tag_write`).
/// - `None` — preserves the existing `tags` array unchanged (every non-tag caller).
///
/// # Errors
///
/// Returns an error if the name is invalid, a given tag is invalid (rejected
/// before any file write), the credentials file cannot be read, or the
/// credential store cannot be written.
#[ inline ]
#[ allow( clippy::too_many_arguments ) ] // 9th-13th params `backend`/`base_url`/`redirect_model`/`inference_provider`/`tags` added by Features 071/072/075 — all args are independent concerns.
#[ allow( clippy::too_many_lines ) ] // Feature 071's backend-gated branches extend a single coherent capture→merge→write sequence — splitting would fragment closely-threaded local state.
pub fn save(
  name               : &str,
  credential_store   : &Path,
  paths              : &ClaudePaths,
  update_marker      : bool,
  creds              : Option< &[u8] >,
  host               : Option< &str >,
  role               : Option< &str >,
  owner              : Option< &str >,
  backend            : AccountBackend,
  base_url           : Option< &str >,
  redirect_model     : Option< &str >,
  inference_provider : Option< &str >,
  tags               : Option< &[ String ] >,
) -> Result< (), std::io::Error >
{
  // `_lock` must be a named binding — `let _ =` would drop (and release) immediately.
  let _lock = lock_store( credential_store )?;
  validate_name_for_save( name, backend, credential_store )?;
  // Feature 075/AC-02: validate tags BEFORE any file write — a rejected tag
  // must leave the store byte-identical, including the credentials file below.
  let tags = tags.map( normalize_tag_set ).transpose()?;
  std::fs::create_dir_all( credential_store )?;
  let dest = credential_store.join( format!( "{name}.credentials.json" ) );
  // Fix(audit-credential-file-perms): every store credential write goes through
  // atomic_write_secret — 0o600 from the first byte, unique temp name, rename commit.
  // Root cause: bare fs::write/fs::copy landed OAuth tokens with umask-default 0644,
  // readable by any local user, and a shared tmp name let concurrent writers collide.
  // Pitfall: fs::copy also PRESERVES the source file's mode — copying a world-readable
  // live file propagates the exposure into the store; write content, not the file.
  if backend == AccountBackend::Redirect
  {
    // Feature 071: a redirect account has no Anthropic OAuth session to capture —
    // write only the caller-supplied static API key as `accessToken`.
    let key = creds.map( String::from_utf8_lossy ).unwrap_or_default();
    let redirect_creds = serde_json::json!( { "accessToken" : key } );
    atomic_write_secret( &dest, &serde_json::to_string_pretty( &redirect_creds ).map( | s | s + "\n" ).unwrap_or_default() )?;
  }
  else
  {
    // Fix(BUG-221): accept direct credential bytes to bypass the copy-from-live-file path.
    if let Some( bytes ) = creds
    {
      let text = core::str::from_utf8( bytes )
      .map_err( | e | std::io::Error::other( format!( "credential bytes are not UTF-8: {e}" ) ) )?;
      atomic_write_secret( &dest, text )?;
    }
    else
    {
      let live = std::fs::read_to_string( paths.credentials_file() )?;
      atomic_write_secret( &dest, &live )?;
    }
  }

  // Build unified {name}.json — read-merge to preserve pre-existing keys (e.g. _renewal_at).
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let mut snapshot = std::fs::read_to_string( &meta_path )
    .ok()
    .and_then( |s| serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .unwrap_or_else( || serde_json::json!( {} ) );
  if let Some( obj ) = snapshot.as_object_mut()
  {
    // Feature 071/AC-04: every save writes `backend`, even the default anthropic path —
    // makes every account file self-describing instead of relying solely on key-absence.
    obj.insert( "backend".to_string(), serde_json::Value::String( backend.as_str().to_string() ) );
    if backend == AccountBackend::Redirect
    {
      if let Some( u ) = base_url
      {
        obj.insert( "base_url".to_string(), serde_json::Value::String( u.to_string() ) );
      }
      if let Some( m ) = redirect_model
      {
        obj.insert( "redirect_model".to_string(), serde_json::Value::String( m.to_string() ) );
      }
    }
    else
    {
      // Fix(BUG-343): only merge live-session identity when the caller is an explicit,
      // user-driven save (`update_marker == true`) OR the live session's own identity
      // actually IS `name` (`live_is_name == true`).
      // Root cause: the merge branch read this machine's own live ~/.claude.json and
      // merged its oauthAccount into ANY target's file unconditionally, with no check
      // that `name` was the live identity. Background refresh/touch loops over the full
      // account list (see refresh_token_with_live_path()) routinely call save() for
      // accounts that are NOT the active one, so the unconditional merge overwrote a
      // non-active target's own identity with whichever account happens to be locally
      // active on this machine. `update_marker == true` (`.account.save`/
      // `.account.relogin`) is preserved unconditionally because those calls, by design,
      // always snapshot "whatever is currently live" under the caller's chosen name —
      // per docs/feature/002_account_save.md AC-05/AC-12, name need not match the live
      // identity there. The live-identity comparison reads the live session's own
      // oauthAccount.emailAddress directly, not the separate `_active` marker file — the
      // marker can go stale relative to the live session after an external login
      // (BUG-212), which would wrongly suppress the merge for the common
      // name-inferred-from-live-session save path.
      // Pitfall: a function that reads "the machine's live session" to enrich a named
      // account's file is only safe when the caller can guarantee `name` IS the live
      // session's own account — once shared with a caller that saves non-active accounts
      // (background refresh, by design), every unguarded live-session read becomes a
      // cross-account identity leak.
      let live_text    = std::fs::read_to_string( paths.claude_json_file() ).unwrap_or_default();
      let live_val     = serde_json::from_str::< serde_json::Value >( &live_text ).ok();
      let live_is_name = live_val.as_ref()
        .and_then( | v | v.get( "oauthAccount" ) )
        .and_then( | o | o.get( "emailAddress" ) )
        .and_then( | e | e.as_str() )
        .is_some_and( | email | email == name );

      if update_marker || live_is_name
      {
        // Merge oauthAccount from live ~/.claude.json (surgical — only per-account data).
        if let Some( live_val ) = &live_val
        {
          if let Some( oauth ) = live_val.get( "oauthAccount" )
          {
            obj.insert( "oauthAccount".to_string(), oauth.clone() );
          }
        }
        // Merge org identity from endpoint 005 (best-effort, network) — Anthropic-only;
        // meaningless (and would spend a live network call) for a redirect account.
        #[ cfg( feature = "enabled" ) ]
        {
          let creds_text = std::fs::read_to_string( paths.credentials_file() ).unwrap_or_default();
          if let Some( token ) = parse_string_field( &creds_text, "accessToken" )
          {
            if let Ok( roles ) = claude_quota::fetch_claude_cli_roles( &token )
            {
              let val_or_null = | s : &str | -> serde_json::Value
              {
                if s.is_empty() { serde_json::Value::Null }
                else { serde_json::Value::String( s.to_string() ) }
              };
              obj.insert( "organization_uuid".to_string(), serde_json::Value::String( roles.organization_uuid.clone() ) );
              obj.insert( "organization_name".to_string(), serde_json::Value::String( roles.organization_name.clone() ) );
              obj.insert( "organization_role".to_string(), serde_json::Value::String( roles.organization_role.clone() ) );
              obj.insert( "workspace_uuid".to_string(), val_or_null( &roles.workspace_uuid ) );
              obj.insert( "workspace_name".to_string(), val_or_null( &roles.workspace_name ) );
            }
          }
        }
      }
    }
    // Merge model preference from live ~/.claude/settings.json (best-effort, anthropic
    // only). The live `model` key is whatever this machine's current session pinned — for
    // a redirect save that is some other account's preference, and replaying it on the
    // next switch would pin a foreign model. Removing (not merely skipping) also
    // self-heals files saved before this gate existed.
    if backend == AccountBackend::Redirect
    {
      obj.remove( "model" );
    }
    else if let Ok( live_settings ) = std::fs::read_to_string( paths.settings_file() )
    {
      if let Some( model ) = parse_string_field( &live_settings, "model" )
      {
        obj.insert( "model".to_string(), serde_json::Value::String( model ) );
      }
    }
    // Merge profile metadata when provided (CLI callers); None preserves existing values.
    if let Some( h ) = host
    {
      obj.insert( "host".to_string(), serde_json::Value::String( h.to_string() ) );
    }
    if let Some( r ) = role
    {
      obj.insert( "role".to_string(), serde_json::Value::String( r.to_string() ) );
    }
    // `owner` — write when Some (CLI saves); None preserves existing field (background callers).
    if let Some( o ) = owner
    {
      obj.insert( "owner".to_string(), serde_json::Value::String( o.to_string() ) );
    }
    // `inference_provider` — write when Some (`.provider.select` only); None preserves
    // existing field unchanged (every other caller) — Feature 072, no auto-detection.
    if let Some( p ) = inference_provider
    {
      obj.insert( "inference_provider".to_string(), serde_json::Value::String( p.to_string() ) );
    }
    // `tags` — write-through when Some (a tag write: replace semantics + lazy `role`
    // migration in the same merge); None preserves the stored set — Feature 075.
    if let Some( t ) = &tags
    {
      apply_tag_write( obj, &TagOp::Replace( t.clone() ) )?;
    }
  }
  // {name}.json is now always non-empty (backend is always inserted above, Feature 071/AC-04) —
  // this guard now only protects the edge case where a pre-existing file was valid JSON but not
  // an object (as_object_mut() above would have left `snapshot` un-mutated in that case).
  if snapshot.as_object().is_some_and( |obj| !obj.is_empty() )
  {
    // Fix(audit-save-metadata-swallow): metadata write failures now propagate.
    // Root cause: `let _ =` discarded the {name}.json write result, so a full disk or
    // permission error silently dropped backend/owner/renewal metadata while save()
    // reported success — the account then misrendered as anthropic/unowned.
    // Pitfall: the credential write above already succeeded by this point; swallowing
    // the metadata half leaves the two files silently inconsistent with no error trail.
    atomic_write( &meta_path, &serde_json::to_string_pretty( &snapshot ).map( | s | s + "\n" ).unwrap_or_default() )?;
  }

  // Clean up old satellite files (migration to unified {name}.json).
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.claude.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.settings.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.roles.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.profile.json" ) ) );

  if update_marker
  {
    atomic_write( &credential_store.join( active_marker_filename() ), name )?;
  }
  Ok( () )
}

/// Validate that a named account can be deleted (name valid + file exists).
///
/// Called by both `delete` and the CLI dry-run path so that dry-run
/// reports the same errors as a live delete.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist.
#[ inline ]
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
pub fn check_delete_preconditions( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  validate_name( name )?;

  let target = credential_store.join( format!( "{name}.credentials.json" ) );
  if !target.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", credential_store.display() ),
    ) );
  }

  Ok( () )
}

/// Delete a named account from `credential_store`.
///
/// Removes `{name}.credentials.json` and `{name}.json` (unified metadata),
/// plus any legacy satellite files from the pre-consolidation layout.
/// Clears the `_active` marker if it points at the deleted account.
///
/// # Errors
///
/// Returns `NotFound` if the account does not exist.
#[ inline ]
pub fn delete( name : &str, credential_store : &Path ) -> Result< (), std::io::Error >
{
  // `_lock` must be a named binding — `let _ =` would drop (and release) immediately.
  let _lock = lock_store( credential_store )?;
  check_delete_preconditions( name, credential_store )?;
  std::fs::remove_file( credential_store.join( format!( "{name}.credentials.json" ) ) )?;
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.json" ) ) );
  // Clean up legacy satellite files from pre-consolidation layout.
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.claude.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.settings.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.roles.json" ) ) );
  let _ = std::fs::remove_file( credential_store.join( format!( "{name}.profile.json" ) ) );
  // Fix(BUG-347): clear every `_active_*` marker naming this account, not only
  // the calling machine's own marker.
  // Root cause: the guard resolved a single path via `active_marker_filename()`
  // (bound to the CALLING machine's own hostname+user) and compared only that
  // one file's content, so a foreign machine's marker naming the same account
  // was never inspected and survived the delete untouched.
  // Pitfall: do not special-case the own marker again here — `all_marker_files()`
  // already enumerates it alongside every foreign marker, so a single scan
  // covers both without reintroducing the same one-marker blind spot.
  for ( path, content ) in all_marker_files( credential_store )
  {
    if content == name
    {
      let _ = std::fs::remove_file( path );
    }
  }
  Ok( () )
}
