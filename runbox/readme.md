# run

Shell scripts for workspace container operations.

| File | Responsibility |
|------|----------------|
| `runbox` | Canonical project wrapper: auto-discovers container runner; copy verbatim for new projects. |
| `container runner` | Universal Docker runner: reads container.yml, builds and runs containers. |
| `verb-run` | Universal verb dispatcher: resolves verb file and execs with VERB_LAYER in environment. |
| `plugins.sh` | Workspace plugin definitions: sourced by container runner; remove for plugin-free operation. |
| `runbox.yml` | Workspace Docker config: image, build args, plugins, test script path. |
| `runbox.dockerfile` | Parameterised multi-stage Docker image; built by container runner. |
| `onboarding.md` | New project integration guide: how to add container support to any ecosystem. |
| `docs/` | Variability analysis: container test infrastructure parameters and plugins. |
| `tests/` | Regression tests for the container runner shell script infrastructure. |
| `example/` | Working integration examples: one per ecosystem (rust, nodejs, python). |
