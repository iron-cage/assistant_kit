# Parameter: `dockerfile`

- **Status:** ✅ Configured — via `runbox.yml`; default: `run/runbox.dockerfile` (relative to `SCRIPT_DIR`)
- **Current State:** `run/runbox.dockerfile` (cargo-chef 4-stage Rust build)
- **Where It Flows:** `runbox.yml dockerfile:` → `$DOCKERFILE` in `runbox-run` → `$CONTAINER_CMD build -f $DOCKERFILE`

### Notes

Controls plugin 003 (dep cache strategy) and plugin 006 (offline runner) simultaneously — the dockerfile defines both via its multi-stage build and its baked `CMD`. Absent from `runbox.yml` → `runbox-run` defaults to `$SCRIPT_DIR/runbox.dockerfile`. A compatible `cache_dir` param must match what the new dockerfile seeds (see `012_cache_dir.md`).

### Example

Switching to a single-stage build (no cargo-chef, simpler for small workspaces):
```yaml
dockerfile: run/runbox-simple.dockerfile
```
`runbox-run` passes `-f run/runbox-simple.dockerfile` to `docker build`. The new dockerfile defines its own dep cache strategy and `CMD` — plugins 003 and 006 are both replaced in one step. Requires `.build` before the next `.test.offline`.
