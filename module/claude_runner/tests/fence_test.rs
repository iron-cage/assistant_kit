//! Unit tests for `strip_fences` — the `--strip-fences` output post-processor.
//!
//! Tests the outermost-fence-pair stripping logic in isolation.
//! Source function: `claude_runner::strip_fences`.
#![ cfg( feature = "enabled" ) ]

use claude_runner::strip_fences;

#[ test ]
fn sf01_basic_fence_pair_stripped() { assert_eq!( strip_fences( "```\nhello\n```\n" ), "hello\n" ); }
#[ test ]
fn sf02_language_tagged_fence_stripped() { assert_eq!( strip_fences( "```rust\nfn f(){}\n```\n" ), "fn f(){}\n" ); }
#[ test ]
fn sf03_no_fences_pass_through() { assert_eq!( strip_fences( "plain text\n" ), "plain text\n" ); }
#[ test ]
fn sf04_single_fence_unchanged() { assert_eq!( strip_fences( "```\n" ), "```\n" ); }
#[ test ]
fn sf05_empty_string_unchanged() { assert_eq!( strip_fences( "" ), "" ); }
#[ test ]
fn sf06_inner_fences_preserved() { assert_eq!( strip_fences( "```\n```inner\n```\n```\n" ), "```inner\n```\n" ); }
#[ test ]
fn sf07_no_trailing_newline_preserved() { assert_eq!( strip_fences( "```\ncontent\n```" ), "content" ); }
#[ test ]
fn sf08_trailing_newline_preserved() { assert_eq!( strip_fences( "```\ncontent\n```\n" ), "content\n" ); }
