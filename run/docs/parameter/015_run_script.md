# Parameter: `run_script`

- **Status:** ✅ Configured — via `runbox.yml`; optional (entry-point execution disabled if absent)
- **Current State:** (unset for Rust workspace modules; set in standalone example projects)
- **Where It Flows:** `runbox.yml run_script:` → `RUN_SCRIPT` in `runbox-run` → `docker run /workspace/$RUN_SCRIPT` executed by `cmd_run()`

### Notes

Optional script path (relative to `/workspace`) executed by `.run`. When absent, `.run` exits with an error rather than silently succeeding. Pre-flight validation confirms the file exists on the host before attempting a container run — a missing file on the host indicates a wrong path in `runbox.yml`, not a stale image.

`_ensure_image()` probes for the script inside the image (alongside `test_script` and `lint_script`) in a single container run. If any configured script is absent from the image, the image is automatically rebuilt rather than emitting a cryptic "not found" error.

Library-only projects typically omit `run_script` entirely. Binary projects (Python modules, Node.js services, Rust binaries) set this to `run/run` — a thin shell script that invokes the compiled or interpreted entry point at an absolute container path.

### Example

Python standalone project:
```yaml
run_script: run/run
```
`run/run` inside the container:
```bash
#!/usr/bin/env bash
exec /workspace/.venv/bin/python -m example_lib
```
Node.js standalone project:
```yaml
run_script: run/run
```
`run/run` inside the container:
```bash
#!/usr/bin/env bash
exec node /workspace/src/main.js
```
Rust standalone project (pre-built binary in image):
```yaml
run_script: run/run
```
`run/run` inside the container:
```bash
#!/usr/bin/env bash
exec /workspace/target/debug/rust_example
```
