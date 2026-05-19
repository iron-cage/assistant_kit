# python

Minimal Python library proving universal runbox usage for a non-Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/example_lib/` | Library source and `__main__` demo entry point. |
| `tests/` | pytest test suite (3 tests). |
| `runbox/` | Runbox integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `pyproject.toml` | Project metadata and dev dependencies (pytest, ruff). |

## Quick Start

```bash
./runbox/runbox .build          # build container image
./runbox/runbox .test           # run tests
./runbox/runbox .lint           # run ruff linter
./runbox/runbox .run            # run demo (prints: add(2, 3) = 5)
./runbox/runbox .test.offline   # offline tests (uses seeded .venv volume)
./runbox/runbox .list           # list tests
./runbox/runbox .shell          # interactive shell
```
