# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that claude_runner must always satisfy.
- **Responsibility**: Index of invariant doc instances covering default flag injection, dependency constraints, command naming convention, and trace universality.
- **In Scope**: Default-on flags (`--dangerously-skip-permissions`, `-c`, `--chrome`), zero consumer workspace dependency rule, binary dependency gating, command naming convention (bare words vs `--` flags), `--trace` universality across all subprocess-executing commands.
- **Out of Scope**: Feature behavior (→ `feature/`), API contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Default Flags](001_default_flags.md) | Automatic flag injection and opt-out mechanism | ✅ |
| 002 | [Dependency Constraints](002_dep_constraints.md) | Zero consumer workspace deps, binary deps gated by enabled, no routines.rs | ✅ |
| 003 | [Command Naming](003_command_naming.md) | Commands are bare words; parameters use `--`/`-` prefix | ✅ |
| 004 | [Trace Universality](004_trace_universality.md) | Every subprocess-executing command must support `--trace` | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |
