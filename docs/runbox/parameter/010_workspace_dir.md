# Parameter: `workspace_dir`

- **Status:** 🔒 Hardcoded — in both `runbox.dockerfile` and `docker-run`
- **Current State:** `/workspace`
- **Where It Flows:** `WORKDIR /workspace`, all volume mount paths, `ENV HOME /workspace`, `docker run -w /workspace`

### Notes

Split across both dockerfile and `docker-run` — both must change together. Baking it into one place is blocked by the need for it at both build time and run time.
