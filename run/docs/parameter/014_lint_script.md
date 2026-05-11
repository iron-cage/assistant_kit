# Parameter: `lint_script`

- **Status:** ✅ Configured — via `runbox.yml`; optional (linting disabled if absent)
- **Current State:** (unset for Rust workspace modules; set in standalone example projects)
- **Where It Flows:** `runbox.yml lint_script:` → `LINT_SCRIPT` in `runbox-run` → `docker run /workspace/$LINT_SCRIPT` executed by `cmd_lint()`

### Notes

Optional script path (relative to `/workspace`) executed by `.lint`. When absent, `.lint` exits with an error rather than silently succeeding. Pre-flight validation confirms the file exists on the host before attempting a container run — a missing file on the host indicates a wrong path in `runbox.yml`, not a stale image.

`_ensure_image()` probes for the script inside the image (alongside `test_script` and `run_script`) in a single container run. If any configured script is absent from the image, the image is automatically rebuilt rather than emitting a cryptic "not found" error.

Standalone projects (e.g. `runbox/example/python/`) set this to `run/lint` — a thin shell script that invokes the ecosystem linter at an absolute container path (`/workspace/.venv/bin/ruff check /workspace/src/`).

### Example

Python standalone project:
```yaml
lint_script: run/lint
```
`run/lint` (works inside the container and natively on the host):
```bash
#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../.venv/bin/ruff" check --no-cache "$SCRIPT_DIR/../src/" "$SCRIPT_DIR/../tests/"
```
Node.js standalone project:
```yaml
lint_script: run/lint
```
`run/lint` (works inside the container and natively on the host):
```bash
#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../node_modules/.bin/eslint" "$SCRIPT_DIR/../src/" "$SCRIPT_DIR/../tests/"
```
Rust standalone project:
```yaml
lint_script: run/lint
```
`run/lint` (works inside the container and natively on the host):
```bash
#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec cargo clippy --manifest-path "$SCRIPT_DIR/../Cargo.toml" --all-targets -- -D warnings
```
