# Verb: `test`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `./run/runbox .test`

### Command

```bash
./run/runbox .test
```

Delegates to runbox, which builds the Docker image if needed and runs the test suite inside the container. The container executes `verb/test.d/l1`, which runs `w3 .test level::3`. Level 3 runs: nextest (all features, warnings-as-errors) + doc tests + clippy (-D warnings). See `CLAUDE.md § Full Verification Commands` for the level breakdown.

### Layers

| Layer | Context | Docker required | `CARGO_NET_OFFLINE` | Default |
|-------|---------|-----------------|---------------------|---------|
| runbox | Docker via `run/runbox .test` | yes | yes (inside container) | yes — no `VERB_LAYER` set |
| `l0` | host | no | no | no — `VERB_LAYER=l0` only |
| `l1` | container (called by runbox-run) | n/a | yes | no — `VERB_LAYER=l1` |

### Notes

The `test` verb is **identical across all modules** — the dispatcher always delegates to the module's own `run/runbox .test`, which scopes the container run to that module's `runbox.yml`.

`verb/test.d/l1` serves as the runbox `test_script`: `cmd_test()` in `runbox-run` mounts and executes `/workspace/module/<name>/verb/test.d/l1` directly inside the container — bypassing the dispatcher entirely. The `_ensure_image()` probe checks for this file inside the image before running; a stale image (missing the script) triggers an automatic rebuild. See `run/docs/parameter/005_test_script.md`.

`verb/test.d/l0` is the host-native layer: runs `w3 .test level::3` directly on the host without Docker or runbox. Use `VERB_LAYER=l0` to invoke it, or call `./verb/test.d/l0` directly. Does not set `CARGO_NET_OFFLINE` (cargo has full network access) and does not force `NO_COLOR` (no PTY wrapping on a real terminal).

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
VERB_LAYER=l1 ./verb/test     # container context: CARGO_NET_OFFLINE=true, NO_COLOR=1
./verb/test.d/l1              # same, called directly
```

Runbox invocation inside Docker:
```bash
docker run --rm \
  -v claude_profile_test_plugin_targets:/tmp/will_test_targets \
  -v /usr/local/bin/w3:/usr/local/bin/w3:ro \
  -v /home/user/.claude:/workspace/.claude:rw \
  claude_profile_test \
  /workspace/module/claude_profile/verb/test.d/l1
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

`verb/test.d/l0` (universal — identical across all cargo modules; host-side invocation via `VERB_LAYER=l0`):
```bash
#!/usr/bin/env bash
# l0 — host-native test execution; runs w3 .test level::3 directly on the host.
# Entered via VERB_LAYER=l0 or called directly as ./verb/test.d/l0.
#
# Differs from l1 (container-internal):
#   - CARGO_NET_OFFLINE is NOT set — host cargo may update registry index
#   - NO_COLOR is NOT set — real terminal controls colour; no PTY wrapping issue
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "w3 .test level::3"; exit 0; fi
exec w3 .test level::3
```

Each module's `run/runbox.yml` sets `test_script: module/<name>/verb/test.d/l1` — the container entry point is the l1 layer directly, with no dispatcher involved. `runbox-run` still injects `VERB_LAYER=l1` as a safety guard, but the script path makes container execution robust without relying on it.
