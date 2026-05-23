# betas

Sends additional beta feature headers with API requests to opt into experimental features.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--betas <betas...>` |
| Env Var | — |
| Config Key | — |

### Type

string[] (space-separated beta header names)

### Default

—

### Description

Sends additional beta feature headers with API requests. Only effective when using API key authentication (not browser OAuth). Beta headers opt into experimental Anthropic API features before they are generally available. The accepted values depend on current Anthropic beta offerings. Use only when specifically required by a beta feature's documentation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |