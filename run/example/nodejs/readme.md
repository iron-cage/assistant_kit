# nodejs

Minimal Node.js library proving universal runbox usage for a non-Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/index.js` | Library source. |
| `src/main.js` | Demo entry point (prints result of `add(2, 3)`). |
| `tests/` | Test suite using Node.js built-in test runner (3 tests). |
| `run/` | Runbox integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `package.json` | Project metadata and dev dependencies (eslint). |
| `.eslintrc.json` | ESLint config: node env, recommended rules. |

## Quick Start

```bash
./run/runbox .build          # build container image
./run/runbox .test           # run tests
./run/runbox .lint           # run eslint
./run/runbox .run            # run demo (prints: add(2, 3) = 5)
./run/runbox .test.offline   # offline tests (uses seeded node_modules volume)
./run/runbox .list           # list tests
./run/runbox .shell          # interactive shell
```
