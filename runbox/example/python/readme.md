# python

Minimal Python library proving universal container runner usage for a non-Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/example_lib/` | Library source and `__main__` demo entry point. |
| `tests/` | pytest test suite (3 tests). |
| `run/` | Container integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `pyproject.toml` | Project metadata and dev dependencies (pytest, ruff). |

## Quick Start

```bash
./verb/.build          # build container image
./verb/.test           # run tests
./verb/.lint           # run ruff linter
./verb/.run            # run demo (prints: add(2, 3) = 5)
./verb/.test.offline   # offline tests (uses seeded .venv volume)
./verb/.list           # list tests
./verb/.shell          # interactive shell
```
