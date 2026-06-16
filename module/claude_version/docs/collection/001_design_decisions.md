# Collection: Design Decisions

### Scope

- **Purpose**: Key design rationale for the clv CLI redesign.
- **Responsibility**: Record of architectural and interface decisions with justification.
- **In Scope**: Syntax choices, pipeline design, exit code semantics, parameter conventions.
- **Out of Scope**: Behavioral contracts (-> `feature/`), CLI reference surface (-> `cli/`), design patterns (-> `pattern/`).

### Items

| ID | Decision | Category |
|----|----------|----------|
| D1 | Unilang `.command param::value` syntax | Syntax |
| D2 | Two-level subcommands | Syntax |
| D3 | Boolean parameters use `0`/`1` values | Parameter Conventions |
| D4 | Unilang exit code semantics via `ErrorData` | Pipeline |
| D5 | Unilang 5-phase pipeline with custom adapter layer | Pipeline |
| D6 | docs/cli/ with three-layer structure | Documentation |
| D7 | Unilang re-adopted for per-command validation | Pipeline |
| D8 | Last occurrence wins for repeated parameters | Parameter Conventions |

### Classification

Decisions organized by concern area:
- **Syntax**: D1, D2 — command naming and structure conventions
- **Pipeline**: D4, D5, D7 — unilang pipeline architecture choices
- **Parameter Conventions**: D3, D8 — parameter parsing and precedence rules
- **Documentation**: D6 — CLI reference structure

### D1 — Unilang `.command param::value` syntax

The CLI uses unilang-based syntax: dot-prefixed commands (`.version.install`)
with `param::value` parameters (`version::stable dry::1`).  This is the
canonical syntax for all claude_version invocations.

### D2 — Two-level subcommands

Commands use at most two dot-separated segments (`.version.show`, `.settings.get`).
Single-segment commands (`.status`, `.processes`, `.help`) are also supported.

### D3 — Boolean parameters use `0`/`1` values

`dry::1` and `force::1` enable the flag; `dry::0` and `force::0` disable it.
Explicit values avoid ambiguity in script composition.

### D4 — Unilang exit code semantics via `ErrorData`

Command routines return `Result<OutputData, ErrorData>` (unilang types). `OutputData`
carries the payload string and format hint. `ErrorData` carries the exit code via
`ErrorCode`: `InternalError | CommandNotImplemented` -> exit 2; all others -> exit 1.

### D5 — Unilang 5-phase pipeline with custom adapter layer

The CLI is implemented via the unilang 5-phase pipeline
(Adapter -> Parser -> SemanticAnalyzer -> Interpreter -> Output). The custom
`src/adapter.rs` layer handles `claude_version`-specific concerns: `v::` alias
expansion (to `verbosity::`), strict 0/1 boolean validation (rejects `dry::true`),
and integer range checks. Unilang provides the command registry, per-command parameter
validation (SemanticAnalyzer rejects unknown params per command), and consistently
formatted error messages.

### D6 — docs/cli/ with three-layer structure

A proper three-layer reference (command/, param/, type/) with parameter
groups, dictionary, and workflows — all in unilang syntax.

### D7 — Unilang re-adopted for per-command validation

`unilang` was added back to Cargo.toml after the hand-rolled parser proved inadequate
for per-command parameter scoping. The unilang SemanticAnalyzer rejects unknown
parameters per command (not globally), which prevents silent acceptance of params on
wrong commands (e.g., `format::` on `.version.guard`). Consistent error message
formatting across all 12 commands is a further benefit. The custom `adapter.rs` layer
retains full control over `claude_version`-specific normalisation without forking unilang.

### D8 — Last occurrence wins for repeated parameters

When `v::` appears multiple times, the last value wins. Simplifies script composition.

### Features

| File | Relationship |
|------|-------------|
| [feature/005_cli_design.md](../feature/005_cli_design.md) | CLI pipeline design implementing these decisions |

### Tests

| File | Relationship |
|------|-------------|
| [`../../tests/docs/collection/001_design_decisions.md`](../../tests/docs/collection/001_design_decisions.md) | Decision implementation test spec |
