# run

Shell scripts for `claude_storage` container operations.

| File | Responsibility |
|------|----------------|
| `docker` | Thin wrapper: delegates to workspace docker-run with storage config. |
| `runbox.yml` | Storage Docker config: image, build args, plugins, test script path. |
