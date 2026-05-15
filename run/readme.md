# run

Shell scripts for workspace container operations.

| File | Responsibility |
|------|----------------|
| `runbox` | Canonical project wrapper: auto-discovers runbox-run; copy verbatim for new projects. |
| `runbox-run` | Universal Docker runner: reads runbox.yml, builds and runs containers. |
| `verb-run` | Universal verb dispatcher: resolves verb file and execs with VERB_LAYER in environment. |
| `plugins.sh` | Workspace plugin definitions: sourced by runbox-run; remove for plugin-free operation. |
| `runbox.yml` | Workspace Docker config: image, build args, plugins, test script path. |
| `runbox.dockerfile` | Parameterised multi-stage Docker image; built by runbox-run. |
| `onboarding.md` | New project integration guide: how to add runbox to any ecosystem. |
| `docs/` | Variability analysis: runbox infrastructure parameters and plugins. |
