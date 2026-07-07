# compact_window

Sets the auto-compaction context window in tokens.

## Type

Env

## Environment Variable

```
CLAUDE_CODE_AUTO_COMPACT_WINDOW
```

## Default

`300000` (300K tokens) — set unconditionally by `ClaudeCommand::new()`.

Standard `claude` default: unset — defers to the model's native context window
(`200000` standard, or `1000000` on extended-context models). See the
binary-perspective entry: [`contract/claude_code/docs/param/074_auto_compact_window.md`](../../../../contract/claude_code/docs/param/074_auto_compact_window.md).

## Description

Caps the context size (in tokens) used for auto-compaction threshold
calculations. When the active conversation approaches this token count (as a
percentage governed by `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`), Claude Code
automatically compacts the conversation.

`claude_runner_core` sets this to `300000` by default — a Tier 1 "different
from claude default" parameter — to prevent automation runs from silently
running on an extended 1M-token window and accumulating a much larger,
slower-to-compact context than intended. Pass `None` via `with_compact_window()`
to opt back into the model's native window (up to 1M on extended-context
models).

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_compact_window( Some( 500_000 ) );  // raise the ceiling
```

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_compact_window( None );  // use model native window (up to 1M for extended)
```

Builder method: `with_compact_window(Option<u32>)`

## Examples

```bash
# Inspect the default in a dry-run
clr --dry-run --message "hi"
# ... CLAUDE_CODE_AUTO_COMPACT_WINDOW=300000 ...
```

## Notes

- Unlike the other Tier 1 defaults (`bash_timeout`, `bash_max_timeout`,
  `auto_continue`, `telemetry`), this parameter accepts `Option<u32>` directly
  in its builder method rather than a bare value — `None` fully suppresses the
  env var, deferring to the binary's own model-native default.
- Only a value **lower** than the model's actual context window is meaningful;
  the binary caps the effective value at the model's real window size.
- Shared with the `isolated`/`refresh` CLI paths in
  `claude_runner::cli::credential` via the `DEFAULT_COMPACT_WINDOW` constant —
  both stay in lockstep.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [../../../../contract/claude_code/docs/param/074_auto_compact_window.md](../../../../contract/claude_code/docs/param/074_auto_compact_window.md) | Binary-perspective reference for the same env var |
