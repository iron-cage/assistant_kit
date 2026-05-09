# Plugin: Build cache persistence

- **Status:** 🔒 Hardcoded — mechanism split across `runbox.dockerfile` and `docker-run`
- **Controls:** How compiled artifacts survive between `docker run` invocations
- **Mechanism:** Named Docker volume seeded from image via `cp -a` in `_ensure_build_cache`; `target_seed/` directory baked into dockerfile at cook stage

### Notes

Always volume+seed strategy. Strategy is split across dockerfile (seed) and docker-run (volume management); both must change together.
