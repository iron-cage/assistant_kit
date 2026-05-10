# python_lib

Minimal Python library demonstrating universal runbox usage for a non-Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/example_lib/` | Library source |
| `tests/` | pytest test suite |
| `run/` | Runbox integration: config, dockerfile, wrapper, plugins |
| `pyproject.toml` | Project metadata and dev dependencies |

## Quick Start

```bash
./run/runbox .build          # build container image
./run/runbox .test.offline   # run tests (uses seeded .venv volume)
./run/runbox .list           # list tests
./run/runbox .shell          # interactive shell
```
