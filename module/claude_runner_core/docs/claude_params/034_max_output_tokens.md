# max_output_tokens

Maximum number of tokens Claude is allowed to produce per response.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_MAX_OUTPUT_TOKENS=<integer>
```

## Default

`128000` (in `claude_runner_core` builder)

Standard claude default: `32000`

## Description

Sets the maximum token count for Claude's response. Higher values allow longer, more comprehensive responses at the cost of more API usage and latency.

The `claude_runner_core` builder defaults to `128000` (128K) instead of the standard 32K. This is intentional — programmatic automation often requires long responses (e.g., generating entire files, comprehensive analyses, large refactoring). The higher default prevents premature truncation in automated workflows.

128K (not higher) is deliberate: it is the real sync max-output ceiling of every current-tier model (Sonnet 5, Opus 4.8, Fable 5— see `contract/claude_code/docs/model/`). An earlier default of `200000` (BUG-429) exceeded that ceiling on every model reachable through this builder — no model could ever honor the portion above 128K, so the value provided no benefit and risked outright request rejection.

Token counts are approximate. 1 token ≈ 4 characters of English text.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: 128,000 tokens
let cmd = ClaudeCommand::new();

// Custom limit
let cmd = ClaudeCommand::new()
  .with_max_output_tokens( 50_000 );
```

Builder method: `with_max_output_tokens(tokens: u32)` — sets `CLAUDE_CODE_MAX_OUTPUT_TOKENS`.

## Examples

```bash
# Shell: set token limit
CLAUDE_CODE_MAX_OUTPUT_TOKENS=100000 claude --print "Generate a large file"

# Low limit for quick responses
CLAUDE_CODE_MAX_OUTPUT_TOKENS=1000 claude --print "One sentence summary"
```

## Notes

- `claude_runner_core` default (128K) vs standard claude default (32K): intentional for automation use, and 128K is the real ceiling of the current model tier (see BUG-429)
- Responses are truncated at this limit; set high enough for the expected output size
- The API charges for output tokens up to this limit (capped at actual output, not the max)
