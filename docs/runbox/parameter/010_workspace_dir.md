# Parameter: `workspace_dir`

- **Status:** 🔒 Hardcoded — in both `runbox.dockerfile` and `docker-run`
- **Current State:** `/workspace`
- **Where It Flows:** `WORKDIR /workspace`, all volume mount paths, `ENV HOME /workspace`, `docker run -w /workspace`

### Notes

Split across both dockerfile and `docker-run` — both must change together. Baking it into one place is blocked by the need for it at both build time and run time.

### Example

`/workspace` is hardcoded in both files:

`runbox.dockerfile`:
```dockerfile
WORKDIR /workspace
ENV HOME=/workspace
COPY --from=cook /workspace/target /workspace/target
RUN mkdir /workspace/target_seed
```
`docker-run`:
```bash
-v "${volume}:/workspace/target"       # build cache mount
-v "${volume}:/workspace/target_seed"  # seeding mount
"/workspace/$TEST_SCRIPT"             # test entrypoint
```
Changing to `/app` requires a coordinated search-and-replace across both files. A partial change (dockerfile says `/workspace`, docker-run mounts at `/app/target`) causes the build-cache volume to miss: cargo sees an empty `target/` and recompiles all workspace crates from scratch on every run.
