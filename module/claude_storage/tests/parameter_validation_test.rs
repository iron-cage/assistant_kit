//! Parameter Validation Tests
//!
//! # Root Cause
//!
//! During manual testing of v1.3.0, we discovered that several CLI commands silently
//! accept invalid parameter values instead of returning clear error messages. This creates
//! user confusion and makes debugging difficult.
//!
//! Specifically:
//!
//! 1. **`.list ``type::invali``d`** - The `type` parameter accepts ANY value without validation.
//!    The code has a catch-all `_` pattern in the match statement that treats all invalid
//!    values as "all", silently listing all projects instead of erroring.
//!
//! 2. **`.count ``target::invali``d`** (Finding #009) - The `target` parameter accepts ANY string
//!    value without validation. Invalid values like "project" (singular) are processed with
//!    confusing errors instead of clear parameter validation messages.
//!
//! # Why Not Caught
//!
//! The existing test suite (55 tests) focused on:
//! - Valid parameter combinations
//! - Functional correctness
//! - Integration workflows
//!
//! But did NOT test:
//! - Invalid parameter value handling
//! - Error message clarity
//! - Parameter value range validation
//!
//! The code relied on unilang's type checking (integer, string, boolean) but didn't add
//! application-level value validation on top of that.
//!
//! # Fix Applied
//!
//! 1. **`.list type::` validation**: Replace catch-all `_` pattern with explicit "all" case
//!    and error return for invalid values.
//!
//! 2. **`.list min_entries::` validation**: Add check to reject negative values.
//!
//! # Prevention
//!
//! ## Parameter Validation Policy
//!
//! All CLI commands must validate parameter values at the application level:
//!
//! 1. **Enum-like parameters** (type, target, etc): Explicitly list all valid values,
//!    error on anything else. Never use catch-all `_` patterns for parameter values.
//!
//! 2. **Error messages**: Always include:
//!    - What value was provided
//!    - Why it's invalid
//!    - What values are valid
//!
//!    Example: "Invalid type: foo. Valid values: uuid, path, all"
//!
//! 3. **Boolean-like integers** (`agent::0|1`, `show_sessions::0|1)`: Rely on unilang's boolean
//!    type checking which already validates 0|1 values.
//!
//! # Pitfall to Avoid
//!
//! **Pitfall**: Using catch-all patterns (`_`) or silent clamping for parameter values.
//!
//! **Reality**: Users need clear feedback when they provide invalid parameter values.
//! Silent acceptance leads to:
//! - Confusion about why command behaves unexpectedly
//! - Difficulty debugging typos in parameter values
//! - Undocumented behavior (what does invalid value do?)
//!
//! **Lesson**: Validate ALL parameter values explicitly. Fail fast with clear error
//! messages. Don't silently accept or clamp invalid values without documentation.

mod common;

/// Test that .list rejects invalid `type::` values
#[ test ]
fn test_list_type_parameter_validation()
{
  let output = common::clg_cmd()
    .args( [ ".list", "type::invalid" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  // Should error with clear message
  assert!(
    !output.status.success(),
    "Command should fail with invalid type parameter"
  );

  assert!(
    combined.contains( "Invalid type" ) || combined.contains( "invalid" ),
    "Error message should mention invalid type. Got: {combined}"
  );
}

/// Test that .list accepts valid `type::` values
#[ test ]
fn test_list_type_parameter_valid_values()
{
  // CLAUDE_STORAGE_ROOT isolation: container's /workspace/.claude/projects is
  // bind-mounted with 0700 (host uid), unreadable by the test user. Point to a
  // non-existent path so list_projects() returns empty list instead of an error.

  // type::uuid
  let output = common::clg_cmd()
    .args( [ ".list", "type::uuid" ] )
    .env( "CLAUDE_STORAGE_ROOT", "/tmp/claude_tests_empty" )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );
  assert!( output.status.success(), "type::uuid should be valid" );

  // type::path
  let output = common::clg_cmd()
    .args( [ ".list", "type::path" ] )
    .env( "CLAUDE_STORAGE_ROOT", "/tmp/claude_tests_empty" )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );
  assert!( output.status.success(), "type::path should be valid" );

  // type::all
  let output = common::clg_cmd()
    .args( [ ".list", "type::all" ] )
    .env( "CLAUDE_STORAGE_ROOT", "/tmp/claude_tests_empty" )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );
  assert!( output.status.success(), "type::all should be valid" );
}

/// Test that .list rejects negative `min_entries`
#[ test ]
fn test_list_min_entries_negative_validation()
{
  let output = common::clg_cmd()
    .args( [ ".list", "min_entries::-5" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  // Should error with clear message
  assert!(
    !output.status.success(),
    "Command should fail with negative min_entries. Got: {combined}"
  );

  assert!(
    combined.contains( "min_entries" ) && combined.contains( "negative" ) || combined.contains( "positive" ),
    "Error message should mention min_entries must be positive. Got: {combined}"
  );
}

/// Test that .show `entries::1` is accepted in content mode (Fix issue-022)
///
/// ## Root Cause
/// A prior "fix" (issue-008) added an error when `entries::1` was used in content
/// mode (verbosity >= 1 && !`metadata_only`), intending to prevent silent-ignore of
/// the parameter. However, the YAML spec example 6 explicitly lists
/// `.show ``session_id::abc123`` ``entries::``1` as valid without `metadata::1`.
/// Content mode already displays all entries by default — `entries::1` is a valid
/// no-op in this context, not an invalid parameter combination.
///
/// ## Why Not Caught
/// The previous fix prioritised "no garbage parameters" over spec compliance.
/// The YAML examples were not checked against the implementation restriction.
///
/// ## Fix Applied
/// Removed the error block in `show_routine` (src/cli/mod.rs) that rejected
/// `entries::1` when not in metadata mode. `entries::1` in content mode is now
/// accepted as a no-op (content mode already shows all entries).
///
/// ## Prevention
/// Before adding an error for a parameter combination, verify that the YAML spec
/// examples do not show that combination as valid. YAML examples are authoritative —
/// they define the user-visible contract.
///
/// ## Pitfall
/// Over-correcting a "garbage parameter" issue by rejecting spec-valid combinations.
/// A no-op is always preferable to an error when the spec documents the combination.
#[ test ]
fn test_show_entries_accepted_in_content_mode()
{
  // session_id::test-session-id won't exist, so we get a project-not-found error,
  // but the key assertion is that the error is NOT about entries/metadata mode
  let output = common::clg_cmd()
    .args( [ ".show", "session_id::test-session-id", "show_entries::1" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  // The command will fail (session not found / project not found) but must NOT
  // fail with "entries only works in metadata mode"
  assert!(
    !combined.contains( "entries" ) || !combined.contains( "metadata mode" ),
    "entries::1 must NOT be rejected as invalid in content mode. Got: {combined}"
  );
}

/// Test that .show entries works correctly in metadata mode
#[ test ]
fn test_show_entries_works_in_metadata_mode()
{
  // This test verifies that entries::1 IS accepted when in metadata mode
  // We expect this to fail for a different reason (session not found),
  // NOT because of parameter validation

  let output = common::clg_cmd()
    .args( [ ".show", "session_id::test-session-id", "show_metadata::1", "show_entries::1" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  // Should fail because session doesn't exist, NOT because of parameter validation
  assert!(
    !output.status.success(),
    "Command should fail (session not found), but for different reason than parameter validation"
  );

  // Should NOT mention "entries" + "metadata" validation error
  let is_param_validation_error = combined.contains( "entries" ) && combined.contains( "metadata" ) && combined.contains( "only works" );

  assert!(
    !is_param_validation_error,
    "Should fail due to missing session, NOT parameter validation. Got: {combined}"
  );
}

/// Test .count target parameter validation (Finding #009)
///
/// ## Root Cause
/// .count command accepts any string for target parameter without validation.
/// Invalid values like "project" (singular) or "foo" are silently processed,
/// causing confusing behavior or silent failures.
///
/// ## Why Not Caught
/// .count command has minimal test coverage. The existing tests only verify
/// valid target values (projects, sessions, entries). No tests checked invalid
/// values or error handling.
///
/// ## Fix Applied
/// Added explicit validation in `count_routine()` to check target parameter
/// against valid values (projects, sessions, entries). Returns clear error
/// message listing valid values when invalid target provided.
///
/// ## Prevention
/// All enumerated parameters must validate against allowed values at routine
/// entry. Return clear error messages that include:
/// - What value was provided
/// - List of valid values
/// - Parameter name
///
/// ## Pitfall
/// Don't assume unilang parser validates enum value constraints. Parser only
/// validates type (String), not value constraints. Application code must
/// validate enumerated parameter values explicitly.
#[ test ]
fn test_count_target_invalid_value()
{
  let output = common::clg_cmd()
    .args( [ ".count", "target::invalid" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Command should fail with invalid target. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "invalid" ) &&
    combined.to_lowercase().contains( "target" ),
    "Error should mention invalid target. Got: {combined}"
  );
}

/// Test .count target accepts valid values
#[ test ]
fn test_count_target_valid_values()
{
  // CLAUDE_STORAGE_ROOT isolation: see test_list_type_parameter_valid_values.

  // target::projects (default) - should always succeed
  let output = common::clg_cmd()
    .args( [ ".count" ] )
    .env( "CLAUDE_STORAGE_ROOT", "/tmp/claude_tests_empty" )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  assert!( output.status.success(), "target::projects (default) should be valid" );

  // target::projects (explicit) - should succeed
  let output = common::clg_cmd()
    .args( [ ".count", "target::projects" ] )
    .env( "CLAUDE_STORAGE_ROOT", "/tmp/claude_tests_empty" )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  assert!( output.status.success(), "target::projects should be valid" );

  // target::sessions — counts all sessions globally; no project required
  let output = common::clg_cmd()
    .args( [ ".count", "target::sessions" ] )
    .env( "CLAUDE_STORAGE_ROOT", "/tmp/claude_tests_empty" )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  // target::sessions succeeds without project:: (counts across all projects)
  assert!(
    output.status.success(),
    "target::sessions must succeed without project:: parameter. Got: {combined}"
  );

  assert!(
    stdout.trim().parse::< u64 >().is_ok(),
    "target::sessions must output a bare integer. Got: {combined}"
  );

  // Must NOT produce an "invalid target" error
  assert!(
    !combined.to_lowercase().contains( "invalid" ) || !combined.to_lowercase().contains( "target" ),
    "Should not error on target validation. Got: {combined}"
  );
}

/// Test .count target singular form (common typo)
#[ test ]
fn test_count_target_singular_form()
{
  let output = common::clg_cmd()
    .args( [ ".count", "target::project" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Command should fail with singular 'project'. Got: {combined}"
  );

  assert!(
    combined.contains( "project" ) && (combined.contains( "valid" ) || combined.contains( "projects" )),
    "Error should mention valid plural form 'projects'. Got: {combined}"
  );
}

/// Test .count target empty value
#[ test ]
fn test_count_target_empty_value()
{
  let output = common::clg_cmd()
    .args( [ ".count", "target::" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Command should fail with empty target. Got: {combined}"
  );
}
