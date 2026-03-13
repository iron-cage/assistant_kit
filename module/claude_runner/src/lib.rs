//! `claude_runner` crate / `clr` binary — Claude Code CLI + command schema constants.
//!
//! This crate has two roles:
//!
//! 1. **Library** — exports [`COMMANDS_YAML`], the path to the `.claude` command schema,
//!    for use by YAML consumers at compile time or runtime.
//!
//! 2. **Binary** (`clr`) — Standalone CLI that mirrors Claude Code's native
//!    `--flag value` syntax and executes via `claude_runner_core`.
//!    Session continuation (`-c`) is applied by default; use `--new-session` to start fresh.
//!
//! ## Two roles, two consumers
//!
//! ```text
//! clr binary (standalone CLI)
//!   invoked directly: clr "Fix bug" --dir /path --model sonnet
//!     → parse_args() → ClaudeCommand builder → claude subprocess (with -c by default)
//!   message given → print mode (default); bare clr → interactive REPL
//!
//! YAML consumers (e.g. willbe's dream CLI, build.rs)
//!   aggregate: claude_runner::COMMANDS_YAML → registers .claude + .claude.help in PHF map
//! ```
//!
//! This lib has **zero willbe dependencies**. Without `enabled`, it is a pure constants +
//! types crate. With `enabled`, it also exposes [`register_commands`] (API consistency shim).
//!
//! ## Registering commands in other binaries
//!
//! **Build-time (PHF):** Point `build.rs` at [`COMMANDS_YAML`].
//!
//! **Runtime:** Use `MultiYamlAggregator` with [`COMMANDS_YAML`].

pub mod verbosity;
pub use verbosity::VerbosityLevel;

/// Absolute path to this crate's command definitions YAML.
///
/// Use in `build.rs` for compile-time aggregation or at runtime for dynamic registration.
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/claude.commands.yaml" );

#[ cfg( feature = "enabled" ) ]
/// Register `claude_runner` commands into an existing registry.
///
/// `claude_runner` commands are defined in [`COMMANDS_YAML`] for compile-time aggregation
/// (used by `claude_tools/build.rs`). This function is provided for API consistency with
/// other Layer 2 crates; the body is intentionally empty because runtime registration of
/// `.claude` commands is handled by the build-time YAML aggregation path in `claude_tools`.
#[ inline ]
pub fn register_commands( _registry : &mut unilang::registry::CommandRegistry ) {}
