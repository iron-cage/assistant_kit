# Verb: `lint`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `cargo clippy -p <module> --all-features -- -D warnings`

### Command

```bash
cargo clippy -p <module> --all-features -- -D warnings
```

Runs clippy on the target module with all features enabled and warnings promoted to errors.

### Notes

`-D warnings` enforces zero-warning policy — any clippy warning fails the verb. This matches the RUSTFLAGS used in the test suite (level::1+), keeping lint and test behaviour consistent.

`--all-features` ensures feature-gated code paths are also linted, preventing feature-specific warnings from hiding until CI.

`lint` is a subset of what `test` runs: `w3 .test level::3` includes clippy. `lint` exists as a standalone verb for rapid feedback during development without running the full test suite.

`--dry-run` emits the exact command and exits 0 — no analysis runs.

### Example

```bash
# claude_profile
./verb/lint              # runs: cargo clippy -p claude_profile --all-features -- -D warnings
./verb/lint --dry-run    # prints: cargo clippy -p claude_profile --all-features -- -D warnings

# claude_runner_core (library module — same pattern)
./verb/lint              # runs: cargo clippy -p claude_runner_core --all-features -- -D warnings
```

`verb/lint` dispatcher (universal — identical across all cargo modules):
```bash
#!/usr/bin/env bash
# lint — run linter; dispatches by VERB_LAYER to lint.d/ layer.
set -euo pipefail
DIR="$(dirname "${BASH_SOURCE[0]}")/lint.d"
LAYER="${VERB_LAYER:-}"
[[ -n "$LAYER" && -f "$DIR/$LAYER" ]] && exec "$DIR/$LAYER" "$@"
exec "$DIR/l2" "$@"
```

`verb/lint.d/l1` (module-specific, entered via `VERB_LAYER=l1`; example: `claude_profile`):
```bash
#!/usr/bin/env bash
# l1 — run cargo clippy directly (cargo ecosystem).
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "cargo clippy -p claude_profile --all-features -- -D warnings"; exit 0; fi
exec cargo clippy -p claude_profile --all-features -- -D warnings
```

`verb/lint.d/l2` (universal — identical across all cargo modules; default host-side invocation):
```bash
#!/usr/bin/env bash
# l2 — host orchestration; delegates to runbox (Docker-orchestrated execution).
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "./run/runbox .lint"; exit 0; fi
exec ./run/runbox .lint
```

Each module's `run/runbox.yml` must declare `lint_script: module/<name>/verb/lint.d/l1` — the container entry point is the l1 layer directly. See `run/docs/parameter/014_lint_script.md`. The linter is ecosystem-specific: ruff for Python, eslint for Node.js, cargo clippy for Rust. `verb/lint` is `available` for all project types — linting is universal.
