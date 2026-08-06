# Verb: `test`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `runbox .live`

### Command

```bash
./verb/test
```

Delegates to the launcher `runbox/runbox` (shared engine, `family_dev/default`) via `.live`, which builds the container image if needed and runs the test suite inside the container. The container executes `verb/test.d/l1`, which runs the cargo gate directly: nextest (all features, warnings-as-errors) + doc tests + clippy (-D warnings).

### Layers

| Layer | Context | Docker | `CARGO_NET_OFFLINE` | Default |
|-------|---------|--------|---------------------|---------|
| runbox | host → container via `runbox/runbox .live` | yes | yes (inside container) | yes — no `VERB_LAYER` set |
| `l0` | host-native — disabled stub, errors out | no | no | no — container-only testing |
| `l1` | container-internal | n/a | yes | no — the config's `script:` entry point |

### Notes

`verb/test` (default, no `VERB_LAYER`) cds to the module directory and calls `../../runbox/runbox .live` — container execution is the default. The engine discovers the module's `runbox/runbox.yml`, handles image management and mounts, then executes `verb/test.d/l1` inside the container.

`verb/test.d/l1` is the container-internal implementation: runs nextest + doc tests + clippy with `CARGO_NET_OFFLINE=true`, `NO_COLOR=1`, and `RUNBOX_CONTAINER=1`. It is the config's `script:` entry point run by the engine.

`verb/test.d/l0` is a disabled stub: host-native test execution is not permitted (container-only testing). It prints an error and exits 1.

`--dry-run` prints `runbox .live` and exits 0 — no tests run.

### Example

```bash
# Default — container via runbox (any module):
./verb/test               # runs: ../../runbox/runbox .live  →  container  →  nextest + doc tests + clippy
./verb/test --dry-run     # prints: runbox .live

# Host-native: disabled — VERB_LAYER=l0 / ./verb/test.d/l0 error out (container-only testing).

# Container-internal (the config's script: entry, VERB_LAYER=l1):
VERB_LAYER=l1 ./verb/test     # container: CARGO_NET_OFFLINE=true, NO_COLOR=1
./verb/test.d/l1              # same, called directly
```

`verb/test` dispatcher (universal — identical across all cargo modules):
```bash
#!/usr/bin/env bash
# test — run full test suite; dispatches by VERB_LAYER to test.d/ layer.
set -euo pipefail
DIR="$(dirname "${BASH_SOURCE[0]}")/test.d"
LAYER="${VERB_LAYER:-}"
[[ -n "$LAYER" && -f "$DIR/$LAYER" ]] && exec "$DIR/$LAYER" "$@"
if [[ "${1:-}" == "--dry-run" ]]; then echo "runbox .live"; exit 0; fi
MODULE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$MODULE_DIR" && exec "$MODULE_DIR/../../runbox/runbox" .live "$@"
```

`verb/test.d/l1` (universal — identical across all cargo modules; entered via `VERB_LAYER=l1`):
```bash
#!/usr/bin/env bash
# l1 — bare test execution (VERB_LAYER=l1); runs the cargo test gate directly.
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "cargo nextest run + doc test + clippy"; exit 0; fi
export CARGO_NET_OFFLINE=true   # deps pre-cooked; no registry update inside container
export NO_COLOR=1               # prevent nextest PTY progress bar (invisible via capture)
export RUNBOX_CONTAINER=1       # container signal for container-only test guards
export RUSTFLAGS="-D warnings"
cargo nextest run --all-features
RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features
exec cargo clippy --all-targets --all-features -- -D warnings
```

`verb/test.d/l0` (disabled stub — container-only testing):
```bash
#!/usr/bin/env bash
# l0 — host-native test execution is disabled.
# Tests must run inside the runbox container.
set -euo pipefail
echo "ERROR: host-native test execution (l0) is disabled." >&2
exit 1
```

Each module's `runbox/runbox.yml` sets `script: module/<name>/verb/test.d/l1` (workspace-relative) — the container entry point is `l1` directly; the engine runs it as the config's `script:` without any dispatcher in between.
