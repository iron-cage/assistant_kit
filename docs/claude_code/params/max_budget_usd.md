# max_budget_usd

Sets a hard cap on API spend (in US dollars) for this invocation.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--max-budget-usd <amount>` |
| Env Var | — |
| Config Key | — |

### Type

float (USD)

### Default

—

### Description

Sets a hard cap on API spend for this invocation. Claude stops making API calls once the accumulated cost exceeds the specified amount in US dollars. Only effective in print mode. Useful for cost-controlled automation where unbounded API spend is unacceptable. The session terminates with an error when the budget is exhausted.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |