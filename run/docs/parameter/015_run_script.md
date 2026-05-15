# Parameter: `run_script`

- **Status:** ✅ Configured — via `runbox.yml`; optional (entry-point execution disabled if absent)
- **Current State:** (unset for Rust workspace modules; set in standalone example projects)
- **Where It Flows:** `runbox.yml run_script:` → `RUN_SCRIPT` in `runbox-run` → `docker run /workspace/$RUN_SCRIPT` executed by `cmd_run()`

### Notes

Optional script path (relative to `/workspace`) executed by `.run`. When absent, `.run` exits with an error rather than silently succeeding. Pre-flight validation confirms the file exists on the host before attempting a container run — a missing file on the host indicates a wrong path in `runbox.yml`, not a stale image.

`_ensure_image()` probes for the script inside the image (alongside `test_script` and `lint_script`) in a single container run. If any configured script is absent from the image, the image is automatically rebuilt rather than emitting a cryptic "not found" error.

Library-only projects typically omit `run_script` entirely. Binary projects (Python modules, Node.js services, Rust binaries) set this to `verb/run` — a self-dispatching dispatcher that routes to `verb/run.d/l1` inside the container (via `VERB_LAYER=l1`) or delegates to runbox via `verb/run.d/l2` when invoked on the host.

### Example

`verb/run` dispatcher (identical for all ecosystems):
```yaml
run_script: verb/run
```
```bash
#!/usr/bin/env bash
# run — execute entry point; dispatches by VERB_LAYER to run.d/ layer.
set -euo pipefail
DIR="$(dirname "${BASH_SOURCE[0]}")/run.d"
LAYER="${VERB_LAYER:-}"
[[ -n "$LAYER" && -f "$DIR/$LAYER" ]] && exec "$DIR/$LAYER" "$@"
exec "$DIR/l2" "$@"
```

`verb/run.d/l1` — container execution layer (ecosystem-specific):

Python:
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../../.venv/bin/python" -m example_lib
```
Node.js:
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec node "$SCRIPT_DIR/../../src/main.js"
```
Rust (pre-built binary in image):
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../../target/debug/rust_example"
```

`verb/run.d/l2` — host orchestration layer (identical for all ecosystems):
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "./run/runbox .run"; exit 0; fi
exec ./run/runbox .run
```
