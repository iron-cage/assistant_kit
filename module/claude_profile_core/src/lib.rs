#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! Layer 1 domain logic for Claude Code account and token management.
//!
//! Depends only on [`claude_common`] — zero CLI framework dependencies.
//!
//! # Modules
//!
//! - [`token`]: OAuth token expiry status detection
//! - [`account`]: Named credential storage and account rotation

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]

pub mod token;
pub mod account;
