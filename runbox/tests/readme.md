# run/tests

Regression tests for the container runner shell script infrastructure.

| File | Responsibility |
|------|----------------|
| `runbox_build_safety_test.sh` | Regression tests: build lock, post-build image verify, content hash staleness detection, OCI mountpoint pre-creation, hash scope coverage, and Cargo.lock pre-generation (T1–T7). |
