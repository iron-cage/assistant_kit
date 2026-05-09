# Plugin: `bin_plugin`

- **Status:** ⚠️ Partial — configurable in `runbox.yml`; capacity hardcoded at one instance
- **Controls:** Host binary injected into container with a working volume
- **Mechanism:** `which name` on host → bind-mount binary `:ro` into container; named Docker volume for working directory

### Notes

Configured via `runbox.yml` key `bin_plugin: name:/container/path`. Current use: `w3:/usr/local/bin/w3`. A second binary plugin slot requires code changes to `docker-run`.
