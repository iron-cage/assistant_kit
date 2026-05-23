# Configuration File Parameters

Not applicable — clp does not read any configuration file.

All behavior is controlled via CLI parameters passed at invocation. There is no `~/.clprc`, `clp.toml`, `clp.yaml`, or equivalent configuration file. Parameter defaults are hard-coded in the binary; overrides require explicit `param::value` syntax at each invocation.

No environment-file dotenv mechanism exists either. See [003_env_param.md](003_env_param.md) for the three OS environment variables clp does consume.
