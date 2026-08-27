#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! # `claude_terminal_core` — terminal output, made readable
//!
//! Interprets a terminal's raw byte stream as the text a person would have read.
//!
//! This is the counterpart to `claude_pty_core`, and the split is deliberate:
//! that crate owns the *device* — allocating a pseudo-terminal and putting a
//! child process on it — while this one owns the *protocol spoken over it*.
//! Interpreting escape sequences needs no pty, and a caller holding captured
//! bytes should not link POSIX FFI to read them.
//!
//! Nothing here knows about Claude Code, daemons, or sessions. It is a scanner
//! over a `&str`.
//!
//! ## Core surface
//!
//! - [`to_plain_text`] — render a raw stream as plain text
//! - [`MAX_ESCAPE_PARAM_CHARS`] — the cap that bounds a desynchronised stream
//!
//! ## A line renderer, not an emulator
//!
//! Exactly one thing is modelled: a cursor moving within the current line. That
//! covers how a command-line program rewrites what it has already printed —
//! `\r`, `ESC [ K`, `\b` — and covers it exactly. Cursor addressing, scroll
//! regions and alternate screens are recognised well enough to be *removed*,
//! never obeyed. See `docs/invariant/002_line_renderer_boundary.md` for why that
//! boundary is a guarantee rather than an unfinished emulator.
//!
//! **Zero dependencies**: the whole crate is one scanner over `&str`. A terminal
//! emulator crate would bring the screen model this crate exists not to have.

#![ deny( missing_docs ) ]
#![ warn( rust_2018_idioms ) ]

pub mod render;

pub use render::{ to_plain_text, MAX_ESCAPE_PARAM_CHARS };
