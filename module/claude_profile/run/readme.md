# run

Shell scripts for `claude_profile` container operations.

| File | Responsibility |
|------|----------------|
| `docker` | Thin wrapper: delegates to workspace docker-run with profile config. |
| `runbox.yml` | Profile Docker config: image, build args, plugins, test script path. |
