# Verb: `test`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `./run/runbox .test`

### Command

```bash
./run/runbox .test
```

Delegates to `run/runbox`, which builds the Docker image if needed and runs the test suite inside the container. The container executes `verb/test.d/l1`, which runs `w3 .test level::3`. Level 3 runs: nextest (all features, warnings-as-errors) + doc tests + clippy (-D warnings). See `CLAUDE.md § Full Verification Commands` for the level breakdown.

### Layers

| Layer | Context | Docker | `CARGO_NET_OFFLINE` | Default |
|-------|---------|--------|---------------------|---------|
| runbox | host → Docker via `run/runbox .test` | yes | yes (inside container) | yes — no `VERB_LAYER` set |
| `l0` | host-native | no | no | no — `VERB_LAYER=l0` only |
| `l1` | container-internal | n/a | yes | no — `VERB_LAYER=l1` (set by runbox-run) |

### Notes

`verb/test` (default, no `VERB_LAYER`) calls `../run/runbox .test` — Docker is the default. `run/runbox` handles image management and mounts, then executes `verb/test.d/l1` inside the container.

`verb/test.d/l1` is the container-internal implementation: runs `w3 .test level::3` with `CARGO_NET_OFFLINE=true` and `NO_COLOR=1`. It is the `test_script` entry point called by `runbox-run`.

`verb/test.d/l0` is the host-native layer: runs `w3 .test level::3` directly on the host without Docker. Use `VERB_LAYER=l0` to invoke, or call `./verb/test.d/l0` directly.

`--dry-run` prints `./run/runbox .test` and exits 0 — no tests run.

### Example

```bash
# Default — Docker via runbox (any module):
./verb/test               # runs: ./run/runbox .test  →  container  →  w3 .test level::3
./verb/test --dry-run     # prints: ./run/runbox .test

# Host-native (explicit):
VERB_LAYER=l0 ./verb/test     # host: w3 .test level::3 directly
./verb/test.d/l0              # same, called directly

# Container-internal (set by runbox-run via VERB_LAYER=l1):
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
if [[ "${1:-}" == "--dry-run" ]]; then echo "./run/runbox .test"; exit 0; fi
exec "$(dirname "${BASH_SOURCE[0]}")/../run/runbox" .test
```

`verb/test.d/l1` (universal — identical across all cargo modules; entered via `VERB_LAYER=l1`):
```bash
#!/usr/bin/env bash
# l1 — bare test execution (VERB_LAYER=l1); runs w3 directly.
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "w3 .test level::3"; exit 0; fi
export CARGO_NET_OFFLINE=true   # deps pre-cooked; no registry update inside container
export NO_COLOR=1               # prevent nextest PTY progress bar (invisible via wrun_core)
exec w3 .test level::3
```

`verb/test.d/l0` (universal — host-native; entered via `VERB_LAYER=l0`):
```bash
#!/usr/bin/env bash
# l0 — host-native test execution; runs w3 .test level::3 directly on the host.
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "w3 .test level::3"; exit 0; fi
exec w3 .test level::3
```

Each module's `run/runbox.yml` sets `test_script: module/<name>/verb/test.d/l1` — the container entry point is `l1` directly. `runbox-run` injects `VERB_LAYER=l1` as a safety guard, but the explicit path makes container execution robust without relying on it.
