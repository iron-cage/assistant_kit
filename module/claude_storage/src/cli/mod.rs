//! CLI command routines for `claude_storage`
//!
//! This module provides command routines that are registered with unilang
//! and called when users invoke CLI commands.
//!
//! ## Known Pitfalls
//!
//! ### Parameter Validation Consistency (Finding #010)
//!
//! **Issue**: Default parameter values do not prevent invalid input — parameters
//! with defaults still require explicit validation.
//!
//! **Solution**: All parameters with value constraints must have explicit
//! validation regardless of default values. Apply validation patterns
//! consistently across all command routines. Boolean parameters accept only
//! `0`/`1`; integer parameters (e.g. `min_entries::`, `limit::`) must be
//! validated as non-negative at point of use.
//!
//! **Prevention**: When adding new parameters, check existing command routines
//! for validation patterns and apply them consistently.

mod storage;
mod format;
mod status;
mod list;
mod show;
mod count;
mod search;
mod export;
mod projects;
mod session;
mod tail;

pub use storage::parse_project_parameter;
pub use format::truncate_if_needed;
pub use projects::{ Conversation, projects_routine };
pub use status::status_routine;
pub use list::list_routine;
pub use show::show_routine;
pub use count::count_routine;
pub use search::search_routine;
pub use export::export_routine;
pub use session::{ project_path_routine, project_exists_routine, session_dir_routine, session_ensure_routine };
pub use tail::tail_routine;
