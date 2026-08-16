//! Unit tests: OAuth account endpoint response parsing (`parse_oauth_account`).
//!
//! All tests are offline — no network, no `ureq` in dev-dependencies.
//! `parse_oauth_account` and `select_membership_index` are always available
//! (no `enabled` feature).
//!
//! ## Test Matrix
//!
//! | ID     | Scenario                                                       | Expected                                        | Status |
//! |--------|----------------------------------------------------------------|-------------------------------------------------|--------|
//! | MRE-237a | Multi-membership: none at [0], stripe+max at [1]             | membership[1] selected; identity from top level | ✅     |
//! | MRE-237b | Multi-membership: none at [0], stripe (no max) at [1]        | membership[1] selected; `has_max == false`      | ✅     |
//! | MRE-237c | Single membership, billing `none`                            | index 0 selected via Priority 3 fallback        | ✅     |
//! | AT-01  | Whitespace before every colon (`"key" : value`)                | body parses; all fields land                    | ✅     |
//! | AT-02  | Identity keys serialized AFTER the memberships array           | user-level `uuid`, not the org's, is returned   | ✅     |
//! | AT-03  | `"claude_max"` as a string *value* only (not a capability)     | `has_max == false`                              | ✅     |
//! | AT-04  | A field *value* textually equal to an identity key name        | scanner skips the value, finds the real key     | ✅     |

use claude_quota::parse_oauth_account;

// ── BUG-237 MREs (moved from src/lib.rs in-src tests — audit minor) ───────────

#[ test ]
#[ doc = "`bug_reproducer(237)`" ]
/// `parse_oauth_account` selects the stripe+max membership over a none-billing entry.
///
/// # Root Cause
/// `str::find("\"organization\":")` always resolves to `memberships[0]`'s organization
/// block. Accounts with a paid subscription at index > 0 were silently misclassified as
/// having no subscription.
///
/// # Why Not Caught
/// All test fixtures used single-membership bodies. Multi-membership accounts require
/// separate Anthropic org entities — uncommon in CI fixtures.
///
/// # Fix Applied
/// `parse_oauth_account` now calls `parse_membership_list` which iterates ALL membership
/// objects using brace-balanced scanning, then `select_membership_index` picks the
/// highest-priority entry.
///
/// # Prevention
/// This test must FAIL before the fix (memberships[0] is "none") and PASS after.
///
/// # Pitfall
/// Always use brace-balanced extraction when iterating JSON arrays containing nested
/// objects — `str::find` on a needle will collide with nested occurrences of the same key.
fn mre_bug237_multi_membership_selects_stripe_max_over_none()
{
  let body = r#"{
    "tagged_id": "user_01ABC",
    "uuid": "aaaa-bbbb",
    "email_address": "alice@acme.com",
    "full_name": "Alice",
    "display_name": "Alice",
    "memberships": [
      { "role": "member", "organization": { "billing_type": "none", "capabilities": ["chat"], "created_at": "2024-01-01T00:00:00Z" } },
      { "role": "admin",  "organization": { "billing_type": "stripe_subscription", "capabilities": ["claude_max","chat"], "rate_limit_tier": "default_claude_max_20x", "created_at": "2024-02-01T00:00:00Z" } }
    ]
  }"#;
  let result = parse_oauth_account( body ).expect( "should parse" );
  assert_eq!( result.billing_type, "stripe_subscription", "must select membership[1] (stripe+max), not membership[0] (none)" );
  assert!( result.has_max, "membership[1] has claude_max capability" );
  assert_eq!( result.org_created_at, "2024-02-01T00:00:00Z" );
  // BUG-295: identity fields from body top-level
  assert_eq!( result.tagged_id, "user_01ABC" );
  assert_eq!( result.uuid, "aaaa-bbbb" );
  assert_eq!( result.email_address, "alice@acme.com" );
  assert_eq!( result.full_name, "Alice" );
  assert_eq!( result.display_name, "Alice" );
  assert_eq!( result.capabilities, vec![ "claude_max", "chat" ] );
  assert_eq!( result.rate_limit_tier, "default_claude_max_20x" );
  assert_eq!( result.memberships.len(), 2, "all memberships preserved" );
}

#[ test ]
#[ doc = "`bug_reproducer(237)`" ]
/// `parse_oauth_account` selects stripe (no max) over none when no max tier is present.
fn mre_bug237_multi_membership_selects_stripe_over_none_no_max()
{
  let body = r#"{
    "tagged_id": "user_02XYZ",
    "uuid": "cccc-dddd",
    "email_address": "bob@example.com",
    "memberships": [
      { "role": "member", "organization": { "billing_type": "none", "capabilities": ["chat"], "created_at": "2024-01-01T00:00:00Z" } },
      { "role": "admin",  "organization": { "billing_type": "stripe_subscription", "capabilities": ["chat"], "created_at": "2024-03-01T00:00:00Z" } }
    ]
  }"#;
  let result = parse_oauth_account( body ).expect( "should parse" );
  assert_eq!( result.billing_type, "stripe_subscription" );
  assert!( !result.has_max, "no claude_max in membership[1]" );
  assert_eq!( result.org_created_at, "2024-03-01T00:00:00Z" );
  assert_eq!( result.tagged_id, "user_02XYZ" );
  assert_eq!( result.email_address, "bob@example.com" );
  assert!( result.rate_limit_tier.is_empty(), "no rate_limit_tier in fixture" );
}

#[ test ]
#[ doc = "`bug_reproducer(237)`" ]
/// Single-membership body: index 0 is always selected (Priority 3 fallback unchanged).
fn mre_bug237_single_membership_fallback_unchanged()
{
  let body = r#"{
    "tagged_id": "user_03QRS",
    "uuid": "eeee-ffff",
    "email_address": "carol@example.com",
    "full_name": "Carol",
    "display_name": "Carol",
    "memberships": [
      { "role": "member", "organization": { "billing_type": "none", "capabilities": ["chat"], "created_at": "2024-01-01T00:00:00Z" } }
    ]
  }"#;
  let result = parse_oauth_account( body ).expect( "should parse" );
  assert_eq!( result.billing_type, "none", "single membership always selected via Priority 3" );
  assert!( !result.has_max );
  assert_eq!( result.memberships.len(), 1, "single membership preserved" );
  assert_eq!( result.tagged_id, "user_03QRS" );
  assert_eq!( result.full_name, "Carol" );
}

// ── AT: audit hardening ───────────────────────────────────────────────────────

#[ test ]
/// AT-01: whitespace between key quote and colon is valid JSON and must parse.
///
/// # Root Cause
/// Every scanner searched for the fused needle `"key":`, coupling the parser to
/// one serializer's spacing — a body emitting `"key" : value` made required
/// fields invisible and failed the whole parse.
///
/// # Why Not Caught
/// All fixtures were written compact, mirroring the API's current serializer;
/// no test exercised JSON-legal spacing variants.
///
/// # Fix Applied
/// `after_key` anchors on the quoted key token, then skips whitespace and
/// requires the colon as a separate step (Fix(audit-needle-colon-coupling)).
///
/// # Prevention
/// This fixture spaces every colon; it fails against the fused-needle scanner.
///
/// # Pitfall
/// A serializer change on the server side is invisible in CI — parsers must
/// accept the JSON grammar, not one pretty-printer's output.
fn at01_whitespace_before_colon_parses()
{
  let body = r#"{
    "tagged_id" : "user_04WS",
    "uuid" : "gggg-hhhh",
    "email_address" : "dora@example.com",
    "memberships" : [
      { "role" : "admin", "organization" : { "billing_type" : "stripe_subscription", "capabilities" : ["claude_max"], "rate_limit_tier" : "default_claude_max_5x", "created_at" : "2024-04-01T00:00:00Z" } }
    ]
  }"#;
  let result = parse_oauth_account( body ).expect( "spaced colons are valid JSON and must parse" );
  assert_eq!( result.tagged_id, "user_04WS" );
  assert_eq!( result.email_address, "dora@example.com" );
  assert_eq!( result.billing_type, "stripe_subscription" );
  assert!( result.has_max );
  assert_eq!( result.rate_limit_tier, "default_claude_max_5x" );
}

#[ test ]
/// AT-02: identity fields serialized AFTER the memberships array are still
/// read from the top level — never shadowed by an organization's same-named keys.
///
/// # Root Cause
/// Identity fields were needle-scanned over the FULL body, so with
/// `memberships` serialized first, the first `"uuid"`/`"created_at"` hit came
/// from inside an organization block, not the user.
///
/// # Why Not Caught
/// The live API serializes identity fields first; fixtures copied that order,
/// so the wrong-scope hit never happened in CI.
///
/// # Fix Applied
/// `identity_scan_regions` masks the memberships array span out of the
/// identity scan (Fix(audit-identity-shadowing)).
///
/// # Prevention
/// This fixture puts memberships first with a decoy org `uuid`; it fails
/// against a full-body scan.
///
/// # Pitfall
/// Top-level and nested JSON keys share names — a needle scanner has no scope
/// awareness unless the nested span is explicitly excluded.
fn at02_identity_after_memberships_not_shadowed_by_org_fields()
{
  let body = r#"{
    "memberships": [
      { "role": "member", "organization": { "uuid": "ORG-UUID-DECOY", "billing_type": "stripe_subscription", "capabilities": ["claude_max"], "created_at": "2024-01-01T00:00:00Z" } }
    ],
    "tagged_id": "user_05TAIL",
    "uuid": "USER-UUID-REAL",
    "email_address": "erin@example.com"
  }"#;
  let result = parse_oauth_account( body ).expect( "should parse" );
  assert_eq!( result.uuid, "USER-UUID-REAL", "user-level uuid must win over the org block's uuid" );
  assert_eq!( result.tagged_id, "user_05TAIL" );
  assert_eq!( result.email_address, "erin@example.com" );
  assert_eq!( result.org_created_at, "2024-01-01T00:00:00Z", "org fields still come from the org block" );
}

#[ test ]
/// AT-03: `claude_max` appearing only as a string *value* of another field is
/// not a capability — `has_max` stays false.
///
/// # Root Cause
/// `org_block.contains( "\"claude_max\"" )` matched the quoted token anywhere
/// in the organization object, so any field whose *value* was the literal
/// string `claude_max` false-flagged Max capability.
///
/// # Why Not Caught
/// Real responses only ever carried the token inside `capabilities`, so the
/// substring shortcut looked equivalent in every fixture.
///
/// # Fix Applied
/// `has_max` now tests membership in the parsed `capabilities` array
/// (Fix(audit-claude-max-substring)).
///
/// # Prevention
/// This fixture plants `"default_model": "claude_max"` with no matching
/// capability; it fails against the substring check.
///
/// # Pitfall
/// Substring presence over serialized JSON has no key/value scope — always
/// test against the parsed structure.
fn at03_claude_max_as_value_is_not_a_capability()
{
  let body = r#"{
    "tagged_id": "user_06VAL",
    "memberships": [
      { "role": "member", "organization": { "billing_type": "stripe_subscription", "default_model": "claude_max", "capabilities": ["chat"], "created_at": "2024-05-01T00:00:00Z" } }
    ]
  }"#;
  let result = parse_oauth_account( body ).expect( "should parse" );
  assert!( !result.has_max, "claude_max as a field VALUE must not flag Max capability" );
  assert_eq!( result.capabilities, vec![ "chat" ] );
}

#[ test ]
/// AT-04: a string value textually equal to an identity key name is skipped —
/// the scanner keeps searching until it finds the real `"key" :` occurrence.
/// Regression guard for `after_key`'s skip-and-continue loop.
fn at04_value_equal_to_key_name_is_skipped()
{
  let body = r#"{
    "display_name": "tagged_id",
    "tagged_id": "user_07SKIP",
    "memberships": [
      { "role": "member", "organization": { "billing_type": "none", "capabilities": [], "created_at": "2024-06-01T00:00:00Z" } }
    ]
  }"#;
  let result = parse_oauth_account( body ).expect( "should parse" );
  assert_eq!( result.tagged_id, "user_07SKIP", "the value occurrence of 'tagged_id' must be skipped" );
  assert_eq!( result.display_name, "tagged_id", "display_name legitimately holds the decoy string" );
}
