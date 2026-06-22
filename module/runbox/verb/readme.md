# verb/

| File | Responsibility |
|------|----------------|
| `test` | Dispatcher: delegates by VERB_LAYER; default calls container runner |
| `test.d/l0` | Host-native layer: runs w3 .test level::3 directly |
| `test.d/l1` | Container-internal layer: runs w3 .test level::3 with offline flags |
| `build` | Compile runbox crate artifacts via cargo build |
