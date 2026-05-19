# run/

Runbox integration for `nodejs` example.

| File | Responsibility |
|------|----------------|
| `runbox` | Universal wrapper: auto-discovers runbox-run via walk-up (copy verbatim). |
| `runbox.yml` | Project config: image, cache_dir, dockerfile, test/lint/run scripts. |
| `runbox.dockerfile` | Node.js container image: npm install + seed mount point. |
| `plugins.sh` | Test lister: node --test --test-reporter=spec instead of default nextest list. |
| `verb-run` | Universal verb dispatcher: resolves verb file and execs with VERB_LAYER in environment. |
