# temperature

Model sampling temperature — controls randomness vs determinism in responses.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_TEMPERATURE=<float>
```

Range: `0.0` to `1.0`

## Default

`1.0` (inherits standard)

## Description

Controls the sampling temperature for the model. Lower values produce more deterministic, focused responses. Higher values produce more varied, creative responses.

- `0.0`: Greedy decoding — always selects the highest-probability token. Maximally deterministic.
- `0.5`: Moderate creativity with reasonable consistency.
- `1.0`: Standard temperature — balanced creativity and consistency (Claude's default).

For code generation and factual tasks, lower temperatures (0.1–0.5) typically produce more reliable results. For creative writing or brainstorming, higher temperatures are appropriate.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: 1.0
let cmd = ClaudeCommand::new();

// Low temperature for deterministic code generation
let cmd = ClaudeCommand::new()
  .with_temperature( 0.1 );
```

Builder method: `with_temperature(temperature: f64)` — sets `CLAUDE_CODE_TEMPERATURE`.

## Examples

```bash
# Deterministic for reproducible output
CLAUDE_CODE_TEMPERATURE=0.0 claude --print "Convert this JSON to TOML"

# Creative brainstorming
CLAUDE_CODE_TEMPERATURE=0.9 claude "Brainstorm 10 product name ideas"

# Balanced
CLAUDE_CODE_TEMPERATURE=0.5 claude --print "Generate test cases"
```

## Notes

- NaN and infinity are not valid values; the builder validates these
- Temperature interacts with `top_p` and `top_k` — using multiple sampling params together may have unexpected effects
- At `0.0`, outputs are deterministic for the same input (given same model version)
