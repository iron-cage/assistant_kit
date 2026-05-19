# python

Minimal Python library proving universal runbox usage for a non-Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/example_lib/` | Library source and `__main__` demo entry point. |
| `tests/` | pytest test suite (3 tests). |
| `run/` | Runbox integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `pyproject.toml` | Project metadata and dev dependencies (pytest, ruff). |

## Quick Start

```bash
./run/runbox .build          # build container image
./run/runbox .test           # run tests
./run/runbox .lint           # run ruff linter
./run/runbox .run            # run demo (prints: add(2, 3) = 5)
./run/runbox .test.offline   # offline tests (uses seeded .venv volume)
./run/runbox .list           # list tests
./run/runbox .shell          # interactive shell
```
