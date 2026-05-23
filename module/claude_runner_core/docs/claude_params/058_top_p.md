# top_p

Top-p (nucleus) sampling parameter.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_TOP_P=<float>
```

Range: `0.0` to `1.0`

## Default

None (inherits standard — Claude uses its internal default when not set)

## Description

Controls nucleus sampling (top-p). At each token generation step, the model considers only the smallest set of tokens whose cumulative probability exceeds `p`.

- `1.0`: Consider all tokens (equivalent to no top-p filtering)
- `0.9`: Consider top 90% probability mass tokens
- `0.5`: Consider top 50% probability mass tokens (more restrictive)

Lower values reduce output diversity by excluding low-probability tokens. Higher values allow more varied outputs.

Top-p is an alternative to `temperature` for controlling output diversity. Using both simultaneously is possible but may have complex interactions — typically one or the other is sufficient.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: None (inherits standard)
let cmd = ClaudeCommand::new();

// Set top-p
let cmd = ClaudeCommand::new()
  .with_top_p( 0.9 );
```

Builder method: `with_top_p(top_p: f64)` — sets `CLAUDE_CODE_TOP_P`.

## Examples

```bash
# Conservative sampling
CLAUDE_CODE_TOP_P=0.7 claude --print "Generate test data"

# Standard nucleus sampling
CLAUDE_CODE_TOP_P=0.95 claude --print "Write documentation"
```

## Notes

- NaN and infinity are not valid; the builder tests validate these edge cases
- Using both `temperature` and `top_p` simultaneously creates complex sampling behavior
- When not set, Claude uses its internal default (behavior may vary by model)
