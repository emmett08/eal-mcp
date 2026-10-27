# EAL/4 in Rust

EAL turns an engineering argument into an executable evidence programme. The
source names the claim, the observations it needs and the conditions under which
those observations support it. An operator-owned TOML manifest connects that
programme to commands, MCP tools, HTTP services or retained measurements.

Start with the **[documentation](docs/index.md)**. It follows one argument from
installation and collection through assessment, inspection and later reuse.
The [first-run guide](docs/getting-started.md) executes a complete synthetic
example. The [EngArg guide](https://github.com/emmett08/engarg/blob/main/docs/index.md)
covers model-assisted authoring and generating missing tool implementations.

```sh
eal-host run case.eal --tools tools.toml --workspace run
```

This command calls `eal_execute` through MCP and retains both collection and
assessment. A completed operation can report a supported, unsupported or contested
claim. The result preserves that distinction and the evidence needed to explain it.

The workspace uses Rust 1.90.0 and edition 2024. Build and contributor checks are
in [development](docs/development.md). The repository uses GPLv3. Required
attribution is in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
