# nodejs

Minimal Node.js library proving universal container runner usage for a non-Rust ecosystem.

| Path | Responsibility |
|------|----------------|
| `src/index.js` | Library source. |
| `src/main.js` | Demo entry point (prints result of `add(2, 3)`). |
| `tests/` | Test suite using Node.js built-in test runner (3 tests). |
| `run/` | Container integration: config, dockerfile, wrapper, scripts. |
| `verb/` | Universal Action Protocol: all 8 verbs implemented. |
| `package.json` | Project metadata and dev dependencies (eslint). |
| `.eslintrc.json` | ESLint config: node env, recommended rules. |

## Quick Start

```bash
./verb/.build          # build container image
./verb/.test           # run tests
./verb/.lint           # run eslint
./verb/.run            # run demo (prints: add(2, 3) = 5)
./verb/.test.offline   # offline tests (uses seeded node_modules volume)
./verb/.list           # list tests
./verb/.shell          # interactive shell
```
