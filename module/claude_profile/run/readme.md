# run

Shell scripts for `claude_profile` container operations.

| File | Responsibility |
|------|----------------|
| `docker` | Thin wrapper: delegates to workspace docker-run with profile config. |
| `docker.yml` | Profile Docker config: image, build args, mounts, test script path. |
| `test` | Run profile tests locally; also called by docker-run inside container. |
