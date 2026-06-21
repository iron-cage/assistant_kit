# Feature: CLI Design

### Scope

- **Purpose**: Document the overall CLI architecture of claude_version including command routing, parameter parsing, exit codes, and help output.
- **Responsibility**: Describe the 5-phase unilang pipeline, parameter validation rules, exit code semantics, and help listing behavior.
- **In Scope**: Command syntax (`.command param::value`), 5-phase pipeline, parameter validation, exit codes 0/1/2, help listing, adapter layer.
- **Out of Scope**: Individual command behavior (→ other feature/ instances), version lock pattern (→ `pattern/`), type inference (→ `algorithm/`), CLI reference surface — syntax tables, type definitions, parameter defaults (→ `../cli/`).

### Design

**Command syntax:** All commands use dot-prefixed dot-separated tokens (`.version.install`, `.processes.kill`, etc.) followed by `param::value` pairs. Unknown parameters exit 1.

**5-phase unilang pipeline (`run_cli()`):**
1. Adapter — converts argv to unilang tokens; handles `v::`/`verbosity::` alias expansion, bool/integer validation, overflow guards
2. Parser — tokenizes unilang input
3. Analyzer — validates command and parameter names, required params, value constraints
4. Interpreter — dispatches to command routines in `commands/`
5. Stdout/stderr — formats output and writes results

**Parameter rules:**
- `v::N` / `verbosity::N`: must be 0–2 integer; out of range → exit 1
- `dry::`, `force::`: boolean, value must be `0` or `1`
- `format::`: `text` (default) or `json`
- `key::`, `value::`: string, required by their commands; absent or empty → exit 1
- `interval::N`, `count::N`: non-negative integer ≤ `i64::MAX`; overflow → exit 1
- Missing value for any parameter → exit 1 with `"{param} requires a value"`
- Unknown parameter → exit 1 with `"Unknown parameter '{param}'"`
- Last occurrence wins for repeated parameters

**Exit codes:**

| Code | Meaning | Trigger |
|------|---------|---------|
| 0 | Success | — |
| 1 | Usage error | Unknown param, bad value, missing required |
| 2 | Runtime error | `ErrorCode::InternalError` or `CommandNotImplemented` |

**Help listing:** `.help` displays commands grouped by functional category (version management, settings & config, process lifecycle, status), all shared parameters, and usage examples. Empty argv also displays help and exits 0. `.help` anywhere in argv triggers help output. Rendered via `cli_fmt::CliHelpTemplate` — intercepted before the unilang pipeline.

**Binary names:** `claude_version` (primary binary) and `clv` (alias binary). Both delegate to `run_cli()`.

**`#[inline]` requirement:** Workspace lint `missing_inline_in_public_items = "warn"` with `-D warnings` requires `#[inline]` on every `pub fn`, `pub` method, and trait impl (`Display::fmt`, `Default::default`).

### Features

| File | Relationship |
|------|-------------|
| [feature/001_version_management.md](001_version_management.md) | Version commands handled by this pipeline |
| [feature/002_process_lifecycle.md](002_process_lifecycle.md) | Process commands handled by this pipeline |
| [feature/003_settings_management.md](003_settings_management.md) | Settings commands handled by this pipeline |
| [feature/006_config_command.md](006_config_command.md) | Unified .config command handled by this pipeline |

### Design Decisions

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

Decisions by concern area: **Syntax**: D1, D2 | **Pipeline**: D4, D5, D7 | **Parameter Conventions**: D3, D8 | **Documentation**: D6

**D2 — Two-level subcommands:** Commands use at most two dot-separated segments (`.version.show`, `.settings.get`). Single-segment commands (`.status`, `.processes`, `.help`) are also supported.

**D6 — docs/cli/ with three-layer structure:** A proper three-layer reference (`command/`, `param/`, `type/`) with parameter groups, dictionary, and workflows — all in unilang syntax.

**D7 — Unilang re-adopted for per-command validation:** `unilang` was added back to Cargo.toml after the hand-rolled parser proved inadequate for per-command parameter scoping. The unilang SemanticAnalyzer rejects unknown parameters per command (not globally), which prevents silent acceptance of params on wrong commands (e.g., `format::` on `.version.guard`). Consistent error message formatting across all 12 commands is a further benefit. The custom `adapter.rs` layer retains full control over `claude_version`-specific normalisation without forking unilang.

### Sources

| File | Relationship |
|------|-------------|
| `../../src/adapter.rs` | Argv to unilang token adapter |
| `../../src/lib.rs` | run_cli() 5-phase pipeline |

### Provenance

| Source | Notes |
|--------|-------|
| `spec.md` (deleted) | Architecture, FR-01 through FR-04, FR-10, FR-11, Command/Parameter Inventory |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/05_cli_design.md](../../tests/docs/feature/05_cli_design.md) | Feature test spec |
