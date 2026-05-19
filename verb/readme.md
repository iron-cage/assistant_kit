# verb

Universal Action Protocol implementation across workspace modules.

| File/Directory | Responsibility |
|----------------|----------------|
| `docs/` | Per-verb reference for all 8 `do` protocol verbs |
| `Dockerfile` | Container image definition for the workspace test environment. |
| `plugins.sh` | Plugin installation script sourced during Docker image build. |
| `verb-run` | Framework runner: builds image from `verb.yml` and runs verb layer scripts inside Docker. |
| `test` | Dispatcher: run workspace test suite; delegates to `test.d/` layer by `VERB_LAYER`. |
| `test.d/` | Layer directory: `docker` (Docker via verb-run, default), `l0` (host-native), `l1` (container-internal). |
