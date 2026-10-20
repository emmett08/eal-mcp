# Rust port status

The development reference is
[`earl@e2d6f346a37f92483b9f9a13923b98feb053447b`](https://github.com/emmett08/earl/tree/e2d6f346a37f92483b9f9a13923b98feb053447b),
package `3.2.4`, language `EAL/3`. The first PR establishes the Rust workspace,
frozen contracts, generated recognition, owned lexical syntax representation and
lexical formatting.
It does not yet provide the EAL reasoning service or an MCP server.

## Scope and component boundaries

The [source inventory](../contracts/source-lock.json) accounts for all 77
handwritten Python modules installed under `src/eal`, totalling 13,647 lines.
Experiments, papers and research reporting scripts are excluded. Relevant
regression tests and executable examples can become Rust tests and examples as
their corresponding implementation is ported.

| Planned component | Responsibility | Stage |
| --- | --- | --- |
| `eal-parser` | Generated recognisers and syntax diagnostics | First PR |
| `eal-core` | Owned syntax, formatting, binding, checking, expressions, methods and authored evaluation | Syntax and lexical formatting in first PR, semantics pending |
| `eal-aspic` | Finite ASPIC construction, preferences, defeat, extensions, compilation and checked view export | Pending |
| `eal-runtime` | Collection, immutable records, reuse, catalogue, method execution and supervision | Pending |
| `eal-mcp` | Exclusive operator/registered catalogues, authentication and transports | Pending |
| `eal-coordination` | Durable task coordination, delivery and effect identities, codec and worker adapters | Pending |
| `eal-cli` | Command-line interfaces and text-only model host | `eal-syntax` front end in first PR, production commands pending |

Pure reasoning must depend on typed values and explicit inputs. Storage, process
execution, filesystem access and transports remain outside that core. The
authored EAL evaluator and opt-in ASPIC compiler retain their separate semantics.

## Remaining implementation sequence

1. **Complete the typed semantic representation and static passes.** Port name
   binding, trusted imports, inherited defaults, typed closed patterns,
   composition and propositions. Extend formatting through these checked
   semantic passes. Compare valid and invalid current
   fixtures with the pinned Python implementation. Require parse–format–parse
   equivalence, formatter idempotence and aggregate resource limits.
2. **Port pure reasoning.** Implement all eight built-in methods, strict typed
   expressions, arbitrary integers, binary64 arithmetic, exact rational unit
   conversions and Python-compatible negative modulo. Retain missing-value and
   expression-error propagation. Port authored objection and support semantics
   and the standalone Dung grounded solver.
3. **Port finite ASPIC.** Preserve premise kinds, rule kinds, all three defeat
   kinds and preference policies, grounded/preferred/stable semantics,
   credulous/sceptical queries and finite premise-founded cyclic construction.
   Distinguish unconstructed, undecided, no-extension and resource-incomplete
   outcomes. Check small theories independently before introducing indexed or
   bitset optimisations.
4. **Port runtime and extensions.** Preserve original observation times, exact
   acquisition correspondence, canonical identities, SQLite records and private
   store ownership. Implement a supervisor process that owns collector cleanup
   and inherited acquisition leases. Replace Python import-based method loading
   with a typed Rust registry and schema-checked subprocess extension protocol.
   Rust reports its own implementation identities.
5. **Port MCP, host, catalogue and coordination.** Use the official `rmcp` SDK
   for stdio and stateless Streamable HTTP. Preserve the eleven-tool operator
   catalogue and separate three-tool registered-source catalogue. Port all four
   entry points and the production coordination modules, including replacement
   of Python-specific process bridges. Preserve delivery identity, durable
   reservations and refusal to retry calls with an uncertain execution outcome.
6. **Verify visualisation and packaging.** Implement checked `aspic-view/3`
   exports. Validate against both the retained schema and the visualiser's
   additional graph checks. Exercise real MCP sessions, cancellation, abrupt
   process death, observation reuse and an installed package with Python absent.

The selected ANTLR generator is pinned with its runtime and recognisers are
checked in. A normal build uses those Rust sources. Java is needed only for
regeneration. The first PR's frozen Python fixtures support normal Rust tests
without importing Python.

## Visualiser boundary

The consumer is pinned to
[`aspic_visualisation@a38bff4a193634556ef81e64f9b7823a1b1de6ec`](https://github.com/emmett08/aspic_visualisation/tree/a38bff4a193634556ef81e64f9b7823a1b1de6ec).
Its `schema/aspic-view.schema.json` matches the producer's
`docs/aspic-view.schema.json` byte for byte. Static JSON imports can retain this
contract without a visualiser change once Rust export exists.

The live editor's `server/solver-plugin.ts` currently spawns Python to run
`server/solve.py`, which imports the EAL solver. A separate visualiser PR must
select a native Rust executable that accepts the current theory request and
emits checked view JSON. File import compatibility and live solving require
separate end-to-end checks. This first PR changes neither source repository nor
visualiser.

## Completion criteria

Parser tests establish recognition and the tested lexical representation.
Complete port acceptance additionally requires differential semantic results,
independent small-graph solver checks, numerical and identity fixtures, real
stdio/HTTP round trips, process ownership tests, observation reuse checks,
visualiser validation and a Python-free installed-package smoke test. Pending
components remain explicit until these checks exercise their implementations.
