#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! Claude Code CLI Launcher
//!
//! General-purpose library for executing Claude Code CLI commands
//! programmatically with full parameter control.
//!
//! # Quick Start
//!
//! ```no_run
//! use claude_runner_core::ClaudeCommand;
//!
//! let cmd = ClaudeCommand::new()
//!   .with_working_directory( "/tmp/work" )
//!   .with_max_output_tokens( 200_000 );
//!
//! let output = cmd.execute()?;
//! println!( "{}", output.stdout );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Architecture
//!
//! - **Single execution point:** All commands go through `execute()`
//! - **Builder pattern:** Configuration via chainable `with_*()` methods
//! - **Private fields:** Can't construct with struct literals
//! - **No session logic:** Pure execution, no state management
//!
//! # Migration: Old Way Impossible
//!
//! This crate enforces the new builder pattern at compile time. Old patterns
//! from the deprecated API are impossible:
//!
//! ## Old Pattern 1: Factory Method (Doesn't Exist)
//!
//! ```compile_fail
//! use claude_runner_core::ClaudeCommand;
//!
//! // ERROR: generate() method doesn't exist
//! let cmd = ClaudeCommand::generate("/tmp", "msg", 1000, Strategy::Fresh);
//! ```
//!
//! ## Old Pattern 2: Direct Construction (Fields Private)
//!
//! ```compile_fail
//! use claude_runner_core::ClaudeCommand;
//! use std::path::PathBuf;
//!
//! // ERROR: fields are private
//! let cmd = ClaudeCommand {
//!   working_directory: Some(PathBuf::from("/tmp")),
//!   max_output_tokens: Some(200_000),
//!   continue_conversation: false,
//!   message: None,
//!   args: vec![],
//! };
//! ```
//!
//! ## New Pattern: Builder Only
//!
//! ```no_run
//! use claude_runner_core::ClaudeCommand;
//!
//! // the ONLY way to construct ClaudeCommand
//! let output = ClaudeCommand::new()
//!   .with_working_directory( "/tmp" )
//!   .with_max_output_tokens( 200_000 )
//!   .execute()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! See [spec.md](../spec.md) for complete documentation.

mod command;
mod control;
mod exit_code;
mod isolated;
mod types;
pub mod process;
pub mod session_dir;

pub use crate::command::{ ClaudeCommand, claude_version, DEFAULT_COMPACT_WINDOW };
pub use crate::control::ControlSession;
pub use crate::exit_code::signal_exit_code;
pub use crate::isolated::{ IsolatedModel, IsolatedRunResult, RunnerError, ISOLATED_DEFAULT_MODEL, ISOLATED_CLAUDE_MD, REFRESH_DEFAULT_MODEL };
pub use crate::types::{
  AccountInfo, ActionMode, ContextUsageResult, EffortLevel, ErrorKind, ExecutionOutput,
  InitializeResult, InputFormat, LogLevel, McpPermissionOverrideMode, McpServerStatusEntry,
  OutputFormat, PermissionMode, ReadFileEncoding, ReadFileResult, ReloadPluginsResult,
  ReloadSkillsResult, RewindFilesResult, SetMcpPermissionModeOverrideResult, SetMcpServersResult,
  ThinkingDisplay,
};
pub use crate::session_dir::{ SessionManager, Strategy };

/// Re-export of [`crate::isolated::run_isolated`].
///
/// # Warning
///
/// Do NOT call `run_isolated()` directly for credential refresh.
/// Use `claude_profile_core::account::refresh_account_token()` instead.
/// See invariant 008: `claude_profile/docs/invariant/008_single_token_refresh_entry.md`.
#[ cfg( feature = "enabled" ) ]
pub use crate::isolated::run_isolated;

/// Re-export of [`crate::isolated::run_isolated_ext`].
///
/// Extended variant that accepts `compact_window: Option<u32>` — use when you need to
/// suppress `CLAUDE_CODE_AUTO_COMPACT_WINDOW` (`None`) or override the 300K default.
///
/// # Warning
///
/// Do NOT call this directly for credential refresh. See [`run_isolated`] warning.
#[ cfg( feature = "enabled" ) ]
pub use crate::isolated::run_isolated_ext;

/// Re-export of [`crate::isolated::resolve_isolated_default_model`].
///
/// Resolves `IsolatedModel::Default`'s model preference across both tiers:
/// project `.clr.toml` → user `~/.clr/config.toml`.
/// Returns `None` if nothing is set at either tier. Used by `run_isolated_ext()`'s
/// `IsolatedModel::Default` match arm.
#[ cfg( feature = "enabled" ) ]
pub use crate::isolated::resolve_isolated_default_model;
