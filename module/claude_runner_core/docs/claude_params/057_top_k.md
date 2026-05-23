# top_k

Top-k sampling cutoff parameter.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_TOP_K=<integer>
```

Range: positive integer (e.g., 1 to several hundred)

## Default

None (inherits standard — Claude uses its internal default when not set)

## Description

Controls top-k sampling. At each token generation step, the model considers only the `k` most probable tokens, regardless of their cumulative probability.

- `k=1`: Greedy — always select the single most likely token (maximally deterministic)
- `k=40`: Consider top 40 tokens at each step (moderate diversity)
- `k=250`: Consider top 250 tokens (high diversity)

Top-k differs from `top_p` in that it uses an absolute count rather than a probability mass cutoff. Top-k is simpler but less adaptive to the model's confidence at each step.

Lower k = more deterministic. Higher k = more diverse output.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: None (inherits standard)
let cmd = ClaudeCommand::new();

// Set top-k
let cmd = ClaudeCommand::new()
  .with_top_k( 40 );
```

Builder method: `with_top_k(top_k: u32)` — sets `CLAUDE_CODE_TOP_K`.

## Examples

```bash
# Deterministic (greedy) output
CLAUDE_CODE_TOP_K=1 claude --print "What is 2+2?"

# Standard top-k sampling
CLAUDE_CODE_TOP_K=40 claude --print "Generate a function name"

# High diversity
CLAUDE_CODE_TOP_K=250 claude --print "Brainstorm variable names"
```

## Notes

- Top-k of 1 is equivalent to greedy decoding (same as `temperature=0`)
- Using top-k, top-p, and temperature simultaneously can have complex interactions
- When not set, Claude uses its internal default (behavior may vary by model version)
