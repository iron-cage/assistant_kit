# run

Shell scripts for `claude_storage` container operations.

| File | Responsibility |
|------|----------------|
| `docker` | Thin wrapper: delegates to workspace docker-run with storage config. |
| `docker.yml` | Storage Docker config: image, build args, mounts, test script path. |
| `test` | Run storage tests locally; also called by docker-run inside container. |
