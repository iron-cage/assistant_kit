//! Account mutation command handlers: use, rotate, save, delete, unclaim.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::cmd_args::{ require_nonempty_string_arg, is_dry, io_err_to_error_data, resolve_account_name };
use super::cmd_context::{ require_claude_paths, require_credential_store };
use claude_profile_core::account::trace_ts;

/// `.account.use` — atomic credential rotation by name.
///
/// # Errors
///
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// or the target account does not exist.
#[ inline ]
// Validate-everything-then-mutate ordering is load-bearing (see BUG-265 note below); extracting
// steps into helpers would make that ordering easy to break silently.
#[ allow( clippy::too_many_lines ) ]
pub fn account_use_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Validate all CLI arguments before any I/O (fast-fail on bad values before filesystem access).
  // Fix(BUG-265): is_dry() check comes after existence validation so
  //   dry-run on nonexistent accounts correctly exits 2 (not silently succeeds).
  // Root cause: is_dry() was checked before existence validation, so `dry::1` on a
  //   missing account silently returned exit 0 instead of exit 2.
  // Pitfall: Only the mutating step (file copy + marker write) is skipped in dry-run;
  //   all validation and precondition checks must run unconditionally.
  let raw_name   = require_nonempty_string_arg( &cmd, "name" )?;
  let touch      = crate::output::parse_int_flag( &cmd, "touch", 1 )?;
  let trace      = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let imodel_str = match cmd.arguments.get( "imodel" )
  {
    None                       => "auto".to_string(),
    Some( Value::String( s ) ) =>
    {
      crate::usage::validate_imodel_str( s )
        .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      s.clone()
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "imodel:: must be a string".to_string() ) ),
  };
  let effort_str = match cmd.arguments.get( "effort" )
  {
    None                       => "auto".to_string(),
    Some( Value::String( s ) ) =>
    {
      crate::usage::validate_effort_str( s )
        .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      s.clone()
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "effort:: must be a string".to_string() ) ),
  };
  let set_model_str = match cmd.arguments.get( "set_model" )
  {
    None                       => None,
    Some( Value::String( s ) ) =>
    {
      crate::usage::validate_set_model( s )
        .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
      Some( s.clone() )
    }
    _ => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch, "set_model:: must be a string".to_string() ) ),
  };
  let refresh          = crate::output::parse_int_flag( &cmd, "refresh", 1 )?;
  let paths            = require_claude_paths()?;
  let credential_store = require_credential_store()?;
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_switch_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account use" ) )?;

  // G5: Ownership guard — non-owned accounts cannot be switched to from this machine.
  // Runs before dry::1 so that dry-run still exits 1 on ownership violation.
  // force::1 bypasses the guard (Feature 036 AC-18).
  let force = crate::output::parse_int_flag( &cmd, "force", 0 )? != 0;
  let owner = crate::account::read_owner( &credential_store, &name );
  if !force && !crate::account::is_owned( &owner )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "ownership violation: this account is owned by {owner}" ),
    ) );
  }

  // G9: Claim-lock guard — locked accounts cannot be switched to as an explicit target.
  // Runs before dry::1, mirroring G5. force::1 bypasses the guard (Feature 070 AC-04).
  if !force && crate::account::read_claim_lock( &credential_store, &name )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "claim-lock violation: {name} is claim-locked" ),
    ) );
  }

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would switch to '{name}'\n" ), "text" ) );
  }

  // Pre-fetch quota before the switch while the target credential file is still readable.
  let mut outcome = if touch != 0
  {
    crate::usage::pre_switch_touch_ctx( &name, &credential_store, trace, &imodel_str, &effort_str )
  }
  else
  {
    crate::usage::PreSwitchOutcome::Unavailable
  };

  // Fix(BUG-213): when touch is enabled and the quota fetch failed (Unavailable),
  //   check expiresAt and attempt refresh (BUG-230) before calling switch_account().
  // Root cause: quota fetch failure returned PreSwitchOutcome::Unavailable immediately,
  //   bypassing the expiry check — expired-token accounts were switched without refresh.
  // Pitfall: quota fetch failure ≠ token validity unknown; always check expiresAt independently.
  if touch != 0 && matches!( outcome, crate::usage::PreSwitchOutcome::Unavailable )
  {
    outcome = check_expiry_and_refresh(
      &name, &credential_store, &paths, refresh, trace, &imodel_str, &effort_str,
    );
  }

  crate::account::switch_account( &name, &credential_store, &paths )
    .map_err( |e| io_err_to_error_data( &e, "account use" ) )?;

  // Post-switch: spawn subprocess touch for all fetch-succeeded cases.
  // Fix(BUG-225): Sonnet→Opus session model override when 7d(Son) < 20%.
  // Root cause: switch_account() restored the snapshot model blindly — quota state not consulted.
  // Pitfall: model restoration from snapshot must be followed by quota-aware override; the
  //   snapshot reflects the model at save time, not current quota utilization.
  // Fix(BUG-285): AlreadyActive path removed — the is_idle check used server-side
  //   resets_at as proxy for local subprocess identity (category error). Always spawn;
  //   the subprocess is idempotent and exits immediately when already active.
  // Root cause: resets_at is written by any Claude session on any machine; it cannot
  //   identify the local subprocess.
  // Pitfall: server-side session state is not a substitute for local subprocess identity.
  match outcome
  {
    crate::usage::PreSwitchOutcome::NeedTouch( ctx ) =>
    {
      crate::usage::apply_post_switch_touch( &name, ctx, &imodel_str, &effort_str, trace, &paths, &credential_store );
    }
    crate::usage::PreSwitchOutcome::Unavailable => {}
  }

  // When set_model:: is explicit, write the requested model last (takes precedence over
  // automatic apply_model_override from apply_post_switch_touch).
  if let Some( ref sm ) = set_model_str
  {
    let model_id = crate::usage::validate_set_model( sm ).ok().flatten();
    claude_profile_core::account::set_session_model( &paths, model_id );
    if trace { eprintln!( "{}account.use  {name}  set_model: {sm}", trace_ts() ) }
  }

  // Feature 004 AC-12: running-session advisory. Switching rewrites
  // ~/.claude/.credentials.json for FUTURE launches only — already-running claude
  // sessions hold the previous token in memory and keep billing the old account,
  // which reads as "it said it switched but didn't" (w003 report). Best-effort
  // scan (CLR_PROC_DIR overrides the proc root in tests); stderr so scripts
  // parsing stdout's "switched to" line are unaffected; exit code stays 0.
  let running = claude_core::process::find_claude_processes();
  if !running.is_empty()
  {
    let n = running.len();
    let ( noun, verb ) = if n == 1 { ( "session", "keeps" ) } else { ( "sessions", "keep" ) };
    eprintln!( "warning: {n} running claude {noun} {verb} using the previous account — restart to pick up '{name}'" );
  }

  Ok( OutputData::new( format!( "switched to '{name}'\n" ), "text" ) )
}

/// Check whether the target account's token is expired; attempt refresh if so.
///
/// Called only when `touch` is enabled and `pre_switch_touch_ctx()` returned `Unavailable`.
/// Returns `PreSwitchOutcome` from re-probed quota when refresh succeeds, `Unavailable`
/// when the token is not expired (or the credential file cannot be read). Exits the
/// process with code 3 when the token is expired but cannot be refreshed.
///
/// # Fix(BUG-213)
/// Root cause: `pre_switch_touch_ctx()` returns `None` for any fetch error without
/// distinguishing "token valid but quota unreachable" from "token locally expired".
/// Pitfall: callers treating all `None` returns identically must add their own expiry
/// guard at the decision point, as done here.
///
/// # Fix(BUG-230)
/// Root cause: the original BUG-213 guard exited 3 without attempting OAuth refresh.
/// Token expiry is recoverable when `refresh != 0` (the default).
/// Pitfall: after a successful refresh the `touch_ctx` must be re-probed — the old `None`
/// is stale once the fresh token makes quota fetch viable.
fn check_expiry_and_refresh(
  name             : &str,
  credential_store : &std::path::Path,
  paths            : &crate::ClaudePaths,
  refresh          : i64,
  trace            : bool,
  imodel_str       : &str,
  effort_str       : &str,
) -> crate::usage::PreSwitchOutcome
{
  let cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
  let Ok( cred_str ) = std::fs::read_to_string( &cred_path )
  else { return crate::usage::PreSwitchOutcome::Unavailable };
  // Fix(audit-inline-expiry-parse): delegate to the canonical key-bound parser.
  // Root cause: a verbatim copy of parse_u64_field's needle/trim/digit-run algorithm
  //   lived inline here — claude_profile_core::token::parse_expires_at already binds
  //   that parser to the "expiresAt" key.
  // Pitfall: the value is milliseconds (as stored in credentials JSON), not seconds.
  let Some( exp_ms ) = claude_profile_core::token::parse_expires_at( &cred_str )
  else { return crate::usage::PreSwitchOutcome::Unavailable };
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_ms = u64::try_from(
    SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_millis()
  ).unwrap_or( u64::MAX );
  if now_ms <= exp_ms
  {
    if trace
    {
      let rem_s = ( exp_ms - now_ms ) / 1000;
      eprintln!( "{}account.use  {name}  expiry check: valid (expires in {}h {}m)", trace_ts(), rem_s / 3600, ( rem_s % 3600 ) / 60 );
    }
    return crate::usage::PreSwitchOutcome::Unavailable;
  }
  let elapsed_s = ( now_ms - exp_ms ) / 1000;
  let h         = elapsed_s / 3600;
  let m         = ( elapsed_s % 3600 ) / 60;
  if refresh != 0
  {
    if trace { eprintln!( "{}account.use  {name}  expiry check: expired({h}h {m}m ago) → attempting refresh", trace_ts() ) }
    let refreshed = crate::usage::attempt_expired_token_refresh( name, credential_store, paths, trace, imodel_str, effort_str );
    if refreshed
    {
      if trace { eprintln!( "{}account.use  {name}  expiry check: refresh OK — re-probing touch context", trace_ts() ) }
      return crate::usage::pre_switch_touch_ctx( name, credential_store, trace, imodel_str, effort_str );
    }
    if trace { eprintln!( "{}account.use  {name}  expiry check: refresh failed → refused", trace_ts() ) }
    eprintln!( "account credentials expired and refresh failed: {name} (expired {h}h {m}m ago)" );
  }
  else
  {
    if trace { eprintln!( "{}account.use  {name}  expiry check: expired({h}h {m}m ago) → refused (refresh::0)", trace_ts() ) }
    eprintln!( "account credentials expired: {name} (expired {h}h {m}m ago)" );
  }
  std::process::exit( 3 );
}

/// `.account.save` — save current credentials as a named account profile.
///
/// # Errors
///
/// Returns `ErrorData` if the name cannot be resolved (explicit empty value or
/// `_active` marker absent from the credential store), HOME is unset,
/// or the credential copy fails.
#[ inline ]
// Sequential handler — name resolution (explicit vs _active marker), then the credential copy;
// each step consumes the previous step's locals.
#[ allow( clippy::too_many_lines ) ]
pub fn account_save_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let paths            = require_claude_paths()?;
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let name             = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    Some( Value::String( _ ) ) =>
      return Err( ErrorData::new( ErrorCode::ArgumentMissing, "name:: value cannot be empty".to_string() ) ),
    _ =>
    {
      // Fix(BUG-212): read oauthAccount.emailAddress from ~/.claude.json as primary inference source;
      //   fall back to _active marker only when emailAddress is absent or empty.
      // Root cause: BUG-209 fix replaced stale top-level emailAddress with _active marker, but the marker
      //   is only written by clp ops (switch_account, save). External OAuth login writes ~/.claude.json
      //   (including oauthAccount.emailAddress) without updating _active — leaving the marker stale.
      // Pitfall: any single-source inference fails when other credential-change paths bypass that source.
      //   oauthAccount.emailAddress is updated by BOTH clp switches (snapshot restore) AND external OAuth
      //   login (Claude writes ~/.claude.json on every auth). _active is clp-only.
      let cs          = require_credential_store()?;
      let cj          = std::fs::read_to_string( paths.claude_json_file() ).unwrap_or_default();
      // Extract emailAddress nested inside oauthAccount {…}: locate "oauthAccount": first,
      // then apply parse_string_field on the suffix so only the nested key is found.
      let oauth_email = cj
        .find( "\"oauthAccount\":" )
        .and_then( | pos | crate::account::parse_string_field( &cj[ pos.. ], "emailAddress" ) )
        .filter( | s | !s.is_empty() );
      if let Some( email ) = oauth_email
      {
        email
      }
      else
      {
        let marker_path = cs.join( crate::account::active_marker_filename() );
        std::fs::read_to_string( &marker_path )
          .ok()
          .map( | s | s.trim().to_string() )
          .filter( | s | !s.is_empty() )
          .ok_or_else( || ErrorData::new(
            ErrorCode::ArgumentMissing,
            "cannot infer account name: no active account set — pass name:: explicitly".to_string(),
          ) )?
      }
    }
  };
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "{}account.save  reading {}", trace_ts(), paths.credentials_file().display() ) }

  // Feature 071: parse backend:: before name validation, so redirect accounts can use an
  // arbitrary label (validate_redirect_name()) instead of the email-shape validate_name()
  // required for backend: anthropic. backend:: is matched case-insensitively here (CLI-input
  // boundary, per docs/cli/param/069_backend.md's case-insensitive constraint) —
  // AccountBackend::parse() itself stays exact-match, since its other callers read the
  // always-canonical-lowercase stored `backend` field.
  let backend_raw = match cmd.arguments.get( "backend" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };

  // Feature 073/078: preset:: is sugar for known-provider redirect accounts — pre-fills
  // backend::/base_url::/inference_provider:: for anything the caller didn't already
  // specify explicitly. "kimi" (Feature 073) and "deepseek" (Feature 078) are the two
  // recognized values today — each new preset is an explicit design addition, not a
  // data-driven provider registry (see docs/type/007_preset.md).
  let preset_raw = match cmd.arguments.get( "preset" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };
  if !preset_raw.is_empty() && !preset_raw.eq_ignore_ascii_case( "kimi" ) && !preset_raw.eq_ignore_ascii_case( "deepseek" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "preset:: invalid value '{preset_raw}' — valid values: kimi, deepseek" ),
    ) );
  }
  let preset_is_kimi     = preset_raw.eq_ignore_ascii_case( "kimi" );
  let preset_is_deepseek = preset_raw.eq_ignore_ascii_case( "deepseek" );
  let backend_raw        = if backend_raw.is_empty() && ( preset_is_kimi || preset_is_deepseek ) { "redirect".to_string() } else { backend_raw };

  if !backend_raw.is_empty()
    && !backend_raw.eq_ignore_ascii_case( "anthropic" )
    && !backend_raw.eq_ignore_ascii_case( "redirect" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "backend:: invalid value '{backend_raw}' — valid values: anthropic, redirect" ),
    ) );
  }
  let backend = crate::account::AccountBackend::parse( &backend_raw.to_lowercase() );

  // Fix(BUG-549): bare save (no backend::, no preset::) on a stored-redirect target is a
  // no-op skip, never an implicit re-backend — 071 AC-19.
  // Root cause: absent backend:: resolved to AccountBackend::Anthropic by type default
  //   before the stored record was consulted; a stored-redirect target then rode the AC-15
  //   delete-and-rewrite path, so the watchdog's routine per-tick bare `.account.save`
  //   destroyed the redirect record (base_url/redirect_model/inference_provider/claim_lock)
  //   and overwrote its static key with the live OAuth session — silently, fleet-synced.
  // Pitfall: do NOT default absent backend:: to the stored backend instead — save()'s
  //   redirect branch writes accessToken from the (absent) api_key:: bytes, clobbering the
  //   stored static key with an empty string. A redirect account has no live OAuth session
  //   to snapshot; skip is the only non-destructive meaning for a snapshot-style bare call.
  //   Explicit backend::redirect re-saves and explicit backend::anthropic re-backends (AC-15)
  //   are deliberate acts and stay untouched.
  // Fix(BUG-554): the skip covers only a *genuinely bare* save — one carrying no mutation at
  //   all. BUG-549's gate tested backend::/preset:: alone (AC-19's literal two-condition text)
  //   and sat upstream of every other parameter's validation, so any mutation-bearing save on
  //   a stored-redirect target — `api_key::NEW`, `tags::work`, … — exited 0 with "save skipped"
  //   and wrote nothing: a deliberate operator mutation discarded under a success exit code.
  // Root cause: the gate encoded "which flags the AC happened to name" rather than "does this
  //   call actually carry a mutation", so every parameter added after BUG-549 inherited the
  //   swallow by default.
  // Pitfall: do NOT fix this by defaulting the absent backend:: to the stored `redirect` and
  //   letting the write proceed — save()'s redirect branch writes accessToken from the (absent)
  //   api_key:: bytes, clobbering the stored static key with an empty string. Intent is
  //   ambiguous here, so it is rejected rather than guessed.
  if backend_raw.is_empty() && !preset_is_kimi && !preset_is_deepseek
  {
    let meta_text      = std::fs::read_to_string( credential_store.join( format!( "{name}.json" ) ) ).unwrap_or_default();
    let stored_backend = crate::account::parse_string_field( &meta_text, "backend" ).unwrap_or_default();
    if stored_backend == "redirect"
    {
      // Split the carried mutators by whether one already has a rejection of its own further
      // down. base_url::/api_key::/redirect_model:: hit the redirect-only check and role:: hits
      // the Feature 075 removal notice, so those fall through to the precise message the
      // operator expects. host::/tags::/inference_provider:: are legal on an anthropic-backend
      // save and have no such guard — falling through would default backend:: to anthropic and
      // ride AC-15's delete-and-rewrite straight back into BUG-549's destruction.
      let unguarded : Vec< String > = [ "host", "tags", "inference_provider" ].iter()
        .filter( | p | matches!( cmd.arguments.get( **p ), Some( Value::String( s ) ) if !s.is_empty() ) )
        .map( | p | format!( "{p}::" ) )
        .collect();
      if !unguarded.is_empty()
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentMissing,
          format!(
            "'{name}' is a redirect account — {} needs an explicit backend:: to apply; pass \
             backend::redirect to modify the redirect record (api_key:: required) or \
             backend::anthropic to convert it to an OAuth account",
            unguarded.join( ", " ),
          ),
        ) );
      }
      let guarded_elsewhere = cmd.arguments.contains_key( "role" )
        || [ "base_url", "api_key", "redirect_model" ].iter()
          .any( | p | matches!( cmd.arguments.get( *p ), Some( Value::String( s ) ) if !s.is_empty() ) );
      if !guarded_elsewhere
      {
        if trace { eprintln!( "{}account.save  skipped (reason: redirect backend — static credentials, nothing to snapshot)", trace_ts() ) }
        return Ok( OutputData::new(
          format!( "'{name}' is a redirect account (static credentials) — nothing to snapshot; save skipped\n" ),
          "text",
        ) );
      }
    }
  }

  // Validate name before dry-run check so dry-run rejects invalid names instead of reporting
  // "[dry-run] would save" for names that would fail. AC-15: an already-saved account's name
  // is not re-validated against a newly requested backend's stricter shape rule on re-save.
  crate::account::validate_name_for_save( &name, backend, &credential_store )
    .map_err( | e | io_err_to_error_data( &e, "account save" ) )?;

  // Feature 071: parse base_url::/api_key::/redirect_model:: before the dry-run check, so
  // validation failures reject even under dry::1 (mirrors validate_name()'s ordering above —
  // dry-run must not mask a rejection a real run would hit).
  let base_url_val = match cmd.arguments.get( "base_url" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _                                           => None,
  };
  // Feature 073/078: preset::kimi/preset::deepseek fill base_url:: with that provider's
  // fixed Anthropic-compatible endpoint when the caller didn't pass one explicitly. Gated
  // on the resolved `backend` (not bare preset_is_kimi/preset_is_deepseek) so an explicit
  // backend::anthropic paired with either preset never forces a redirect-only field onto
  // an anthropic-backend save.
  let base_url_val = if base_url_val.is_none() && backend == crate::account::AccountBackend::Redirect
  {
    if preset_is_kimi { Some( "https://api.moonshot.ai/anthropic".to_string() ) }
    else if preset_is_deepseek { Some( "https://api.deepseek.com/anthropic".to_string() ) }
    else { None }
  }
  else
  {
    base_url_val
  };
  let api_key_val = match cmd.arguments.get( "api_key" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _                                           => None,
  };
  let redirect_model_val = match cmd.arguments.get( "redirect_model" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _                                           => None,
  };
  if backend == crate::account::AccountBackend::Redirect
  {
    let mut missing = Vec::new();
    if base_url_val.is_none()       { missing.push( "base_url::" ); }
    if api_key_val.is_none()        { missing.push( "api_key::" ); }
    if redirect_model_val.is_none() { missing.push( "redirect_model::" ); }
    if !missing.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentMissing,
        format!( "backend::redirect requires base_url::, api_key::, redirect_model:: — missing: {}", missing.join( ", " ) ),
      ) );
    }
  }
  else
  {
    let mut unexpected = Vec::new();
    if base_url_val.is_some()       { unexpected.push( "base_url::" ); }
    if api_key_val.is_some()        { unexpected.push( "api_key::" ); }
    if redirect_model_val.is_some() { unexpected.push( "redirect_model::" ); }
    if !unexpected.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "{} redirect-only — requires backend::redirect", unexpected.join( ", " ) ),
      ) );
    }
  }

  // Feature 075: role:: is REMOVED — a role value is now just a tag. Rejected loudly
  // (never silently ignored) so scripted callers migrate instead of losing data.
  if cmd.arguments.contains_key( "role" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "role:: REMOVED — use tags:: instead (a role value is now just a tag, e.g. tags::work)".to_string(),
    ) );
  }
  // Feature 075: tags:: replaces the whole stored tag set (comma-separated). Validated
  // before the dry-run check, mirroring base_url::'s ordering above — dry-run must not
  // mask a rejection a real run would hit. save() re-validates internally before writing.
  let tags_val : Option< Vec< String > > = match cmd.arguments.get( "tags" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.split( ',' ).map( str::to_string ).collect() ),
    _                                           => None,
  };
  if let Some( ref raw ) = tags_val
  {
    crate::account::normalize_tag_set( raw )
      .map_err( | e | io_err_to_error_data( &e, "account save" ) )?;
  }

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would save current credentials as '{name}'\n" ), "text" ) );
  }

  // Resolve host profile metadata before calling save().
  // host:: defaults to auto-captured $USER@<hostname> when omitted.
  let host_val = match cmd.arguments.get( "host" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    _ =>
    {
      let user     = std::env::var( "USER" ).unwrap_or_default();
      let hostname = crate::account::resolve_hostname();
      format!( "{user}@{hostname}" )
    }
  };
  // Ownership-neutral: preserves existing owner via read-merge.
  // Owner can only be set by write_owner() — no CLI-exposed set path.
  // Redirect accounts capture api_key:: as the credential payload; Anthropic accounts keep
  // capturing the live ~/.claude/.credentials.json session (creds: None — see save()'s own
  // None branch), preserving exact pre-071 behavior when backend:: is absent.
  let creds_bytes = if backend == crate::account::AccountBackend::Redirect
  {
    api_key_val.as_deref().map( str::as_bytes )
  }
  else
  {
    None
  };
  // Feature 072: inference_provider:: is a plain tag written verbatim to `{name}.json` —
  // no fallback-chain/auto-detection logic here (provider selection is a manual, global
  // config value written solely via `.provider.select`; this field only records which
  // provider this saved account's credentials belong to).
  let inference_provider_val = match cmd.arguments.get( "inference_provider" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    Some( Value::String( _ ) ) => return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "inference_provider:: must be a non-empty value".to_string(),
    ) ),
    _ => None,
  };
  // Feature 073/078: preset::kimi/preset::deepseek fill inference_provider:: with that
  // provider's own tag when the caller didn't pass one explicitly — this is what makes
  // switch_account() write the matching tier env vars
  // (claude_profile_core::account::write_kimi_tier_env_vars()/write_deepseek_tier_env_vars()).
  // Same resolved-backend gating as base_url_val above.
  let inference_provider_val = if inference_provider_val.is_none() && backend == crate::account::AccountBackend::Redirect
  {
    if preset_is_kimi { Some( "kimi".to_string() ) }
    else if preset_is_deepseek { Some( "deepseek".to_string() ) }
    else { None }
  }
  else
  {
    inference_provider_val
  };
  // AC-15/no-partial-update (Out of Scope note: claude_profile_core's save() merge logic is
  // task 433's and stays untouched) — save() read-merges the existing {name}.json, so a
  // re-save that changes backend would otherwise leave the prior backend's fields (e.g. a
  // stale base_url/redirect_model from a prior redirect save) behind. When the on-disk
  // backend differs from the one being written, delete the meta file first so save() starts
  // from empty and genuinely rewrites from scratch; an ordinary same-backend re-save (the
  // common case, including every pre-071 account) is untouched — parse_string_field() is used
  // instead of a JSON library per this crate's zero-third-party-deps-in-src rule.
  let meta_path        = credential_store.join( format!( "{name}.json" ) );
  let existing_meta    = std::fs::read_to_string( &meta_path ).unwrap_or_default();
  let old_backend      = crate::account::parse_string_field( &existing_meta, "backend" ).unwrap_or_else( || "anthropic".to_string() );
  if !existing_meta.is_empty() && old_backend != backend.as_str()
  {
    let _ = std::fs::remove_file( &meta_path );
  }

  crate::account::save(
    &name, &credential_store, &paths, true, creds_bytes, Some( &host_val ), None, None,
    backend, base_url_val.as_deref(), redirect_model_val.as_deref(), inference_provider_val.as_deref(),
    tags_val.as_deref(),
  )
    .map_err( |e| io_err_to_error_data( &e, "account save" ) )?;

  if trace { eprintln!( "{}account.save  write: OK  host={host_val}", trace_ts() ) }

  Ok( OutputData::new( format!( "saved current credentials as '{name}'\n" ), "text" ) )
}

/// `.account.delete` — delete a saved account (guard: refuses active).
///
/// # Errors
///
/// Returns `ErrorData` if name is missing/empty, HOME is unset,
/// or the account does not exist.
#[ inline ]
pub fn account_delete_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Fix(BUG-266):
  // Root cause: is_dry() was checked before existence check,
  //   so dry-run bypassed NotFound (missing account).
  // Pitfall: precondition checks must run before the dry-run shortcut.
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let raw_name         = require_nonempty_string_arg( &cmd, "name" )?;
  let credential_store = require_credential_store()?;
  if trace { eprintln!( "{}account.delete  store: {}", trace_ts(), credential_store.display() ) }
  let name             = resolve_account_name( &raw_name, &credential_store )?;
  crate::account::check_delete_preconditions( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account delete" ) )?;

  // G6: Ownership guard — non-owned accounts cannot be deleted from this machine.
  // Runs before dry::1 so that dry-run still exits 1 on ownership violation.
  // force::1 bypasses the guard (Feature 036 AC-19).
  let force = crate::output::parse_int_flag( &cmd, "force", 0 )? != 0;
  let owner = crate::account::read_owner( &credential_store, &name );
  if !force && !crate::account::is_owned( &owner )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "ownership violation: this account is owned by {owner}" ),
    ) );
  }

  if is_dry( &cmd )
  {
    return Ok( OutputData::new( format!( "[dry-run] would delete account '{name}'\n" ), "text" ) );
  }

  crate::account::delete( &name, &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "account delete" ) )?;
  Ok( OutputData::new( format!( "deleted account '{name}'\n" ), "text" ) )
}


