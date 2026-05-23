# max_budget_usd

Maximum dollar amount to spend on API calls for the session.

## Type

**CLI** — numeric value (USD)

## Syntax

```
claude --print --max-budget-usd <amount>
```

## Default

None (no spend cap)

## Description

Sets a hard cap on API spending for the session. When the cumulative cost of API calls reaches this amount, Claude Code stops making additional calls and returns with whatever has been computed so far.

Only works with `--print` mode.

Useful for:
- Preventing runaway costs in automated pipelines
- Testing with cost-controlled exploration
- Enforcing per-task budgets in multi-task workflows

The amount is in USD. Fractional values are accepted (e.g., `0.10` = 10 cents).

## Builder API

Use `with_max_budget_usd()` — Accepts a USD amount as `f64`.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_max_budget_usd( 0.50 )
  .with_message( "Keep spend under 50 cents" );
```

## Examples

```bash
# Cap at 10 cents per task
claude --print --max-budget-usd 0.10 "Quick code review"

# Cap at $1 for a complex task
claude --print --max-budget-usd 1.00 --effort high "Security audit"

# Batch processing with per-task cap
for file in src/*.rs; do
  claude --print --max-budget-usd 0.05 "Review $file"
done
```

## Notes

- When budget is exhausted, the response is truncated — this may result in incomplete output
- Combine with `--effort` to control how much thinking is spent before the cap
- Only works with `--print`; interactive sessions have no budget cap mechanism
- Cost calculation is approximate; actual charges may differ slightly from the cap
