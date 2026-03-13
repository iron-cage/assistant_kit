//! `claude_tools` — Layer 3 super-app aggregating all `claude_*` CLI commands.
//!
//! Aggregates commands from all Layer 2 crates:
//! - [`claude_manager`]: version management, settings, session, account commands
//! - [`claude_runner`]: `.claude` AI-assistance command (YAML-based)
//! - [`claude_storage`]: storage exploration commands (YAML-based)
//!
//! # Feature Gate
//!
//! All modules require the `enabled` feature. Without it the crate compiles to an empty
//! shell, which is the intended behaviour for library crates in this workspace.

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]
