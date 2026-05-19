# run/

Runbox integration for `python` example.

| File | Responsibility |
|------|----------------|
| `runbox` | Universal wrapper: auto-discovers runbox-run via walk-up (copy verbatim). |
| `runbox.yml` | Project config: image, cache_dir, dockerfile, test/lint/run scripts. |
| `runbox.dockerfile` | Python container image: venv install + seed mount point. |
| `plugins.sh` | Test lister: pytest --collect-only instead of default nextest list. |
| `verb-run` | Universal verb dispatcher: resolves verb file and execs with VERB_LAYER in environment. |
