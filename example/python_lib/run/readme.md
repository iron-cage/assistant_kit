# run

Runbox integration for python_lib.

| File | Responsibility |
|------|----------------|
| `runbox` | Universal wrapper: auto-discovers runbox-run (copy verbatim for any project) |
| `runbox.yml` | Project config: image, cache_dir, dockerfile, test script |
| `runbox.dockerfile` | Python container image with venv-based dependency caching |
| `plugins.sh` | Test lister: pytest --collect-only instead of cargo nextest list |
| `test` | Online test script: invokes pytest inside the container |
