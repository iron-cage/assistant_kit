# Parameter: `workspace_root`

- **Status:** ✅ Configured — via `runbox.yml`; default: parent of `runbox-run` script directory
- **Current State:** (unset — defaults to `SCRIPT_DIR/..` = `dev/`)
- **Where It Flows:** `runbox.yml workspace_root:` → resolved relative to `CONFIG_DIR` → `WORKSPACE_ROOT` in `runbox-run` → Docker build context (`docker build ... $WORKSPACE_ROOT`)

### Notes

Required only for standalone projects where the runbox config lives outside the default workspace root. When unset, `WORKSPACE_ROOT` defaults to the parent of the `runbox-run` script — correct for the Rust workspace where `run/runbox-run` sits one level inside the project root (`dev/run/` → `dev/`).

Standalone projects (e.g. `example/python_lib/`) set `workspace_root: ..` so `WORKSPACE_ROOT` resolves to the project root regardless of where the universal `runbox-run` binary lives. The path is always resolved relative to `CONFIG_DIR` (the directory containing the config file) — not relative to `SCRIPT_DIR` — making it portable.

### Example

Python standalone project with config at `example/python_lib/run/runbox.yml`:
```yaml
workspace_root: ..
```
`runbox-run` resolves this to `$CONFIG_DIR/..` = `example/python_lib/`. Docker build context becomes the project root, so `COPY . .` in the dockerfile copies the full Python project into the container.
