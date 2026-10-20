# Pinned source contracts

These files record the port's development reference. They describe the Python
implementation at `earl@e2d6f346a37f92483b9f9a13923b98feb053447b`, package
`3.2.4`, supporting `EAL/3`. A contract snapshot does not implement an operation.

| File | Contents |
| --- | --- |
| `source-lock.json` | Exact revision, grammar hash, all 77 handwritten installed Python modules, planned Rust responsibilities and imported-file provenance |
| `public-contracts.json` | Four entry points and CLI argument structures, exclusive MCP catalogues, eight built-in and five optional method contracts, ASPIC schemas, execution budgets and persistence contracts |
| `identity-fixtures.json` | Synthetic Python canonical JSON, SHA-256 and frozen HMAC examples |
| `schemas/aspic-view.schema.json` | Byte-identical current producer schema, independently compared with the pinned visualiser |
| `schemas/coordination-worker-response.schema.json` | Current coordination worker response schema |
| `licenses/` | Upstream licence texts retained with the imported materials and planned coordination port |

The manifest records implemented, partial and pending native module replacements.
The status document qualifies their tested scope and native implementation deltas.
Packet/catalogue/discovery/transport and coordination adapters remain pending;
planned component names do not imply complete component acceptance.

The input schemas in `public-contracts.json` come from `operations.py` annotations
and `OPERATION_FIELDS`. They retain types, required fields and defaults while
omitting framework-generated titles and descriptions. The Rust MCP stage must
also compare real `tools/list` responses and exercise protocol round trips.

Built-in and optional method contracts retain the Python reference implementation's version and
execution classification under explicitly named `reference_*` fields. Python
code-object digests and registry fingerprints have been omitted. Rust must
identify its own implementation.

The identity fixtures use a published synthetic key. They are reference data for
testing, and must never become a runtime key. Their `EAL/2` HMAC prefix is the
existing frozen acquisition hash domain used by `EAL/observation-record/1`.
It identifies that record algorithm independently of the EAL/3 source language.
The fixtures cover arbitrary integers, integral floats, negative zero, exponent
formatting, Unicode, key ordering and typed Boolean values. Native canonical bytes and runtime HMACs are checked against these references.

Inventory extraction is deterministic: enumerate `src/eal/**/*.py` in path order,
exclude `generated/`, hash exact bytes, and record the first module docstring line
and planned Rust component. Public method schemas come from
`default_registry().describe()` and the five optional `MethodContract.describe()`
results with Python implementation identities removed.
ASPIC schema constants and `ExecutionLimits` defaults are read directly from
their source modules. MCP signatures and argparse construction supply interface
metadata. The source manifest records hashes of these inputs. The Python
implementation remains a development oracle outside the delivered application.

For the remaining stages and acceptance checks, see
[`docs/port-status.md`](../docs/port-status.md).
