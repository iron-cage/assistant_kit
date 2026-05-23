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

### Description

Run Claude in non-interactive print mode: send the prompt, receive the response on stdout, then exit. Automatically enabled when a positional message is given. In print mode the workspace trust dialog is skipped — only use in trusted directories. Combines with `--output-format` to control response encoding.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |