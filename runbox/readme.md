# runbox — owning container config

The workspace's owning config for the `runbox` container engine (the engine
itself is not vendored here — it ships with `family_dev` and is installed
globally as `runbox` on `PATH`; the engine derives the workspace root from
this config's location, never from its own). The retired in-tree stack —
embedded Rust engine crate (`module/runbox/`), flat-YAML `runbox-run` runner,
per-module walk-up wrappers, `plugins.sh` hook — was removed on migration
(TSK-1436 pilot, 2026-08-06); configs now use the engine's nested schema.

One shared image (`assistant_kit_claude_profile_test`) serves the whole workspace: this
config declares it, `.build` bakes it, and every module's `verb/test` consumes
it via `runbox .live` with the module's own `test.d/l1` as payload. The tag is
unique to this checkout — sibling `assistant_kit` checkouts under `yrd_core/`
each declare their own, so a rebake here never swaps the image out from under
an unrelated checkout.

| Path | Responsibility |
|------|----------------|
| `runbox.yml` | Owning config: image, user, script, mounts, plugins, build inputs. |
| `runbox.dockerfile.template` | Dockerfile template rendered by `.build`. |

Common invocations (any directory inside the workspace):

```bash
runbox .build          # bake/refresh the shared image
runbox .live           # full workspace suite (config script: verb/test.d/l1)
runbox .shell          # interactive shell in the test environment
runbox .clean          # remove aged runbox-owned debris
runbox .help           # engine reference; runbox .live.help etc. per command
```

Module entry points: `module/<m>/verb/test` (full suite, payload
`module/<m>/verb/test.d/l1`), `module/<m>/verb/test_only <filter>` (targeted,
payload `module/<m>/verb/test_only.d/l1 <filter>`). Engine documentation:
`family_dev/default/module/runbox/readme.md` (schema, shared-image model,
staleness contract, `.clean`).
