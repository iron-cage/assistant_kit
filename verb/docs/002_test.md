# Verb: `test`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `w3 .test level::3`

### Command

```bash
w3 .test level::3
```

Delegates to `w3`, the workspace-level test runner. Level 3 runs: nextest (all features, warnings-as-errors) + doc tests + clippy (-D warnings). See `CLAUDE.md § Full Verification Commands` for the level breakdown.

### Notes

The `test` verb is **identical across all modules** — the command does not vary by module name because `w3 .test level::3` already scopes itself to the current workspace context. This makes `verb/test` the single source of truth for what "run tests" means for any module.

`verb/test` doubles as the runbox `test_script`: `cmd_test()` in `runbox-run` mounts and executes `/workspace/module/<name>/verb/test` inside the container. The `_ensure_image()` probe checks for this file inside the image before running; a stale image (missing the script) triggers an automatic rebuild. See `run/docs/parameter/005_test_script.md`.

`--dry-run` prints `w3 .test level::3` and exits 0 — no tests run.

### Example

```bash
# Any module (command is identical)
./verb/test               # runs: w3 .test level::3
./verb/test --dry-run     # prints: w3 .test level::3
```

Runbox invocation inside Docker:
```bash
docker run --rm \
  -v claude_profile_test_plugin_targets:/tmp/will_test_targets \
  -v /usr/local/bin/w3:/usr/local/bin/w3:ro \
  -v /home/user/.claude:/workspace/.claude:rw \
  claude_profile_test \
  /workspace/module/claude_profile/verb/test
```

`verb/test` dispatcher (universal — identical across all cargo modules):
```bash
#!/usr/bin/env bash
# test — run full test suite; dispatches by VERB_LAYER to test.d/ layer.
set -euo pipefail
DIR="$(dirname "${BASH_SOURCE[0]}")/test.d"
LAYER="${VERB_LAYER:-}"
[[ -n "$LAYER" && -f "$DIR/$LAYER" ]] && exec "$DIR/$LAYER" "$@"
exec "$DIR/l1" "$@"
```

`verb/test.d/l1` (universal — identical across all cargo modules):
```bash
#!/usr/bin/env bash
# l1 — bare test execution (VERB_LAYER=l1); runs w3 directly.
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "w3 .test level::3"; exit 0; fi
exec w3 .test level::3
```
