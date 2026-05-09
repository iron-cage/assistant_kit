# run

Shell scripts for workspace container operations.

| File | Responsibility |
|------|----------------|
| `docker` | Thin wrapper: delegates to docker-run with workspace config. |
| `docker-run` | Universal Docker runner: reads runbox.yml, builds and runs containers. |
| `runbox.yml` | Workspace Docker config: image, build args, plugins, test script path. |
| `test` | Run workspace tests locally; also called by docker-run inside container. |
