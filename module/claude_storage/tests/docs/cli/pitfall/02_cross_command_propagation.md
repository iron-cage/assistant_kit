# Pitfall :: Cross-Command Bug Propagation

Contract tests verifying that multi-file bug fixes are complete and propagation comments are present at all fix sites.

**Source:** [cli/pitfall/02_cross_command_propagation.md](../../../../docs/cli/pitfall/02_cross_command_propagation.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PF-1 | `search.rs` contains at least 2 propagation-fix comments (issues #009, #012) | Fix Site Annotation |
| PF-2 | `count.rs` contains at least 2 propagation-fix comments (issues #010, #012) | Fix Site Annotation |
| PF-3 | No unpatched copy of a known-buggy pattern survives in any `src/cli/` file | Pattern Exhaustion |

## Test Coverage Summary

- Fix Site Annotation: 2 tests (PF-1, PF-2)
- Pattern Exhaustion: 1 test (PF-3)

**Total:** 3 pitfall contract cases

**Implementation target:** `tests/invariant_contracts_test.rs`

## Test Cases

---

### PF-1: `search.rs` contains at least 2 propagation-fix comments

- **Given:** source file `src/cli/search.rs` (2 documented fix sites for issues #009 and #012)
- **When:** the file is searched for the propagation comment pattern `// Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.`
- **Then:** the comment appears at least 2 times; grep-count-based check (not line-number-specific — line numbers change with refactoring)

---

### PF-2: `count.rs` contains at least 2 propagation-fix comments

- **Given:** source file `src/cli/count.rs` (2 documented fix sites for issues #010 and #012)
- **When:** the file is searched for the propagation comment pattern `// Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.`
- **Then:** the comment appears at least 2 times; grep-count-based check (not line-number-specific — line numbers change with refactoring)

---

### PF-3: No unpatched copy of a known-buggy pattern survives in `src/cli/`

- **Given:** the grep patterns identified during issues #009, #010, and #012 bug fixes
- **When:** `src/cli/` is searched for each pattern
- **Then:** either no match is found, or every match site carries a propagation-fix comment confirming the patch was applied
