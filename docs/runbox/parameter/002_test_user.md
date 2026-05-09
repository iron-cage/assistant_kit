# Parameter: `test_user`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** `testuser`
- **Where It Flows:** `ARG TEST_USER` in `runbox.dockerfile` → `useradd` + `chown` at build time

### Notes

Non-root user. Required for `chmod 000` permission tests and realistic home-path resolution tests.
