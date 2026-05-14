# run

Shell scripts for `claude_profile` container operations.

| File | Responsibility |
|------|----------------|
| `runbox` | Thin wrapper: delegates to workspace runbox-run with profile config. |
| `runbox.yml` | Profile Docker config: image, build args, plugins, test script path. |
| `verb-run` | Universal verb dispatcher: resolves flat-file or directory-form verbs by VERB_LAYER. |
