# run

Shell scripts for workspace container operations.

| File | Responsibility |
|------|----------------|
| `docker` | Thin wrapper: delegates to docker-run with workspace config. |
| `docker-run` | Universal Docker runner: reads runbox.yml, builds and runs containers. |
| `plugins.sh` | Workspace plugin definitions: sourced by docker-run; remove for plugin-free operation. |
| `runbox.yml` | Workspace Docker config: image, build args, plugins, test script path. |
| `test` | Run workspace tests locally; also called by docker-run inside container. |
| `docs/` | Variability analysis: runbox infrastructure parameters and plugins. |
