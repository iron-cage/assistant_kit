# Parameter: `lint_script`

- **Status:** ✅ Configured — via `runbox.yml`; optional (linting disabled if absent)
- **Current State:** (unset for Rust workspace modules; set in standalone example projects)
- **Where It Flows:** `runbox.yml lint_script:` → `LINT_SCRIPT` in `runbox-run` → `docker run /workspace/$LINT_SCRIPT` executed by `cmd_lint()`

### Notes

Optional script path (relative to `/workspace`) executed by `.lint`. When absent, `.lint` exits with an error rather than silently succeeding. Pre-flight validation confirms the file exists on the host before attempting a container run — a missing file on the host indicates a wrong path in `runbox.yml`, not a stale image.

`_ensure_image()` probes for the script inside the image (alongside `test_script` and `run_script`) in a single container run. If any configured script is absent from the image, the image is automatically rebuilt rather than emitting a cryptic "not found" error.

Standalone projects (e.g. `runbox/example/python/`) set this to `verb/lint` — a self-dispatching dispatcher that routes to `verb/lint.d/l1` inside the container (via `VERB_LAYER=l1`) or delegates to runbox via `verb/lint.d/l2` when invoked on the host.

### Example

`verb/lint` dispatcher (identical for all ecosystems):
```yaml
lint_script: verb/lint
```
```bash
#!/usr/bin/env bash
# lint — run linter; dispatches by VERB_LAYER to lint.d/ layer.
set -euo pipefail
DIR="$(dirname "${BASH_SOURCE[0]}")/lint.d"
LAYER="${VERB_LAYER:-}"
[[ -n "$LAYER" && -f "$DIR/$LAYER" ]] && exec "$DIR/$LAYER" "$@"
exec "$DIR/l2" "$@"
```

`verb/lint.d/l1` — container execution layer (ecosystem-specific):

Python:
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../../.venv/bin/ruff" check --no-cache "$SCRIPT_DIR/../../src/" "$SCRIPT_DIR/../../tests/"
```
Node.js:
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../../node_modules/.bin/eslint" "$SCRIPT_DIR/../../src/" "$SCRIPT_DIR/../../tests/"
```
Rust:
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec cargo clippy --manifest-path "$SCRIPT_DIR/../../Cargo.toml" --all-targets -- -D warnings
```

`verb/lint.d/l2` — host orchestration layer (identical for all ecosystems):
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/../.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "./run/runbox .lint"; exit 0; fi
exec ./run/runbox .lint
```
