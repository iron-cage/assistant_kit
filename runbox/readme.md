# runbox — owning container config

The workspace's owning config for the `runbox` container engine (the engine
itself is not vendored here — it ships with `family_dev` and is installed
globally as `runbox` on `PATH`; the engine derives the workspace root from
this config's location, never from its own).

One shared image (`claude_storage_core_test`) serves the whole workspace: this
config declares it, `.build` bakes it, and every module's `verb/test` consumes
it via `runbox .live` with the module's own `test.d/l1` as payload.

| Path | Responsibility |
|------|----------------|
| `runbox.yml` | Owning config: image, user, script, mounts, plugins, build inputs. |
| `runbox.dockerfile.template` | Dockerfile template rendered by `.build` (copied from `family_dev`, with the nextest aarch64 install patched to use the get.nexte.st CDN; this repo's `render_sha` fingerprint is its own). |

Common invocations (any directory inside the workspace):

```bash
runbox .build          # bake/refresh the shared image
runbox .live           # full workspace suite (config script: verb/test.d/l1)
runbox .shell          # interactive shell in the test environment
runbox .clean          # remove aged runbox-owned debris
runbox .help           # engine reference; runbox .live.help etc. per command
```
