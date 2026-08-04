//! Version management helpers for Claude Code.
//!
//! Provides version detection, alias resolution, installation, and preference persistence.
//! These are pure domain operations with no CLI framework dependencies.

use claude_core::ClaudePaths;
use claude_core::process::find_claude_processes;
use claude_core::settings_io::{ get_setting, set_setting, remove_setting, set_env_var, remove_env_var };
use crate::CoreError;

// ── Constants ─────────────────────────────────────────────────────────────────

const INSTALL_URL : &str = "https://claude.ai/install.sh";

// ── Version alias table ───────────────────────────────────────────────────────
//
// Maintenance: when bumping `stable`/`month` (or any pinned alias value),
// grep the repo for the old literal (e.g. `2\.1\.78`) and update every
// genuine hit — a partial update silently breaks doc/test consistency.
// Known locations as of the 2.1.220 bump:
//   1. module/claude_version_core/src/version.rs — this table (the canonical source)
//   2. module/claude_version/docs/feature/001_version_management.md — alias table
//   3. module/claude_version/docs/cli/user_story/001_environment_check.md — example walkthrough transcript
//   4. module/claude_version/docs/cli/type/03_version_spec.md — named alias resolution table
//   5. module/claude_version/docs/cli/format/01_text.md — named alias table format example
//   6. module/claude_version/tests/docs/cli/type/03_version_spec.md — TC-1 test-planning spec
//   7. module/claude_version/tests/docs/cli/param/01_version.md — EC-10 test-planning spec
//   8. module/claude_version/tests/docs/feature/001_version_management.md — FT-1 test-planning spec
//   9. module/claude_version/tests/cli/feature_surface_test.rs — FT-1 cross-reference comment
// Not every file containing the literal string needs updating — most Rust
// test fixtures use it as arbitrary-but-consistent fixture data (lock-state
// drift detection, verbosity rendering) decoupled from this table, or derive
// their expected value programmatically from VERSION_ALIASES itself.

/// A named version alias that resolves to a specific semver or the literal `"latest"`.
#[ derive( Debug ) ]
pub struct VersionAlias
{
  /// Short alias name used on the CLI (e.g. `"stable"`, `"month"`, `"latest"`).
  pub name        : &'static str,
  /// Resolved semver string, or empty string for the `latest` alias.
  pub value       : &'static str,
  /// Human-readable description shown in `.version.list` output.
  pub description : &'static str,
}

/// All known version aliases in display order.
pub const VERSION_ALIASES : &[ VersionAlias ] = &[
  VersionAlias { name : "latest", value : "",       description : "Most recent published release" },
  VersionAlias { name : "stable", value : "2.1.220", description : "Pinned stable release (recommended)" },
  VersionAlias { name : "month",  value : "2.1.74", description : "~1 month old release for stability" },
];

// ── Version history snapshot ──────────────────────────────────────────────────
//
// Compiled-in fallback for `.version.history` when the live GitHub Releases API
// fetch and the local 1-hour cache both fail (e.g. no network). Generated from
// the archived changelog table at `contract/claude_code/docs/version/readme.md`
// (versions 2.1.74-2.1.220, newest first). `summary` reuses that table's
// 47-character truncated Summary column verbatim -- regenerating this array means
// re-deriving it from the same source, not writing fresh summaries.
//
// The 2.1.120 entry has `date : "unknown"` -- its GitHub release was retracted
// after being archived; no fabricated date is used.
//
// Regeneration: re-fetch `https://api.github.com/repos/anthropics/claude-code/releases`,
// update the doc archive's Overview Table (ID, Version, Date, Summary, Status),
// then regenerate this array from the updated table, newest version first.

/// A single compiled-in release-history record (version, date, one-line summary).
#[ derive( Debug ) ]
pub struct VersionRecord
{
  /// Semver string without a leading `v` (e.g. `"2.1.220"`).
  pub version : &'static str,
  /// Release date in `YYYY-MM-DD` form, or `"unknown"` if the source release was retracted.
  pub date    : &'static str,
  /// First changelog bullet, truncated to 47 characters with a trailing `...` if cut.
  pub summary : &'static str,
}

/// Compiled-in release history, newest first. Fallback source for `.version.history`
/// when live fetch and cache both fail; see module-level comment above for provenance.
pub const VERSION_HISTORY : &[ VersionRecord ] = &[
  VersionRecord { version : "2.1.220", date : "2026-07-25", summary : "Bug fixes and reliability improvements" },
  VersionRecord { version : "2.1.219", date : "2026-07-24", summary : "Added Claude Opus 5 (`claude-opus-5`), now the ..." },
  VersionRecord { version : "2.1.218", date : "2026-07-22", summary : "Changed `/code-review` to run as a background s..." },
  VersionRecord { version : "2.1.217", date : "2026-07-21", summary : "Added emoji shortcode autocomplete in the promp..." },
  VersionRecord { version : "2.1.216", date : "2026-07-20", summary : "Added `sandbox.filesystem.disabled` setting to ..." },
  VersionRecord { version : "2.1.215", date : "2026-07-19", summary : "Claude no longer runs the `/verify` and `/code-..." },
  VersionRecord { version : "2.1.214", date : "2026-07-18", summary : "Fixed single-segment `dir/**` allow rules like ..." },
  VersionRecord { version : "2.1.212", date : "2026-07-17", summary : "`/fork` now copies your conversation into a new..." },
  VersionRecord { version : "2.1.211", date : "2026-07-15", summary : "Added `--forward-subagent-text` flag and `CLAUD..." },
  VersionRecord { version : "2.1.210", date : "2026-07-14", summary : "Added a live elapsed-time counter to the collap..." },
  VersionRecord { version : "2.1.209", date : "2026-07-14", summary : "Fixed /model and other dialogs being blocked in..." },
  VersionRecord { version : "2.1.208", date : "2026-07-14", summary : "Added screen reader mode: opt-in plain-text ren..." },
  VersionRecord { version : "2.1.207", date : "2026-07-11", summary : "Auto mode is now available without `CLAUDE_CODE..." },
  VersionRecord { version : "2.1.206", date : "2026-07-10", summary : "Added directory path suggestions to `/cd`, matc..." },
  VersionRecord { version : "2.1.205", date : "2026-07-08", summary : "Added an auto mode rule that blocks tampering w..." },
  VersionRecord { version : "2.1.204", date : "2026-07-08", summary : "Fixed hook events not streaming during SessionS..." },
  VersionRecord { version : "2.1.203", date : "2026-07-07", summary : "Added a warning when your login is about to exp..." },
  VersionRecord { version : "2.1.202", date : "2026-07-06", summary : "Added a \"Dynamic workflow size\" setting in `/co..." },
  VersionRecord { version : "2.1.201", date : "2026-07-03", summary : "Claude Sonnet 5 sessions no longer use the mid-c..." },
  VersionRecord { version : "2.1.200", date : "2026-07-03", summary : "Changed `AskUserQuestion` dialogs to no longer a..." },
  VersionRecord { version : "2.1.199", date : "2026-07-02", summary : "Stacked slash-skill invocations like `/skill-a /..." },
  VersionRecord { version : "2.1.198", date : "2026-07-01", summary : "Claude in Chrome is now generally available" },
  VersionRecord { version : "2.1.197", date : "2026-06-30", summary : "Introducing Claude Sonnet 5: now the default mo..." },
  VersionRecord { version : "2.1.196", date : "2026-06-29", summary : "Added support for organization default models —..." },
  VersionRecord { version : "2.1.195", date : "2026-06-26", summary : "Added `CLAUDE_CODE_DISABLE_MOUSE_CLICKS` to dis..." },
  VersionRecord { version : "2.1.193", date : "2026-06-25", summary : "Added `autoMode.classifyAllShell` setting to ro..." },
  VersionRecord { version : "2.1.191", date : "2026-06-24", summary : "Added `/rewind` support for resuming a conversa..." },
  VersionRecord { version : "2.1.190", date : "2026-06-24", summary : "Bug fixes and reliability improvements" },
  VersionRecord { version : "2.1.187", date : "2026-06-23", summary : "Added `sandbox.credentials` setting to block sa..." },
  VersionRecord { version : "2.1.186", date : "2026-06-22", summary : "Added `claude mcp login <name>` and `claude mcp..." },
  VersionRecord { version : "2.1.185", date : "2026-06-20", summary : "The stream-stall hint now reads \"Waiting for AP..." },
  VersionRecord { version : "2.1.183", date : "2026-06-19", summary : "Improved auto mode safety: destructive git comm..." },
  VersionRecord { version : "2.1.181", date : "2026-06-17", summary : "Added `/config key=value` syntax to set any set..." },
  VersionRecord { version : "2.1.179", date : "2026-06-16", summary : "Fixed mid-stream connection drops: partial resp..." },
  VersionRecord { version : "2.1.178", date : "2026-06-15", summary : "Agent teams: removed the `TeamCreate` and `Team..." },
  VersionRecord { version : "2.1.176", date : "2026-06-12", summary : "Session titles are now generated in the languag..." },
  VersionRecord { version : "2.1.175", date : "2026-06-12", summary : "Added `enforceAvailableModels` managed setting ..." },
  VersionRecord { version : "2.1.174", date : "2026-06-12", summary : "Added `wheelScrollAccelerationEnabled` setting ..." },
  VersionRecord { version : "2.1.173", date : "2026-06-11", summary : "Fixed Fable 5 model names with a `[1m]` suffix ..." },
  VersionRecord { version : "2.1.172", date : "2026-06-10", summary : "Sub-agents can now spawn their own sub-agents (..." },
  VersionRecord { version : "2.1.170", date : "2026-06-09", summary : "Introducing Claude Fable 5: a Mythos-class mode..." },
  VersionRecord { version : "2.1.169", date : "2026-06-08", summary : "Self-hosted runner: added a `post-session` life..." },
  VersionRecord { version : "2.1.168", date : "2026-06-06", summary : "Bug fixes and reliability improvements" },
  VersionRecord { version : "2.1.167", date : "2026-06-06", summary : "Bug fixes and reliability improvements" },
  VersionRecord { version : "2.1.166", date : "2026-06-06", summary : "Added `fallbackModel` setting to configure up t..." },
  VersionRecord { version : "2.1.165", date : "2026-06-05", summary : "Bug fixes and reliability improvements" },
  VersionRecord { version : "2.1.163", date : "2026-06-04", summary : "Added `requiredMinimumVersion` and `requiredMax..." },
  VersionRecord { version : "2.1.162", date : "2026-06-03", summary : "`claude agents --json` now includes `waitingFor..." },
  VersionRecord { version : "2.1.161", date : "2026-06-02", summary : "`OTEL_RESOURCE_ATTRIBUTES` values are now inclu..." },
  VersionRecord { version : "2.1.160", date : "2026-06-02", summary : "Added a prompt before writing to shell startup ..." },
  VersionRecord { version : "2.1.159", date : "2026-05-31", summary : "Internal infrastructure improvements (no user-f..." },
  VersionRecord { version : "2.1.158", date : "2026-05-30", summary : "Auto mode is now available on Bedrock, Vertex, ..." },
  VersionRecord { version : "2.1.157", date : "2026-05-29", summary : "Plugins in `.claude/skills` directories are now..." },
  VersionRecord { version : "2.1.156", date : "2026-05-29", summary : "Fixed an issue when using Opus 4.8 where thinki..." },
  VersionRecord { version : "2.1.154", date : "2026-05-28", summary : "Opus 4.8 is here! Now defaults to high effort ·..." },
  VersionRecord { version : "2.1.153", date : "2026-05-28", summary : "Added `skipLfs` option to `github`/`git` plugin..." },
  VersionRecord { version : "2.1.152", date : "2026-05-27", summary : "`/code-review --fix` now applies review finding..." },
  VersionRecord { version : "2.1.150", date : "2026-05-23", summary : "Internal infrastructure improvements (no user-f..." },
  VersionRecord { version : "2.1.149", date : "2026-05-22", summary : "`/usage` now shows a per-category breakdown of ..." },
  VersionRecord { version : "2.1.148", date : "2026-05-22", summary : "Fixed the Bash tool returning exit code 127 on ..." },
  VersionRecord { version : "2.1.147", date : "2026-05-21", summary : "Pinned background sessions (`Ctrl+T` in `claude..." },
  VersionRecord { version : "2.1.145", date : "2026-05-19", summary : "Added `claude agents --json` to list live Claud..." },
  VersionRecord { version : "2.1.144", date : "2026-05-19", summary : "Added `/resume` support for background sessions..." },
  VersionRecord { version : "2.1.143", date : "2026-05-15", summary : "Added plugin dependency enforcement: `claude pl..." },
  VersionRecord { version : "2.1.142", date : "2026-05-14", summary : "Added new `claude agents` flags: `--add-dir`, `..." },
  VersionRecord { version : "2.1.141", date : "2026-05-13", summary : "Added `terminalSequence` field to hook JSON out..." },
  VersionRecord { version : "2.1.140", date : "2026-05-12", summary : "Improved Agent tool `subagent_type` matching to..." },
  VersionRecord { version : "2.1.139", date : "2026-05-11", summary : "Added agent view (Research Preview): a single l..." },
  VersionRecord { version : "2.1.138", date : "2026-05-09", summary : "Internal fixes" },
  VersionRecord { version : "2.1.137", date : "2026-05-09", summary : "[VSCode] Fixed extension failing to activate on..." },
  VersionRecord { version : "2.1.136", date : "2026-05-08", summary : "Added `CLAUDE_CODE_ENABLE_FEEDBACK_SURVEY_FOR_O..." },
  VersionRecord { version : "2.1.133", date : "2026-05-07", summary : "Added `worktree.baseRef` setting (`fresh` | `he..." },
  VersionRecord { version : "2.1.132", date : "2026-05-06", summary : "Added `CLAUDE_CODE_SESSION_ID` environment vari..." },
  VersionRecord { version : "2.1.131", date : "2026-05-06", summary : "Fixed VS Code extension failing to activate on ..." },
  VersionRecord { version : "2.1.129", date : "2026-05-06", summary : "Added `--plugin-url <url>` flag to fetch a plug..." },
  VersionRecord { version : "2.1.128", date : "2026-05-04", summary : "Bare `/color` (no args) now picks a random sess..." },
  VersionRecord { version : "2.1.126", date : "2026-05-01", summary : "The `/model` picker now lists models from your ..." },
  VersionRecord { version : "2.1.123", date : "2026-04-29", summary : "Fixed OAuth authentication failing with a 401 r..." },
  VersionRecord { version : "2.1.122", date : "2026-04-28", summary : "Added `ANTHROPIC_BEDROCK_SERVICE_TIER` environm..." },
  VersionRecord { version : "2.1.121", date : "2026-04-28", summary : "Added `alwaysLoad` option to MCP server config ..." },
  VersionRecord { version : "2.1.120", date : "unknown", summary : "Windows: Git for Windows (Git Bash) is no longe..." },
  VersionRecord { version : "2.1.119", date : "2026-04-23", summary : "`/config` settings (theme, editor mode, verbose..." },
  VersionRecord { version : "2.1.118", date : "2026-04-23", summary : "Added vim visual mode (`v`) and visual-line mod..." },
  VersionRecord { version : "2.1.117", date : "2026-04-22", summary : "Forked subagents can now be enabled on external..." },
  VersionRecord { version : "2.1.116", date : "2026-04-20", summary : "`/resume` on large sessions is significantly fa..." },
  VersionRecord { version : "2.1.114", date : "2026-04-18", summary : "Fixed a crash in the permission dialog when an ..." },
  VersionRecord { version : "2.1.113", date : "2026-04-17", summary : "Changed the CLI to spawn a native Claude Code b..." },
  VersionRecord { version : "2.1.112", date : "2026-04-16", summary : "Fixed \"claude-opus-4-7 is temporarily unavailab..." },
  VersionRecord { version : "2.1.111", date : "2026-04-16", summary : "Claude Opus 4.7 xhigh is now available! Use /ef..." },
  VersionRecord { version : "2.1.110", date : "2026-04-15", summary : "Added `/tui` command and `tui` setting — run `/..." },
  VersionRecord { version : "2.1.109", date : "2026-04-15", summary : "Improved the extended-thinking indicator with a..." },
  VersionRecord { version : "2.1.108", date : "2026-04-14", summary : "Added `ENABLE_PROMPT_CACHING_1H` env var to opt..." },
  VersionRecord { version : "2.1.107", date : "2026-04-14", summary : "Show thinking hints sooner during long operations" },
  VersionRecord { version : "2.1.105", date : "2026-04-13", summary : "Added `path` parameter to the `EnterWorktree` t..." },
  VersionRecord { version : "2.1.101", date : "2026-04-10", summary : "Added `/team-onboarding` command to generate a ..." },
  VersionRecord { version : "2.1.98", date : "2026-04-09", summary : "Added interactive Google Vertex AI setup wizard..." },
  VersionRecord { version : "2.1.97", date : "2026-04-08", summary : "Added focus view toggle (`Ctrl+O`) in `NO_FLICK..." },
  VersionRecord { version : "2.1.96", date : "2026-04-08", summary : "Fixed Bedrock requests failing with `403 \"Autho..." },
  VersionRecord { version : "2.1.94", date : "2026-04-07", summary : "Added support for Amazon Bedrock powered by Man..." },
  VersionRecord { version : "2.1.92", date : "2026-04-04", summary : "Added `forceRemoteSettingsRefresh` policy setti..." },
  VersionRecord { version : "2.1.91", date : "2026-04-02", summary : "Added MCP tool result persistence override via ..." },
  VersionRecord { version : "2.1.90", date : "2026-04-01", summary : "Added `/powerup` — interactive lessons teaching..." },
  VersionRecord { version : "2.1.89", date : "2026-04-01", summary : "Added `\"defer\"` permission decision to `PreTool..." },
  VersionRecord { version : "2.1.87", date : "2026-03-29", summary : "Fixed messages in Cowork Dispatch not getting d..." },
  VersionRecord { version : "2.1.86", date : "2026-03-27", summary : "Added `X-Claude-Code-Session-Id` header to API ..." },
  VersionRecord { version : "2.1.85", date : "2026-03-26", summary : "Added `CLAUDE_CODE_MCP_SERVER_NAME` and `CLAUDE..." },
  VersionRecord { version : "2.1.84", date : "2026-03-26", summary : "Added PowerShell tool for Windows as an opt-in ..." },
  VersionRecord { version : "2.1.83", date : "2026-03-25", summary : "Added `managed-settings.d/` drop-in directory a..." },
  VersionRecord { version : "2.1.81", date : "2026-03-20", summary : "Added `--bare` flag for scripted `-p` calls — s..." },
  VersionRecord { version : "2.1.80", date : "2026-03-19", summary : "Added `rate_limits` field to statusline scripts..." },
  VersionRecord { version : "2.1.79", date : "2026-03-18", summary : "Added `--console` flag to `claude auth login` f..." },
  VersionRecord { version : "2.1.78", date : "2026-03-17", summary : "Added `StopFailure` hook event that fires when ..." },
  VersionRecord { version : "2.1.77", date : "2026-03-17", summary : "Increased default maximum output token limits f..." },
  VersionRecord { version : "2.1.76", date : "2026-03-14", summary : "Added MCP elicitation support — MCP servers can..." },
  VersionRecord { version : "2.1.75", date : "2026-03-13", summary : "Added 1M context window for Opus 4.6 by default..." },
  VersionRecord { version : "2.1.74", date : "2026-03-12", summary : "Added actionable suggestions to `/context` comm..." },
];

// ── Version detection ─────────────────────────────────────────────────────────

/// Extract the semver token (digits and dots) from a raw version string.
///
/// Strips an optional leading `v` or `V` prefix. Returns `raw` unchanged if
/// no semver-shaped token is found.
#[ inline ]
#[ must_use ]
pub fn extract_semver( raw : &str ) -> &str
{
  raw.split_whitespace()
  .find_map( | t |
  {
    let candidate = t.strip_prefix( 'v' )
    .or_else( || t.strip_prefix( 'V' ) )
    .unwrap_or( t );
    if !candidate.is_empty() && candidate.chars().all( | c | c.is_ascii_digit() || c == '.' )
    {
      Some( candidate )
    }
    else
    {
      None
    }
  } )
  .unwrap_or( raw )
}

/// Read the installed version from the `~/.local/bin/claude` symlink target.
///
/// Returns `None` if `HOME` is not set or the symlink does not exist.
#[ inline ]
#[ must_use ]
pub fn get_version_from_symlink() -> Option< String >
{
  std::env::var( "HOME" ).ok().filter( | h | !h.is_empty() )?;
  let link = binary_symlink_path();
  let target = std::fs::read_link( &link ).ok()?;
  let name = target.file_name()?.to_str()?;
  if !name.is_empty() && name.chars().all( | c | c.is_ascii_digit() || c == '.' )
  {
    Some( name.to_string() )
  }
  else
  {
    None
  }
}

/// Run `claude --version` and return its trimmed stdout.
///
/// Returns `None` if `claude` is not in PATH or the command fails.
#[ inline ]
#[ must_use ]
pub fn get_claude_version_raw() -> Option< String >
{
  let output = std::process::Command::new( "bash" )
  .args( [ "-c", "claude --version" ] )
  .env( "DISABLE_AUTOUPDATER", "1" )
  .output()
  .ok()?;
  let s = String::from_utf8_lossy( &output.stdout ).trim().to_string();
  if s.is_empty() { None } else { Some( s ) }
}

/// Get the installed Claude Code version (symlink-based detection preferred).
///
/// Returns `None` if no installed version can be detected.
#[ inline ]
#[ must_use ]
pub fn get_installed_version() -> Option< String >
{
  get_version_from_symlink()
  .or_else( ||
  {
    get_claude_version_raw().map( | raw | extract_semver( &raw ).to_string() )
  } )
}

// ── Alias resolution ──────────────────────────────────────────────────────────

/// Resolve a version spec to the value passed to the official installer.
///
/// Aliases map to their pinned semver or `"latest"`. Unknown specs are returned
/// unchanged (e.g. a raw `"1.2.3"` passes through as-is).
#[ inline ]
#[ must_use ]
pub fn resolve_version_spec( spec : &str ) -> &str
{
  VERSION_ALIASES.iter()
  .find( | a | a.name == spec )
  .map_or( spec, | a | if a.value.is_empty() { a.name } else { a.value } )
}

/// Validate a version spec: must be a known alias or a 3-part semver.
///
/// # Errors
///
/// Returns [`CoreError::ParseError`] for empty or unrecognised specs.
#[ inline ]
pub fn validate_version_spec( spec : &str ) -> Result< (), CoreError >
{
  if spec.is_empty()
  {
    return Err( CoreError::ParseError( "version:: value cannot be empty".to_string() ) );
  }

  if VERSION_ALIASES.iter().any( | a | a.name == spec )
  {
    return Ok( () );
  }

  // Semver: exactly 3 dot-separated numeric parts, no leading zeros.
  let parts : Vec< &str > = spec.split( '.' ).collect();
  if parts.len() == 3
  && parts.iter().all( | p |
  {
    !p.is_empty()
    && p.chars().all( | c | c.is_ascii_digit() )
    && ( p.len() == 1 || !p.starts_with( '0' ) )
  } )
  {
    return Ok( () );
  }

  Err( CoreError::ParseError( format!(
    "unknown version '{spec}': expected 'stable', 'latest', 'month', or semver like '1.2.3'"
  ) ) )
}

// ── Installation helpers ──────────────────────────────────────────────────────

/// Move the existing `claude` binary aside so a new install replaces it cleanly.
///
/// The binary is renamed to a `.preinstall` sidecar rather than deleted, so a
/// failed install can put it back. Returns the original binary path when a
/// swap-out occurred; the caller settles the sidecar's fate afterward via
/// `restore_swapped_binary()` (restore on failure, discard on success).
/// Renaming preserves the inode, so running sessions are exactly as unaffected
/// as they were by deletion (Unix open-file semantics).
///
/// Fix(BUG-016): rename aside instead of deleting outright.
/// Root cause: `remove_file` here was irreversible — when the installer later
/// refused to install (while still exiting 0), the launcher was already gone
/// and nothing restored it.
/// Pitfall: a destructive preparation step must stay reversible until the
/// outcome it prepares for is confirmed.
#[ inline ]
#[ must_use ]
pub fn hot_swap_binary() -> Option< String >
{
  eprintln!( "hot_swap_binary()" );
  let claude_path = std::process::Command::new( "which" )
  .arg( "claude" )
  .output()
  .ok()
  .filter( | o | o.status.success() )
  .map_or_else(
    binary_symlink_path,
    | o | String::from_utf8_lossy( &o.stdout ).trim().to_string(),
  );

  if std::path::Path::new( &claude_path ).exists()
  {
    let backup = format!( "{claude_path}.preinstall" );
    if std::fs::rename( &claude_path, &backup ).is_ok()
    {
      return Some( claude_path );
    }
    // Rename failed — fall back to plain removal so the installer can still
    // write the new binary; there is nothing recoverable to return.
    let _ = std::fs::remove_file( &claude_path );
  }
  None
}

/// Settle the fate of a binary moved aside by `hot_swap_binary()`.
///
/// If the install wrote a fresh binary at `original`, the `.preinstall`
/// sidecar is deleted; otherwise the sidecar is renamed back into place.
/// Uses `symlink_metadata` (not `exists`) so a dangling symlink — the normal
/// shape of the launcher whenever the versions directory changed underneath
/// it — still counts as present.
fn restore_swapped_binary( original : &str )
{
  let backup = format!( "{original}.preinstall" );
  if std::fs::symlink_metadata( &backup ).is_err()
  {
    return;
  }
  if std::fs::symlink_metadata( original ).is_ok()
  {
    let _ = std::fs::remove_file( &backup );
  }
  else
  {
    let _ = std::fs::rename( &backup, original );
  }
}

/// Return the path to the versions directory where Claude Code binaries live.
#[ inline ]
#[ must_use ]
pub fn versions_dir_path() -> String
{
  let home = std::env::var( "HOME" ).unwrap_or_default();
  format!( "{home}/.local/share/claude/versions" )
}

/// The current filesystem lock state of the versions directory, inferred
/// from its `chmod` mode.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum VersionsDirLockMode
{
  /// Mode `555` (read + execute only) — matches a pinned install.
  Locked,
  /// Mode `755` (read + write + execute) — matches an unpinned (`latest`) install.
  Unlocked,
  /// Directory exists but its mode is neither `555` nor `755` — a genuine
  /// permission anomaly, distinct from `Absent`.
  Unknown,
  /// Directory does not exist (nothing installed yet), or this platform
  /// cannot report POSIX mode bits — no reliable compliance signal either
  /// way, so callers must not treat this as a mismatch.
  Absent,
}

impl core::fmt::Display for VersionsDirLockMode
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.write_str( match self
    {
      Self::Locked   => "555",
      Self::Unlocked => "755",
      Self::Unknown  => "unknown",
      Self::Absent   => "absent",
    } )
  }
}

/// Read the current `chmod` mode of the versions directory.
///
/// Read-only — performs no mutation, so it is NOT one of the 10 traced
/// mutating functions (see `docs/pattern/002_parameter_trace.md`).
#[ inline ]
#[ must_use ]
#[ cfg( unix ) ]
// `core` has no `io` module (OS error codes are inherently std-only), and this
// whole function is already `std::fs`/`std::os::unix`-bound, so `std_instead_of_core`'s
// suggested `core::` path does not exist for the `ErrorKind` match guard below.
#[ allow( clippy::std_instead_of_core ) ]
pub fn read_versions_dir_lock_mode() -> VersionsDirLockMode
{
  use std::os::unix::fs::PermissionsExt;
  let dir = versions_dir_path();
  match std::fs::metadata( &dir )
  {
    Ok( meta ) => match meta.permissions().mode() & 0o777
    {
      0o555 => VersionsDirLockMode::Locked,
      0o755 => VersionsDirLockMode::Unlocked,
      _     => VersionsDirLockMode::Unknown,
    },
    // Only a genuinely missing directory is `Absent` (no install has happened
    // yet). Any other I/O error (e.g. permission denied on a parent directory)
    // is a real, investigation-worthy anomaly, not "nothing to see here" — it
    // falls into `Unknown` so it is flagged rather than silently swallowed.
    Err( e ) if e.kind() == std::io::ErrorKind::NotFound => VersionsDirLockMode::Absent,
    Err( _ ) => VersionsDirLockMode::Unknown,
  }
}

/// Read the current `chmod` mode of the versions directory.
///
/// Non-Unix fallback: file mode bits are not available, so this always
/// reports `Absent` — no reliable compliance signal either way.
#[ inline ]
#[ must_use ]
#[ cfg( not( unix ) ) ]
pub fn read_versions_dir_lock_mode() -> VersionsDirLockMode
{
  VersionsDirLockMode::Absent
}

/// Return the path to the `~/.local/bin/claude` hot-swap symlink.
#[ inline ]
#[ must_use ]
pub fn binary_symlink_path() -> String
{
  let home = std::env::var( "HOME" ).unwrap_or_default();
  format!( "{home}/.local/bin/claude" )
}

/// Return the path to the `~/.claude/.transient/version_history_cache.json` cache file.
#[ inline ]
#[ must_use ]
pub fn version_history_cache_path() -> String
{
  let home = std::env::var( "HOME" ).unwrap_or_default();
  format!( "{home}/.claude/.transient/version_history_cache.json" )
}

/// Purge all cached binaries from `versions_dir` except `keep`.
///
/// Best-effort: silently ignores all errors (consistent with `lock_version()`
/// and `unlock_versions_dir()`). Only deletes entries whose names consist
/// entirely of ASCII digits and dots — the version-string pattern (e.g. `2.1.78`).
/// This guard prevents accidental deletion of future lock/metadata files that
/// Claude's updater might add to the same directory.
///
/// Called from `perform_install()` before `lock_version()` for pinned installs.
/// The `versions_dir` parameter is explicit (not read from `HOME`) to allow
/// test isolation without `std::env::set_var`, which is not thread-safe.
///
/// No-op when `versions_dir/keep` itself does not exist: a purge that cannot
/// prove its keep target is actually present must not delete anything.
#[ inline ]
pub fn purge_stale_versions( versions_dir : &str, keep : &str )
{
  eprintln!( "purge_stale_versions(versions_dir={versions_dir:?}, keep={keep:?})" );
  // Fix(BUG-016): refuse to purge when the keep target is absent.
  // Root cause: with the keep file never written (installer refused but exited
  // 0), this loop deleted every cached version — including the only working
  // binary the running sessions were started from.
  // Pitfall: a cleanup that "keeps" something must first prove the kept thing
  // exists; otherwise "keep X" degrades silently into "delete everything".
  if !std::path::Path::new( versions_dir ).join( keep ).exists()
  {
    return;
  }
  let Ok( entries ) = std::fs::read_dir( versions_dir ) else { return; };
  for entry in entries.flatten()
  {
    let name      = entry.file_name();
    let name_str  = name.to_string_lossy();
    if name_str == keep { continue; }
    if !name_str.chars().all( | c | c.is_ascii_digit() || c == '.' ) { continue; }
    let _ = std::fs::remove_file( entry.path() );
  }
}

/// Unlock the versions directory so the installer can write new binaries.
#[ inline ]
pub fn unlock_versions_dir()
{
  eprintln!( "unlock_versions_dir()" );
  let dir = versions_dir_path();
  if std::path::Path::new( &dir ).exists()
  {
    let _ = std::process::Command::new( "chmod" )
    .args( [ "755", &dir ] )
    .status();
  }
}

/// Apply version lock (pinned) or unlock (latest) after a successful install.
///
/// Sets or removes 5 self-service bypass vectors in `~/.claude/settings.json`:
/// `autoUpdates`, `env.DISABLE_AUTOUPDATER`, `autoUpdatesChannel`,
/// `minimumVersion`, `env.DISABLE_UPDATES`. For pinned versions, also
/// `chmod 555` the versions directory to prevent silent auto-updates.
///
/// `resolved` is the resolved semver string written to `minimumVersion` for
/// pinned installs; ignored when `is_latest` is `true`.
#[ inline ]
pub fn lock_version( is_latest : bool, resolved : &str )
{
  eprintln!( "lock_version(is_latest={is_latest}, resolved={resolved:?})" );
  if let Some( paths ) = ClaudePaths::new()
  {
    let settings_file = paths.settings_file();
    if let Some( parent ) = settings_file.parent()
    {
      let _ = std::fs::create_dir_all( parent );
    }

    let auto_val = if is_latest { "true" } else { "false" };
    let _ = set_setting( &settings_file, "autoUpdates", auto_val );

    if is_latest
    {
      let _ = remove_env_var( &settings_file, "DISABLE_AUTOUPDATER" );
      let _ = remove_env_var( &settings_file, "DISABLE_UPDATES" );
      let _ = remove_setting( &settings_file, "autoUpdatesChannel" );
      let _ = remove_setting( &settings_file, "minimumVersion" );
    }
    else
    {
      let _ = set_env_var( &settings_file, "DISABLE_AUTOUPDATER", "1" );
      let _ = set_env_var( &settings_file, "DISABLE_UPDATES", "1" );
      let _ = set_setting( &settings_file, "autoUpdatesChannel", "stable" );
      let _ = set_setting( &settings_file, "minimumVersion", resolved );
    }
  }

  let dir = versions_dir_path();
  if std::path::Path::new( &dir ).exists()
  {
    let mode = if is_latest { "755" } else { "555" };
    let _ = std::process::Command::new( "chmod" )
    .args( [ mode, &dir ] )
    .status();
  }
}

/// Lift the settings-level update locks so the official installer can run.
///
/// The installer bootstrap honors update-disabling keys from Claude's own
/// `settings.json` (`env.DISABLE_AUTOUPDATER`, `env.DISABLE_UPDATES`,
/// `autoUpdates`, `minimumVersion`). Left in place from a PREVIOUS pinned
/// install, they make it refuse with "Updates are disabled by your
/// administrator" while still exiting 0. `lock_version()` re-applies the lock
/// after the outcome is verified; on failure the lock stays lifted, which is
/// the truthful state (`.version.guard` / `.status` then report drift).
///
/// Public so the unlock key set is directly testable alongside `lock_version()`.
/// Fix(BUG-017): an untestable private function cannot express the invariant
/// that unlock keys must mirror lock keys — any drift silently re-triggers BUG-016.
#[ inline ]
pub fn unlock_settings_for_install()
{
  eprintln!( "unlock_settings_for_install()" );
  if let Some( paths ) = ClaudePaths::new()
  {
    let settings_file = paths.settings_file();
    let _ = set_setting( &settings_file, "autoUpdates", "true" );
    let _ = remove_env_var( &settings_file, "DISABLE_AUTOUPDATER" );
    let _ = remove_env_var( &settings_file, "DISABLE_UPDATES" );
    let _ = remove_setting( &settings_file, "minimumVersion" );
  }
}

/// Decide whether an installer run actually produced the requested outcome.
///
/// Pure decision function: `installed` is the version detected AFTER the
/// installer ran (`None` when no binary is detectable at all). The official
/// bootstrap can refuse to install while still exiting 0, so the exit code
/// alone is never evidence of success — this check is what gates the
/// destructive follow-ups (purge, lock) in `perform_install()`.
///
/// For pinned installs the detected version must equal `resolved` exactly.
/// For `latest` any detectable version passes — the installer chooses the
/// concrete semver, so a silent refusal that leaves a pre-existing binary in
/// place is indistinguishable here; that is acceptable because the `latest`
/// path never purges and so cannot destroy anything on a false pass.
#[ inline ]
#[ must_use ]
pub fn verify_install_outcome( resolved : &str, is_latest : bool, installed : Option< &str > ) -> bool
{
  installed.is_some_and( | v | is_latest || v == resolved )
}

/// Execute the install sequence: settings-unlock → hot-swap → dir-unlock →
/// curl → verify → purge → lock.
///
/// The installer's exit code alone is NOT trusted: the official bootstrap can
/// refuse to install (e.g. "Updates are disabled by your administrator", from
/// update-disabling keys in `settings.json`) while still exiting 0. The
/// outcome is confirmed via `verify_install_outcome()` before any destructive
/// follow-up (purge, lock) runs; on any failure the hot-swapped launcher is
/// restored and the settings-level lock is left lifted so `.version.guard` /
/// `.status` report the drift truthfully instead of re-asserting a lock over
/// a version that was never installed.
///
/// For pinned versions (`!is_latest`), `purge_stale_versions` runs after the
/// verified install and BEFORE `lock_version` (which applies chmod 555).
/// Purging after chmod 555 would silently fail. Purge is skipped for `latest`
/// so the cached version history remains available for rollback.
///
/// `resolved` is the semver string or `"latest"`. `is_latest` controls
/// whether auto-updates are enabled and the versions dir is left unlocked.
///
/// # Errors
///
/// Returns [`CoreError::ProcessError`] if the installer script fails to run or
/// exits non-zero, or if it exits 0 without the requested version actually
/// being installed.
#[ inline ]
pub fn perform_install( resolved : &str, is_latest : bool ) -> Result< (), CoreError >
{
  eprintln!( "perform_install(resolved={resolved:?}, is_latest={is_latest})" );

  // Fix(BUG-016): lift the settings-level update locks before invoking the
  // installer, not only the versions-dir chmod.
  // Root cause: env.DISABLE_AUTOUPDATER / env.DISABLE_UPDATES / autoUpdates /
  // minimumVersion persisted by the PREVIOUS pinned install made the official
  // bootstrap refuse ("Updates are disabled by your administrator") while
  // still exiting 0 — the lock blocked its own re-install path.
  // Pitfall: every lock layer that can block the installer must be lifted
  // pre-install and re-applied only after a verified outcome.
  unlock_settings_for_install();

  let swapped = if find_claude_processes().is_empty() { None } else { hot_swap_binary() };

  unlock_versions_dir();

  let shell_cmd = if is_latest
  {
    format!( "curl -fsSL {INSTALL_URL} | bash" )
  }
  else
  {
    format!( "curl -fsSL {INSTALL_URL} | bash -s -- {resolved}" )
  };

  // Fix(BUG-016): strip inherited update-disabling env vars instead of
  // injecting DISABLE_AUTOUPDATER=1 into the installer.
  // Root cause: the injected flag (meant to stop the bootstrap self-updating
  // mid-install) is one of the flags that make it refuse the requested install
  // outright — and both flags also arrive inherited when run from a Claude
  // session shell.
  // Pitfall: a flag that suppresses a tool's self-update can suppress the very
  // operation being requested; post-install `lock_version()` already provides
  // the durable update block.
  let status = std::process::Command::new( "bash" )
  .args( [ "-c", &shell_cmd ] )
  .env_remove( "DISABLE_AUTOUPDATER" )
  .env_remove( "DISABLE_UPDATES" )
  .status();

  let status = match status
  {
    Ok( s ) => s,
    Err( e ) =>
    {
      if let Some( original ) = &swapped { restore_swapped_binary( original ); }
      return Err( CoreError::ProcessError( format!( "failed to run installer: {e}" ) ) );
    }
  };

  // Fix(BUG-016): verify the outcome — the exit code alone is not evidence.
  // Root cause: `status.success()` was the sole gate before purge/lock; the
  // bootstrap exits 0 even when it refuses to install, so the purge ran with
  // its keep target never written and deleted every cached binary.
  // Pitfall: never let destructive cleanup key off a subprocess exit code when
  // that subprocess is known to exit 0 on refusal.
  let installed = get_installed_version();
  let ok = status.success() && verify_install_outcome( resolved, is_latest, installed.as_deref() );

  // Settle the hot-swap sidecar either way: restore the old launcher on
  // failure, discard the sidecar after a verified success.
  if let Some( original ) = &swapped { restore_swapped_binary( original ); }

  if !ok
  {
    let msg = if status.success()
    {
      match installed.as_deref()
      {
        None => format!( "install failed: installer exited 0 but did not install the requested {resolved} (no claude binary detectable) — check for update-disabling settings or environment" ),
        Some( v ) => format!( "install failed: installer exited 0 but installed version is {v}, not the requested {resolved}" ),
      }
    }
    else
    {
      "install failed".to_string()
    };
    return Err( CoreError::ProcessError( msg ) );
  }

  if !is_latest
  {
    purge_stale_versions( &versions_dir_path(), resolved );
  }
  lock_version( is_latest, resolved );
  Ok( () )
}

// ── Preference persistence ─────────────────────────────────────────────────────

/// Read the user's preferred version from `~/.claude/settings.json`.
///
/// Returns `None` if `HOME` is unset, the settings file is absent, or no
/// preference has been stored yet.
#[ inline ]
#[ must_use ]
pub fn read_preferred_version() -> Option< ( String, Option< String > ) >
{
  let paths = ClaudePaths::new()?;
  let settings_file = paths.settings_file();
  let spec = get_setting( &settings_file, "preferredVersionSpec" )
    .ok()?
    .filter( | s | !s.is_empty() )?;
  let resolved = get_setting( &settings_file, "preferredVersionResolved" )
    .ok()
    .flatten()
    .filter( | v | v != "null" && !v.is_empty() );
  Some( ( spec, resolved ) )
}

/// Persist the user's preferred version in `~/.claude/settings.json`.
///
/// Both `preferredVersionSpec` and `preferredVersionResolved` are written.
/// For the `latest` alias, `resolved` is stored as `"null"`.
///
/// # Errors
///
/// Returns [`CoreError`] if `HOME` is unset or the settings file cannot be written.
#[ inline ]
pub fn store_preferred_version( spec : &str, resolved : &str, is_latest : bool ) -> Result< (), CoreError >
{
  eprintln!( "store_preferred_version(spec={spec:?}, resolved={resolved:?}, is_latest={is_latest})" );
  let paths = ClaudePaths::new().ok_or_else( ||
    CoreError::ProcessError( "HOME environment variable not set".to_string() )
  )?;
  let settings_file = paths.settings_file();
  if let Some( parent ) = settings_file.parent()
  {
    let _ = std::fs::create_dir_all( parent );
  }
  set_setting( &settings_file, "preferredVersionSpec", spec )
    .map_err( CoreError::IoError )?;
  let resolved_val = if is_latest { "null" } else { resolved };
  set_setting( &settings_file, "preferredVersionResolved", resolved_val )
    .map_err( CoreError::IoError )?;
  Ok( () )
}
