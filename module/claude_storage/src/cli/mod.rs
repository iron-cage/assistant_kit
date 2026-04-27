//! CLI command routines for `claude_storage`
//!
//! This module provides command routines that are registered with unilang
//! and called when users invoke CLI commands.
//!
//! ## Known Pitfalls
//!
//! ### Parameter Validation Consistency (Finding #010)
//!
//! **Issue**: Default parameter values do not prevent invalid input - parameters
//! with defaults still require explicit validation.
//!
//! **Context**: `search_routine` was missing verbosity range validation (0-5)
//! while `status_routine` and `show_routine` had it. The default value (1) made
//! it seem like validation was unnecessary, but users can override defaults
//! with invalid values like -1 or 10.
//!
//! **Solution**: All parameters with value constraints must have explicit
//! validation, regardless of default values. Apply validation patterns
//! consistently across all command routines:
//!
//! ```rust,no_run
//! # use unilang::{ VerifiedCommand, ErrorData, ErrorCode };
//! # const VERBOSITY_MAX : i64 = 5;
//! # fn example( cmd : VerifiedCommand ) -> Result< (), ErrorData >
//! # {
//! let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );
//!
//! // Always validate range, even with defaults
//! if !( 0..=VERBOSITY_MAX ).contains( &verbosity )
//! {
//!   return Err( ErrorData::new(
//!     ErrorCode::InternalError,
//!     format!( "Invalid verbosity: {}. Valid range: 0-5", verbosity )
//!   ));
//! }
//! # Ok( () )
//! # }
//! ```
//!
//! **Prevention**: When adding new parameters, check existing command routines
//! for validation patterns and apply them consistently. Never assume defaults
//! eliminate the need for validation.
//!
//! See: `tests/search_command_test.rs::test_search_verbosity_invalid`

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
