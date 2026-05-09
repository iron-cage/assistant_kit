# Runbox Plugins

Swappable behavioral components that fill execution slots in the runbox infrastructure.
Plugins have identity, presence/absence logic, and lifecycle management.
When a plugin is absent, the execution shape changes — not just a value within it.

See `parameters.md` for scalar configuration values.

## Legend

| Symbol | Meaning |
|--------|---------|
| ⚠️ | Partial — implemented and configurable, but capacity hardcoded at one instance |
| 🔒 | Hardcoded — mechanism embedded in `runbox.dockerfile` or `docker-run`, not yet swappable |

## Table

| Plugin | Status | What It Controls | Mechanism | Notes |
|--------|--------|-----------------|-----------|-------|
| `bin_plugin` | ⚠️ | Host binary injected into container with a working volume | `which name` → bind-mount `:ro`; named volume for working dir | Configurable in `runbox.yml`; capacity hardcoded at one; current use: `w3` |
| `plugin_mount` | ⚠️ | Host data directory mounted into container | Presence check → `-v`; required+rw for `.test`, optional+ro for `.shell` | Configurable in `runbox.yml`; capacity hardcoded at one; current use: `~/.claude` |
| Dep cache | 🔒 | How external deps are pre-compiled before the test stage | cargo-chef 4-stage build hardcoded in dockerfile | Always cargo-chef; no way to switch to sccache or a simpler single-stage build |
| Build cache persistence | 🔒 | How compiled artifacts survive between `docker run` invocations | Named volume seeded from image via `cp -a` in `_ensure_build_cache`; `target_seed/` dir baked into dockerfile | Always volume+seed; strategy split across docker-run and dockerfile |
| Test lister | 🔒 | What command enumerates tests for `.list` | `cargo nextest list $CMD_SCOPE --all-features` hardcoded in `cmd_list` | Tied to nextest; changing the test runner would break `.list` |
| Offline runner | 🔒 | What the image's baked `CMD` executes | `cargo nextest run $CMD_SCOPE --filter-expr $CMD_FILTER` baked at build time via dockerfile `ARG` | Always nextest; changing requires a new image build |
