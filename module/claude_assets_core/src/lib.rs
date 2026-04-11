#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! Layer 1 domain logic for Claude Code artifact installation.
//!
//! Provides symlink-based install/uninstall of rules, commands, agents, skills,
//! plugins, and hooks from a central source (`$PRO_CLAUDE`) into project-local
//! `.claude/<kind>/` directories.
//!
//! Zero CLI framework dependencies — no `unilang`, no `clap`.

#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]

pub mod artifact;
pub mod error;
pub mod install;
pub mod paths;
pub mod registry;
