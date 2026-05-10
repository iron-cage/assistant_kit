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
`run/lint` inside the container:
```bash
#!/usr/bin/env bash
exec /workspace/.venv/bin/ruff check /workspace/src/ /workspace/tests/
```
Node.js standalone project:
```yaml
lint_script: run/lint
```
`run/lint` inside the container:
```bash
#!/usr/bin/env bash
exec /workspace/node_modules/.bin/eslint /workspace/src/ /workspace/tests/
```
Rust standalone project:
```yaml
lint_script: run/lint
```
`run/lint` inside the container:
```bash
#!/usr/bin/env bash
exec cargo clippy --manifest-path /workspace/Cargo.toml --all-targets -- -D warnings
```
