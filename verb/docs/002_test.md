# Verb: `test`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `w3 .test level::3`

### Command

```bash
w3 .test level::3
```

Delegates to `w3`, the workspace-level test runner. Level 3 runs: nextest (all features, warnings-as-errors) + doc tests + clippy (-D warnings). See `CLAUDE.md § Full Verification Commands` for the level breakdown.

### Layers

| Layer | Context | Docker required | `CARGO_NET_OFFLINE` | Default |
|-------|---------|-----------------|---------------------|---------|
| `l0` | host | no | no | no — `VERB_LAYER=l0` |
| `l1` | container (called by runbox-run) | n/a | yes | no — `VERB_LAYER=l1` |
| `l2` | host | yes | n/a | yes — no `VERB_LAYER` set |

### Notes

The `test` verb is **identical across all modules** — the command does not vary by module name because `w3 .test level::3` already scopes itself to the current workspace context. This makes `verb/test` the single source of truth for what "run tests" means for any module.

`verb/test.d/l1` serves as the runbox `test_script`: `cmd_test()` in `runbox-run` mounts and executes `/workspace/module/<name>/verb/test.d/l1` directly inside the container — bypassing the dispatcher entirely. The `_ensure_image()` probe checks for this file inside the image before running; a stale image (missing the script) triggers an automatic rebuild. See `run/docs/parameter/005_test_script.md`.

`verb/test.d/l0` is the host-native alternative to `l2`: runs `w3 .test level::3` directly on the host without Docker or runbox. Does not set `CARGO_NET_OFFLINE` (cargo has full network access) and does not force `NO_COLOR` (no PTY wrapping on a real terminal). Invoke via `VERB_LAYER=l0 ./verb/test` or directly as `./verb/test.d/l0`. Universal — identical across all cargo modules alongside `l1` and `l2`.

`--dry-run` prints `w3 .test level::3` and exits 0 — no tests run.

### Example

```bash
# Default — runbox-orchestrated (any module):
./verb/test               # runs: w3 .test level::3 via Docker
./verb/test --dry-run     # prints: ./run/runbox .test

# Host-native — no Docker (any module):
VERB_LAYER=l0 ./verb/test     # runs: w3 .test level::3 on host
./verb/test.d/l0              # same, called directly
./verb/test.d/l0 --dry-run    # prints: w3 .test level::3
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
exec "$DIR/l2" "$@"
```

`verb/test.d/l1` (universal — identical across all cargo modules; entered via `VERB_LAYER=l1`):
```bash
#!/usr/bin/env bash
# l1 — bare test execution (VERB_LAYER=l1); runs w3 directly.
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "w3 .test level::3"; exit 0; fi
exec w3 .test level::3
```

`verb/test.d/l2` (universal — identical across all cargo modules; default host-side invocation):
```bash
#!/usr/bin/env bash
# l2 — host orchestration; delegates to runbox (Docker-orchestrated execution).
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "./run/runbox .test"; exit 0; fi
exec ./run/runbox .test
```

`verb/test.d/l0` (universal — identical across all cargo modules; entered via `VERB_LAYER=l0`):
```bash
#!/usr/bin/env bash
# l0 — host-native test execution; runs w3 .test level::3 directly on the host.
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "w3 .test level::3"; exit 0; fi
exec w3 .test level::3
```

Each module's `run/runbox.yml` sets `test_script: module/<name>/verb/test.d/l1` — the container entry point is the l1 layer directly, with no dispatcher involved. `runbox-run` still injects `VERB_LAYER=l1` as a safety guard, but the script path makes container execution robust without relying on it.
