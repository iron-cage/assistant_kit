# run/

Container integration for `rust` example.

| File | Responsibility |
|------|----------------|
| `runbox` | Universal wrapper: auto-discovers container runner via walk-up (copy verbatim). |
| `runbox.yml` | Project config: image, cache_dir, dockerfile, test/lint/run scripts. |
| `runbox.dockerfile` | Rust container image: cargo build + clippy + seed mount point. |
| `plugins.sh` | Test lister: cargo test -- --list instead of default nextest list. |
| `verb-run` | Universal verb dispatcher: resolves verb file and execs with VERB_LAYER in environment. |
