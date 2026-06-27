//! `claude_journal` — append-only JSONL event journal for CLR.
//!
//! # Overview
//!
//! Events are written one per line to daily `YYYY-MM-DD.jsonl` files under a
//! configurable directory (default `~/.clr/journal/`).  Each line is a
//! self-contained JSON object that can be parsed independently, making the
//! journal crash-safe: a partial write on shutdown leaves all prior lines intact.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use claude_journal::{ JournalWriter, EventRecord, EventType };
//! use std::path::PathBuf;
//!
//! let writer = JournalWriter::new( PathBuf::from( "/tmp/my-journal" ) );
//! let mut ev = EventRecord::new( EventType::Execution );
//! ev.fields.exit_code = Some( 0 );
//! writer.append( &ev ).expect( "write failed" );
//! ```
//!
//! ## Reading / querying
//!
//! ```rust,no_run
//! use claude_journal::{ JournalReader, JournalFilter };
//! use std::path::PathBuf;
//!
//! let reader = JournalReader::open( PathBuf::from( "/tmp/my-journal" ) );
//! let filter = JournalFilter::default();
//! let events = reader.query( &filter );
//! println!( "{} events found", events.len() );
//! ```

#![ doc( html_root_url = "https://docs.rs/claude_journal/0.1.0" ) ]
#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]

pub mod event;
pub mod rotation;
pub mod reader;
pub mod writer;

pub use event::{ EventFields, EventRecord, EventType };
pub use reader::{ JournalFilter, JournalReader };
pub use writer::JournalWriter;
