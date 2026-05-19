# nodejs

Minimal Node.js library proving universal runbox usage for a non-Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/index.js` | Library source. |
| `src/main.js` | Demo entry point (prints result of `add(2, 3)`). |
| `tests/` | Test suite using Node.js built-in test runner (3 tests). |
| `runbox/` | Runbox integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `package.json` | Project metadata and dev dependencies (eslint). |
| `.eslintrc.json` | ESLint config: node env, recommended rules. |

## Quick Start

```bash
./runbox/runbox .build          # build container image
./runbox/runbox .test           # run tests
./runbox/runbox .lint           # run eslint
./runbox/runbox .run            # run demo (prints: add(2, 3) = 5)
./runbox/runbox .test.offline   # offline tests (uses seeded node_modules volume)
./runbox/runbox .list           # list tests
./runbox/runbox .shell          # interactive shell
```
