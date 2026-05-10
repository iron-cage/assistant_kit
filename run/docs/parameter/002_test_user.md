# Parameter: `test_user`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** `testuser`
- **Where It Flows:** `ARG TEST_USER` in `runbox.dockerfile` → `useradd` + `chown` at build time

### Notes

Non-root user. Required for `chmod 000` permission tests and realistic home-path resolution tests.

### Example

```yaml
test_user: testuser
```
Passed as `--build-arg TEST_USER=testuser`. Dockerfile consequence:
```dockerfile
ARG TEST_USER=testuser
RUN [ "$TEST_USER" = "root" ] || useradd -m -s /bin/bash "$TEST_USER"
RUN [ "$TEST_USER" = "root" ] || (chown -R "$TEST_USER":"$TEST_USER" /workspace /usr/local/cargo \
      && chmod -R a+rwX /workspace /usr/local/cargo)
USER $TEST_USER
```
A test that calls `fs::set_permissions(path, Permissions::from_mode(0o000))` then asserts `PermissionDenied` would silently pass as root (root bypasses DAC). Running as `testuser` ensures the denial is real.
