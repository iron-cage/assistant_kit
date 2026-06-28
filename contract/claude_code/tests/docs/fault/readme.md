# Fault Tests

Fault classification behavioral test cases for the `contract/claude_code` fault collection. Each file covers one aspect of the fault taxonomy documented in `docs/fault/readme.md`.

### Scope

- **Purpose**: Document FT-N test cases for fault classification behavior described in `docs/fault/readme.md`.
- **Responsibility**: Index of per-fault-surface test case planning files covering the `classify_error()` priority algorithm and detection signal guarantees.
- **In Scope**: Error classification priority order (E1–E6), silent failure mode detection (F1–F4), `classify_error()` correctness contracts.
- **Out of Scope**: Behavioral quirks Q1–Q5 (covered by `tests/behavior/`); live binary invocations for E3/E4 (architecture constraint — N/A); implementation details of `ErrorKind` enum (→ `claude_runner_core/tests/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_fault_classification.md | FT cases for `classify_error()` priority algorithm and dual-channel scanning |
