# print

Run Claude in non-interactive print mode: send the prompt, print the response, then exit.

### Forms

| | Value |
|-|-------|
| CLI Flag | `-p` / `--print` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off` (auto-on when a positional message is supplied)

### Since

pre-v1.0 (unverified)

### Description

Run Claude in non-interactive print mode: send the prompt, receive the response on stdout, then exit. Automatically enabled when a positional message is given. In print mode the workspace trust dialog is skipped — only use in trusted directories. Combines with `--output-format` to control response encoding.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [044_output_format.md](044_output_format.md) | Response encoding (text/json/stream-json) |
| doc | [043_no_session_persistence.md](043_no_session_persistence.md) | Ephemeral sessions (requires print mode) |
| doc | [026_fallback_model.md](026_fallback_model.md) | Fallback model (only active in print mode) |