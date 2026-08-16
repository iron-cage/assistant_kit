# claude_runner_core

> **Workspace:** assistant — `claude_runner_core`

Claude Code process execution with builder pattern and single execution point.

## Files

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest: deps, features, metadata |
| `src/` | Builder pattern implementation: `ClaudeCommand`, types, process scanner |
| `tests/` | Builder API, migration validation, verification framework (43 test files) |
| `docs/` | Behavioral requirements: features, invariants, parameter reference |
| `../../../../agent_kit/task/claude_runner_core/` | Crate task registry — External Layout (see `agent_kit/task/`) |
| `verb/` | Shell scripts for each `do` protocol verb. |

### Responsibility Table

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|---------------|--------------|-------|--------------|
| claude_runner_core | Claude Code process execution | ClaudeCommand Config → Process Output | Command building, process spawning, output capture, token limits | ❌ Session storage paths → `claude_profile`<br>❌ Continuation detection → `claude_profile`<br>❌ Context injection → `dream_agent`<br>❌ Parameter parsing → `dream_agent`<br>❌ Session strategy → `dream_agent` |

### Scope

**Responsibility:**
- Claude Code process execution (Command::new("claude"))
- Builder pattern API (ClaudeCommand::new().with_*())
- Token limit configuration (128K default)
- Process output capture (stdout/stderr)
- Single execution point (duplication = 1x)

**In Scope:**
- ClaudeCommand::new() builder entry point
- with_working_directory(), with_max_output_tokens(), with_continue_conversation(), etc. (69 typed builder methods)
- execute() terminal method with process spawning
- stdout/stderr capture and parsing
- Exit code handling and error mapping

**Out of Scope:**
- ❌ Session storage path resolution → delegated to `claude_profile` crate
- ❌ Continuation detection → delegated to `claude_profile` crate
- ❌ Context injection from consumer_runner → delegated to `dream_agent` crate
- ❌ Parameter parsing from CLI → delegated to `dream_agent` crate
- ❌ Session lifecycle strategy → delegated to `dream_agent` crate

## Features

- **Builder Pattern**: Fluent API with method chaining (NO deprecated factories)
- **Token Limit Fix**: Explicit 128K token default (prevents "exceeded maximum" errors)
- **Single Execution Point**: Consolidates duplicate Command::new("claude") calls
- **Type Safety**: Builder pattern enforces correct configuration
- **Lean Dependencies**: claude_core + tempfile, plus optional error_tools/serde/serde_json/data_fmt behind features

## Usage

```rust,no_run
use claude_runner_core::ClaudeCommand;

// Basic execution
let result = ClaudeCommand::new()
  .with_working_directory("/home/user/project")
  .with_max_output_tokens(128_000)
  .with_continue_conversation(true)
  .execute()?;

println!("Output: {}", result.stdout);

// Advanced configuration
let result = ClaudeCommand::new()
  .with_working_directory("/tmp/work")
  .with_max_output_tokens(128_000)
  .with_model("claude-opus-4-5")
  .with_verbose(true)
  .with_system_prompt("You are a helpful coding assistant")
  .with_message("Fix the bug in main.rs")
  .execute()?;

# Ok::<(), Box<dyn std::error::Error>>(())
```

## Architecture

```text
Builder Pattern Flow:

ClaudeCommand::new()
  └→ with_working_directory()      (fluent method chaining)
  └→ with_max_output_tokens()
  └→ with_continue_conversation()
  └→ execute()                     ← SINGLE execution point
      └→ build_command()           (assembles std::process::Command)
      └→ Command::new("claude")    ← ONLY location in entire codebase
      └→ output()                  (spawn, capture stdout/stderr)
      └→ Return ExecutionOutput
```

## Migration from Old API

**Before (DEPRECATED - DO NOT USE):**
```text
// Factory method (DEPRECATED)
ClaudeCommand::generate(/* 40 parameters */)

// Mixed execution (DEPRECATED)
session.execute_interactive()
session.execute_non_interactive()

// Duplicate execution points (2x)
Command::new("claude")  // Location 1
Command::new("claude")  // Location 2
```

**After (THIS CRATE):**
```text
// Builder pattern (CORRECT)
ClaudeCommand::new()
  .with_*()
  .execute()

// Single execution point (1x)
Command::new("claude")  // ONLY in claude_runner_core::build_command()
```

## Token Limit Bug Fix

**Problem:** Default Claude Code token limit is 32K, causing "exceeded maximum" errors

**Solution:** Set explicit max_output_tokens to 128K:

```rust,no_run
use claude_runner_core::ClaudeCommand;

let result = ClaudeCommand::new()
  .with_max_output_tokens(128_000)  // Explicit token limit
  .execute()?;

# Ok::<(), Box<dyn std::error::Error>>(())
```

## Reference Documentation

- **Parameter Reference**: `docs/claude_params/` — all 59 `claude` binary parameters (CLI flags + env vars), with builder API mapping and default comparisons
- **Builder API**: `src/command/` doc comments — authoritative builder method documentation
- **Tests**: `tests/readme.md` — full test suite coverage map
- **Tasks**: `../../../../agent_kit/task/claude_runner_core/` — crate task registry (External Layout)

## Dependencies

- **claude_core**: Shared process scanning and profile primitives (workspace)
- **tempfile**: Random-suffixed private temp dirs/files for isolated runs and stdin materialization
- **error_tools** *(optional, feature `enabled`)*: Workspace-standard error handling
- **serde / serde_json** *(optional, feature `enabled`)*: Control-protocol and JSON output support
- **data_fmt** *(optional, feature `ps_table`)*: Process-table rendering

Verify: `sed -n '/\[dependencies\]/,/\[dev-dependencies\]/p' Cargo.toml`

## Testing

```bash
cargo nextest run
```

## Critical Execution Rule

**Command::new("claude") MUST appear exactly once:**
- ✅ Single occurrence in claude_runner_core::build_command()
- ❌ Zero occurrences in dream_agent
- ❌ Zero occurrences in claude_profile

Verification: `grep -rn 'Command::new( "claude" )' src/` should find exactly 1 match (`src/command/mod.rs`, inside `build_command()`; doc examples and the Windows `Command::new( "cmd" )` fallback don't match this pattern).
