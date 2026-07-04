# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that the library must always satisfy.
- **Responsibility**: Index of invariant doc instances capturing measurable guarantees and enforcement mechanisms.
- **In Scope**: Safety guarantees (append-only, atomic writes), performance thresholds with measurement methods.
- **Out of Scope**: Functional design (→ `feature/`), algorithm correctness properties (→ `algorithm/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Safety Guarantees](001_safety_guarantees.md) | Append-only semantics and atomic write correctness | ✅ |
| 002 | [Performance](002_performance.md) | Measurable throughput and memory targets | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
