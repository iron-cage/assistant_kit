# claude_runner_core

> **Workspace:** assistant — `claude_runner_core`

Claude Code process execution with builder pattern and single execution point.

## Files

| File / Directory | Responsibility |
|------------------|----------------|
| `Cargo.toml` | Crate manifest: deps, features, metadata |
| `src/` | Builder pattern implementation: `ClaudeCommand`, types, process scanner |
| `tests/` | Builder API, migration validation, verification framework (31 test files) |
| `docs/` | Behavioral requirements: features, invariants, parameter reference |
| `task/` | Crate-level task tracking |
| `verb/` | Shell scripts for each `do` protocol verb. |

### Responsibility Table

| Entity | Responsibility | Input→Output | Scope | Out of Scope |
|--------|---------------|--------------|-------|--------------|
| claude_runner_core | Claude Code process execution | ClaudeCommand Config → Process Output | Command building, process spawning, output capture, token limits | ❌ Session storage paths → `claude_profile`<br>❌ Continuation detection → `claude_profile`<br>❌ Context injection → `dream_agent`<br>❌ Parameter parsing → `dream_agent`<br>❌ Session strategy → `dream_agent` |

### Scope

**Responsibility:**
- Claude Code process execution (Command::new("claude"))
- Builder pattern API (ClaudeCommand::new().with_*())
- Token limit configuration (200K default)
- Process output capture (stdout/stderr)
- Single execution point (duplication = 1x)

**In Scope:**
- ClaudeCommand::new() builder entry point
- with_working_directory(), with_max_output_tokens(), with_continue_conversation(), etc. (61 typed builder methods)
- execute() terminal method with process spawning
- stdout/stderr capture and parsing
- Exit code handling and error mapping

**Out of Scope:**
- ❌ Session storage path resolution → delegated to `claude_profile` crate
- ❌ Continuation detection → delegated to `claude_profile` crate
- ❌ Context injection from wplan → delegated to `dream_agent` crate
- ❌ Parameter parsing from CLI → delegated to `dream_agent` crate
- ❌ Session lifecycle strategy → delegated to `dream_agent` crate

## Features

- **Builder Pattern**: Fluent API with method chaining (NO deprecated factories)
- **Token Limit Fix**: Explicit 200K token default (prevents "exceeded maximum" errors)
- **Single Execution Point**: Consolidates duplicate Command::new("claude") calls
- **Type Safety**: Builder pattern enforces correct configuration
- **Minimal Dependencies**: Only error_tools + standard library

## Usage

```rust,no_run
use claude_runner_core::ClaudeCommand;

// Basic execution
let result = ClaudeCommand::new()
  .with_working_directory("/home/user/project")
  .with_max_output_tokens(200_000)
  .with_continue_conversation(true)
  .execute()?;

println!("Output: {}", result.stdout);

// Advanced configuration
let result = ClaudeCommand::new()
  .with_working_directory("/tmp/work")
  .with_max_output_tokens(200_000)
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
      └→ CommandBuilder::build()   (construct std::process::Command)
      └→ Command::new("claude")    ← ONLY location in entire codebase
      └→ ProcessExecutor::run()    (spawn, capture output)
      └→ Return ExecutionResult
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
Command::new("claude")  // ONLY in claude_runner_core::execute()
```

## Token Limit Bug Fix

**Problem:** Default Claude Code token limit is 32K, causing "exceeded maximum" errors

**Solution:** Set explicit max_output_tokens to 200K:

```rust,no_run
use claude_runner_core::ClaudeCommand;

let result = ClaudeCommand::new()
  .with_max_output_tokens(200_000)  // Explicit token limit
  .execute()?;

# Ok::<(), Box<dyn std::error::Error>>(())
```

## Reference Documentation

- **Parameter Reference**: `docs/claude_params/` — all 59 `claude` binary parameters (CLI flags + env vars), with builder API mapping and default comparisons
- **Builder API**: `src/command.rs` doc comments — authoritative builder method documentation
- **Tests**: `tests/readme.md` — full test suite coverage map
- **Tasks**: `task/` — crate-level task tracking

## Dependencies

- **error_tools**: Workspace-standard error handling (Result, Error types)

Total: 1 workspace dependency (error_tools), 0 external direct dependencies

## Testing

```bash
cargo nextest run
```

## Critical Execution Rule

**Command::new("claude") MUST appear exactly once:**
- ✅ Single occurrence in claude_runner_core::execute()
- ❌ Zero occurrences in dream_agent
- ❌ Zero occurrences in claude_profile

Verification: `grep -r "Command::new.*claude" src/` should find exactly 1 match.
