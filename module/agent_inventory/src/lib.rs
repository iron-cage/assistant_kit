#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! `agent_inventory` — agent-agnostic asset discovery via the `AgentAdapter` trait.
//!
//! Layer 1 domain crate (no CLI framework dependencies). Adapters are feature-gated:
//! only activated adapters add compile-time cost.

pub mod adapter;
pub mod entry;
pub mod error;
pub mod inventory;

#[ cfg( feature = "claude_code" ) ]
pub mod claude_code;
