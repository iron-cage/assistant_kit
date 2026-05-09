# Parameter: `test_script`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** module-relative path (e.g., `run/test`, `module/dream/run/test`)
- **Where It Flows:** `docker run /workspace/$TEST_SCRIPT` — executed after plugin mounts are wired

### Notes

Full-test entrypoint. May invoke bin plugins (e.g., `w3`) and assumes plugin mounts are present. Used by `docker-run`'s `.test` command path.
