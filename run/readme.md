# run

Shell scripts for workspace container operations.

| File | Responsibility |
|------|----------------|
| `runbox` | Thin wrapper: delegates to runbox-run with workspace config. |
| `runbox-run` | Universal Docker runner: reads runbox.yml, builds and runs containers. |
| `plugins.sh` | Workspace plugin definitions: sourced by runbox-run; remove for plugin-free operation. |
| `runbox.yml` | Workspace Docker config: image, build args, plugins, test script path. |
| `test` | Run workspace tests locally; also called by runbox-run inside container. |
| `runbox.dockerfile` | Parameterised multi-stage Docker image; built by runbox-run. |
| `docs/` | Variability analysis: runbox infrastructure parameters and plugins. |
