# Parameter: `dockerfile`

- **Status:** ✅ Configured — via `runbox.yml`; default: `runbox/runbox.dockerfile` (relative to `SCRIPT_DIR`)
- **Current State:** `runbox/runbox.dockerfile` (cargo-chef 4-stage Rust build)
- **Where It Flows:** `runbox.yml dockerfile:` → `$DOCKERFILE` in `runbox-run` → `$CONTAINER_CMD build -f $DOCKERFILE`

### Notes

Controls plugin 003 (dep cache strategy) and plugin 006 (offline runner) simultaneously — the dockerfile defines both via its multi-stage build and its baked `CMD`. Absent from `runbox.yml` → `runbox-run` defaults to `$SCRIPT_DIR/runbox.dockerfile`. A compatible `cache_dir` param must match what the new dockerfile seeds (see `012_cache_dir.md`).

**Hash scope:** `$DOCKERFILE` is one of five inputs to `_content_hash()` in `runbox-run`. Changing the active Dockerfile (or its contents) automatically invalidates the cached container image and triggers a rebuild on the next `.test` or `.build` invocation. This prevents stale images from surviving Dockerfile edits. See BUG-002 in `agent_kit/task/runbox/bug/` for the history of this extension.

### Example

Switching to a single-stage build (no cargo-chef, simpler for small workspaces):
```yaml
dockerfile: runbox/runbox-simple.dockerfile
```
`runbox-run` passes `-f runbox/runbox-simple.dockerfile` to `docker build`. The new dockerfile defines its own dep cache strategy and `CMD` — plugins 003 and 006 are both replaced in one step. Requires `.build` before the next `.test.offline`.
