# Plugin: Build cache persistence

- **Status:** 🔒 Hardcoded — mechanism split across `run/runbox.dockerfile` and `runbox-run`
- **Controls:** How compiled artifacts survive between `docker run` invocations
- **Mechanism:** Named Docker volume seeded from image via `cp -a` in `_ensure_build_cache`; `target_seed/` directory baked into dockerfile at cook stage

### Notes

Always volume+seed strategy. Strategy is split across dockerfile (seed) and runbox-run (volume management); both must change together.

### Example

First `./run/runbox .test.offline` invocation:
1. `_ensure_build_cache`: `docker volume inspect workspace_test_target` → not found
2. Creates volume; seeds it: `docker run -v workspace_test_target:/workspace/target_seed workspace_test bash -c "cp -a /workspace/target/. /workspace/target_seed/ && chmod -R a+rwX /workspace/target_seed/"`
3. `cmd_test_offline` runs: `docker run -v workspace_test_target:/workspace/target workspace_test cargo nextest run --workspace --filter-expr "..."`
4. Cargo finds a pre-populated `target/` — only changed workspace crates recompile; unchanged artifacts from the cook stage are reused immediately

After `./run/runbox .build` (image rebuild):
- `cmd_build()` deletes `workspace_test_target` and any plugin-provided volumes (`workspace_test_plugin_targets` when `run/plugins.sh` is active, via `_plugin_build_volumes`)
- Next `_ensure_build_cache` re-seeds from the new image's freshly-cooked `target/`

The seed lives at `/workspace/target` in the image (deposited by the cook stage). `/workspace/target_seed` is a temporary mount point used only during seeding — absent from normal test runs.
