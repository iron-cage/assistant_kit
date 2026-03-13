//! Bug reproducer: `truncate_if_needed` panics on multibyte UTF-8 (issue-018)
//!
//! ## Root Cause
//!
//! `truncate_if_needed` uses `&text[..len]` which slices by byte offset.
//! When `len` falls inside a multibyte UTF-8 sequence (e.g., a 4-byte emoji),
//! Rust panics with "byte index N is not a char boundary".
//!
//! ## Why Not Caught
//!
//! `truncate_if_needed` was private with no unit tests.  All integration
//! tests used ASCII-only session content, so the panic path was never reached.
//!
//! ## Fix Applied
//!
//! Replace `&text[..len]` with a char-boundary-safe truncation that finds the
//! nearest valid boundary at or before `len`.
//!
//! ## Prevention
//!
//! Never use byte-offset slicing (`&s[..n]`) on arbitrary user text.  Always
//! use `str::floor_char_boundary` (nightly) or the `is_char_boundary` loop.
//!
//! ## Pitfall
//!
//! `str::len()` returns bytes, not characters.  Using it as a slice limit on
//! text that may contain emoji, CJK, or accented characters will panic.

/// Before fix, this would panic with "byte index 7 is not a char boundary".
/// After fix, it must truncate gracefully to the nearest char boundary.
#[test]
fn tc001_truncate_mid_emoji_graceful()
{
  let text = "Hello \u{1F30D} world";
  // Byte 7 is inside the 4-byte emoji (bytes 6-9).
  // Fix snaps back to byte 6 (boundary before emoji).
  let result = claude_storage::cli::truncate_if_needed( text, Some( 7 ) );
  assert!( result.starts_with( "Hello " ), "must snap to char boundary: {result}" );
  assert!( result.contains( "truncated" ), "must contain truncation marker: {result}" );
}

/// After fix, truncation at a mid-character byte must NOT panic and must
/// produce a valid UTF-8 string ending at the nearest char boundary.
#[test]
fn tc002_truncate_mid_emoji_no_panic()
{
  let text = "Hello \u{1F30D} world";
  let result = claude_storage::cli::truncate_if_needed( text, Some( 7 ) );
  // Must be valid UTF-8 (compilation guarantees this for String)
  // Must contain the truncation marker
  assert!( result.contains( "truncated" ), "must contain truncation marker: {result}" );
  // Must NOT contain partial bytes — Rust String guarantees this
  // The truncated prefix should be "Hello " (6 bytes — boundary just before emoji)
  assert!( result.starts_with( "Hello " ), "must start with 'Hello ': {result}" );
}

/// Truncation exactly at a char boundary should work cleanly.
#[test]
fn tc003_truncate_at_char_boundary_works()
{
  let text = "Hello \u{1F30D} world";
  // Byte 6 is the start of the emoji — valid boundary.
  let result = claude_storage::cli::truncate_if_needed( text, Some( 6 ) );
  assert!( result.starts_with( "Hello " ), "must keep up to boundary: {result}" );
  assert!( result.contains( "truncated" ), "must truncate: {result}" );
}

/// Truncation with None should return the full string.
#[test]
fn tc004_truncate_none_returns_full()
{
  let text = "Hello \u{1F30D} world";
  let result = claude_storage::cli::truncate_if_needed( text, None );
  assert_eq!( result, text );
}

/// Truncation with limit >= length should return the full string.
#[test]
fn tc005_truncate_at_or_beyond_length()
{
  let text = "Hello \u{1F30D} world";
  let result = claude_storage::cli::truncate_if_needed( text, Some( text.len() ) );
  assert_eq!( result, text );
  let result2 = claude_storage::cli::truncate_if_needed( text, Some( text.len() + 100 ) );
  assert_eq!( result2, text );
}

/// CJK characters are 3 bytes each — truncation mid-CJK must not panic.
#[test]
fn tc006_truncate_mid_cjk_no_panic()
{
  // Each CJK char is 3 bytes: 你=3, 好=3, 世=3, 界=3 → 12 bytes total
  let text = "\u{4F60}\u{597D}\u{4E16}\u{754C}"; // 你好世界
  let result = claude_storage::cli::truncate_if_needed( text, Some( 4 ) );
  // Byte 4 is mid-second character — must snap back to byte 3 (end of first char)
  assert!( result.contains( "truncated" ), "must truncate: {result}" );
}

/// Truncation at byte 0 should produce empty prefix with truncation marker.
#[test]
fn tc007_truncate_at_zero()
{
  let text = "Hello \u{1F30D} world";
  let result = claude_storage::cli::truncate_if_needed( text, Some( 0 ) );
  assert!( result.contains( "truncated" ), "must truncate: {result}" );
  assert!( result.starts_with( "..." ) || result.is_empty() || result.contains( "truncated" ) );
}
