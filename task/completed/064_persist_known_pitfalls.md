# TSK-064: Add Known Pitfalls section to src/persist.rs module doc

## Goal

Add a `# Known Pitfalls` section to the module-level `//!` doc comment in
`src/persist.rs` documenting the `is_dir()` vs `exists()` confusion revealed
by bug issue-001.

## Motivation

The bug-fix workflow requires a module-level Known Pitfalls section when a bug
reveals a systemic design flaw. Issue-001 found that `path.exists()` was used
instead of `path.is_dir()` to validate the `$PRO` environment variable, allowing
a file path to silently pass as a valid storage root. This is a systemic flaw in
"validate path is a directory" logic that can recur anywhere path validation is added.

## In Scope

- `module/claude_profile/src/persist.rs`:
  - Append `# Known Pitfalls` section to module-level `//!` doc block
  - Document P1: `exists()` vs `is_dir()` for `$PRO` validation (issue-001)

## Out of Scope

- Changes to the inline `Fix(issue-001)` source comment at `resolve_root()` — already correct
- Any other files

## Work Procedure

1. Open `src/persist.rs`. Find the end of the `//!` module doc block
   (line ending with `See FR-15 in \`spec.md\`.`).

2. After that line, append:
```rust
//!
//! # Known Pitfalls
//!
//! ## P1 — `exists()` vs `is_dir()` for `$PRO` validation (issue-001)
//!
//! `path.exists()` returns `true` for both files and directories. Using `exists()`
//! to guard `$PRO` allows a file path to pass as a valid storage root, producing
//! a nonsensical base like `<file>/persistent/claude_profile/` that causes
//! `ensure_exists()` to fail with `ENOTDIR` at runtime.
//!
//! **Always use `is_dir()`** when validating environment variables that must resolve
//! to a directory root. `exists()` is correct only for existence checks where file vs
//! directory doesn't matter.
//!
//! Reproducer: `persist_test.rs::p14_pro_set_to_existing_file_falls_back_to_home`.
```

3. Confirm `cargo build` succeeds with zero warnings.

## Validation List

Desired answer for every question is YES.

- [ ] Does `src/persist.rs` module doc have a `# Known Pitfalls` section?
- [ ] Does the pitfall name `is_dir()` vs `exists()` explicitly?
- [ ] Does the pitfall reference `issue-001` (traceability)?
- [ ] Does the pitfall reference `p14` reproducer test (traceability)?
- [ ] Does `cargo build` succeed with zero warnings after the change?
- [ ] Is the inline fix comment at the `is_dir()` call site unchanged?

## Validation Procedure

### Measurements

**M1 — Known Pitfalls section present**
```bash
grep -c "Known Pitfall" module/claude_profile/src/persist.rs
```
Before: 0. Expected: ≥1.

**M2 — Pitfall mentions is_dir**
```bash
grep -c "is_dir" module/claude_profile/src/persist.rs
```
Before: 1 (inline comment). Expected: ≥2 (one in module doc, one inline).

**M3 — Compile passes**
```bash
cd /home/user1/pro/lib/wip_core/claude_tools/dev && RUSTFLAGS="-D warnings" cargo build -p claude_profile --all-features 2>&1 | tail -3
```
Expected: `Finished` — zero errors, zero warnings.

### Anti-faking checks

**AF1 — Not just a comment stub**
```bash
grep -A 10 "Known Pitfall" module/claude_profile/src/persist.rs | grep -c "is_dir\|exists\|ENOTDIR"
```
Expected: ≥2 (the pitfall body contains substantive detail).

## Outcomes

**Completed:** 2026-03-31
**Result:** Done — appended `# Known Pitfalls` section to `src/persist.rs` module doc documenting the `is_dir()` vs `exists()` pitfall from issue-001 with reproducer reference.
**Files changed:** `module/claude_profile/src/persist.rs`
