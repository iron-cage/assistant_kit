# api_key

The Anthropic API key used to authenticate all API requests.

### Forms

| | Value |
|-|-------|
| CLI Flag | — (removed from CLI; was `--api-key`) |
| Env Var | `ANTHROPIC_API_KEY` |
| Config Key | — |

### Type

string

### Default

—

### Description

The Anthropic API key used to authenticate requests. Must be set in the environment; the `--api-key` CLI flag was removed from the binary. Without a valid key, Claude Code falls back to browser-based OAuth authentication. For automation and CI, always set this env var explicitly rather than relying on interactive login.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |