# Parameter: `cache_dir`

- **Status:** ✅ Configured — via `runbox.yml`; default: `target`
- **Current State:** `target` (Rust/cargo build output directory)
- **Where It Flows:** `runbox.yml cache_dir:` → `$CACHE_DIR` in `runbox-run` → volume name `${IMAGE}_$CACHE_DIR`; container mount `$WORKSPACE_DIR/$CACHE_DIR`; seed mount `$WORKSPACE_DIR/${CACHE_DIR}_seed`

### Notes

Controls plugin 004 (build cache persistence) without touching `runbox-run` logic. Three derived values all follow the param: the named Docker volume, the runtime mount path, and the seeding mount point. A compatible dockerfile must bake `${CACHE_DIR}_seed/` as an empty directory so Docker initialises the volume with the right ownership on first mount. The default Rust dockerfile bakes `target_seed/` — no `runbox.yml` entry needed for Rust workspaces.

### Example

Node ecosystem using `node_modules` as the artifact dir:
```yaml
dockerfile: run/runbox-node.dockerfile
cache_dir: node_modules
```
`runbox-run` manages volume `workspace_test_node_modules`, mounts it at `/workspace/node_modules` for `.test.offline` and `.shell`, and seeds it via `cp -a /workspace/node_modules/. /workspace/node_modules_seed/` on first run. Requires the Node dockerfile to bake `/workspace/node_modules_seed/` at the correct path.
