# Portable executable examples

These are synthetic conformance packages. They demonstrate execution mechanics
and specified finite computations, not facts about an external production system.

After building/installing the EAL binaries, run from this directory:

```sh
eal-host run finite-inventory/case.eal --tools finite-inventory/tools.toml --workspace inventory-run --claim inventory_verified
eal-host run reasoning-modes/case.eal --tools reasoning-modes/tools.toml --workspace modes-run --claim all_modes
```

Python 3 is required by these small custom fixture tools. Copy either directory
to another workspace to run it; neither requires EngArg, catalogue registration
or an execution-plan JSON file. See [the executable-program guide](../../docs/executable-programs.md)
for query bindings, direct tool contracts, negative cases and interpretation limits.
