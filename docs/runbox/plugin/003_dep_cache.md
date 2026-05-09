# Plugin: Dep cache

- **Status:** 🔒 Hardcoded — mechanism embedded in `runbox.dockerfile`
- **Controls:** How external dependencies are pre-compiled before the test stage
- **Mechanism:** cargo-chef 4-stage build hardcoded in dockerfile (chef → planner → cook → test)

### Notes

Always cargo-chef. No way to switch to sccache or a simpler single-stage build without rewriting the dockerfile.
