# Command Group Tests

### Scope

- **Purpose**: Document integration test cases proving (or, here, disproving) `command_group` membership.
- **Responsibility**: Index of per-group equivalence test files, pointing to already-existing tests rather than re-specifying them.
- **In Scope**: Test evidence backing `docs/cli/command_group/`'s Representation Absorption Test verdicts.
- **Out of Scope**: Per-command integration tests (→ `../command/`), per-parameter edge cases (→ `../param/`).

**Source:** [docs/cli/command_group/readme.md](../../../../docs/cli/command_group/readme.md)

No command_group members exist in `claude_storage` — see the source readme's "Evaluated, Not Qualifying" table for the full candidate-pair analysis. This mirror is correspondingly empty of group-specific test files: there is no equivalence behavior to test when no two commands share a routine function.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `procedure.md` | Refresh the zero-group verdict; add and remove group test specs | — |
| `readme.md` | This index — records the zero-group outcome and its evidentiary basis | ✅ |

### Evidentiary Basis for the Zero-Group Verdict

The verdict is a static-dispatch fact, not behavioral output, so no runtime equivalence test applies. It rests on two mechanically-checkable properties of `src/`, both confirmed at doc-authoring time:

1. **`src/cli_main.rs`'s `routines` phf map** registers exactly 16 command names against exactly 16 distinct function identifiers — no two command names resolve to the same routine function.
2. **No `*_routine` function calls another `*_routine` function** — confirmed by grepping every `pub fn *_routine` definition against every `*_routine(` call site across `src/cli/*.rs`; the only matches are each function's own definition line and the `pub use` re-export list in `src/cli/mod.rs`.

Existing per-command integration tests in [`../command/`](../command/readme.md) (16 files, one per command) already exercise each routine's distinct behavior independently; their divergent expected outputs are themselves evidence against equivalence (e.g. `../command/09_project_exists.md`'s INT-1..10 assert exit-code branching that `../command/08_project_path.md`'s tests never exhibit, despite both commands accepting the identical `{path::, topic::}` parameter set). `.rollup`'s own addition is a further, deliberate data point: it shares `scope::`/`path::`/`depth::` parameter wiring with `.usage` yet produces structurally different output (grouped/sorted/projected rows vs. flat per-session rows) — see [`../command/14_rollup.md`](../command/14_rollup.md)'s own Test Cases and `docs/cli/command_group/readme.md`'s "Evaluated, Not Qualifying" table for the `.rollup`/`.usage` non-equivalence verdict.

### Navigation

*(none — zero qualifying groups)*
