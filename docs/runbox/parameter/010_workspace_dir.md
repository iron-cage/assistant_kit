# Parameter: `workspace_dir`

- **Status:** ✅ Configured — via `runbox.yml`; default: `/workspace`
- **Current State:** `/workspace`
- **Where It Flows:** `runbox.yml workspace_dir:` → `--build-arg WORKSPACE_DIR` → `WORKDIR`, `ENV HOME`, `COPY`, and `RUN` paths in planner/cook/test stages; `$WORKSPACE_DIR` in all `docker-run` volume mounts and the test entrypoint path

### Notes

Propagates through both files at build time (4 dockerfile stages) and runtime (6 docker-run call sites). Dockerfile paths are baked into the image on `.build`; docker-run picks up the runtime paths immediately from `runbox.yml`. The `run/test` scripts inside the container are not auto-updated — they still contain hard-coded paths and must be updated manually if `workspace_dir` changes.

### Example

Moving the container workspace to `/app`:
```yaml
workspace_dir: /app
```
`docker-run` passes `--build-arg WORKSPACE_DIR=/app` → dockerfile bakes `WORKDIR /app`, `ENV HOME=/app`, `COPY --from=cook /app/target /app/target`, `RUN mkdir /app/target_seed`, and `chown`/`chmod` on `/app`. Docker-run uses `/app` in all `-v` mounts (`${IMAGE}_target:/app/target`) and the test entrypoint (`/app/$TEST_SCRIPT`). Requires `.build` before the next `.test`; also requires updating `run/test` and any other scripts that reference `/workspace` directly.
