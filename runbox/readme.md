# runbox — container test environment (consumer checkout)

This checkout consumes the shared runbox engine that lives in
`family_dev/default/module/runbox/sh/runbox` (a sibling checkout under
`yrd_core/`). The retired in-tree stack — embedded Rust engine crate
(`module/runbox/`), flat-YAML `runbox-run` runner, per-module walk-up
wrappers, `plugins.sh` hook — was removed on migration (TSK-1436 pilot,
2026-08-06); configs now use the engine's nested schema.

| Path | Responsibility |
|------|----------------|
| readme.md | This overview and directory registry |
| runbox | Launcher: resolves and execs the shared engine, preserving CWD |
| runbox.yml | Owning config: image tag, build inputs, shared-image bake |
| runbox.dockerfile.template | Dockerfile template the engine renders at `.build` |

- One shared image (`claude_journal_viewer_test`) serves the whole checkout; it is built from
  THIS directory's config (`./runbox/runbox .build` from the checkout root) and
  module configs consume it (`image:` reference only, no `build:` section).
- Module entry points: `module/<m>/verb/test` (full suite via `.live`),
  `module/<m>/verb/test_only <filter>` (targeted, `.live -- env NEXTEST_FILTER=…`).
- Engine documentation: `family_dev/default/module/runbox/readme.md` (schema,
  shared-image model, staleness contract, `.clean`).
