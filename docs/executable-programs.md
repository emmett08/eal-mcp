# Executable evidence programs

An executable package consists of `case.eal`, `tools.toml` and any custom tool
implementations under `tools/`. EAL owns acquisition dependencies, checks,
method queries, assumptions and objections. TOML selects concrete tools and
operator-authorised execution limits. There is no capability registration,
provider ranking or semantic execution-plan sidecar on this path.

Run an installed package through the MCP host:

```sh
eal-host run case.eal --tools tools.toml --workspace run --claim inventory_verified
```

`--context context.json` supplies an explicit context; otherwise the manifest's
`[context]` table is used. `--now` selects the assessment time, not the source's
observation time. The host sends the same `eal_execute` request available to a
native MCP client. The response contains both `collection` and `assessment`.
A successful command says that the operation completed; inspect the claim's
status to determine whether it is supported.

Optional preparation discovers only the selected operator-configured MCP
interfaces and emits inspectable server and schema pins:

```sh
eal-host prepare case.eal --tools tools.toml --workspace prepare-run \
  --claim inventory_verified --output-tools prepared.tools.toml
```

Run with `--tools prepared.tools.toml` to enforce the generated pins. The output
file must be new and beside the input manifest so relative tools keep their
meaning. Preparation calls no evidence operation and creates no observation or
assessment. Its receipt is not required for execution. See
[direct tools](direct-tools.md#response-mappings-and-schemas) for identity limits.

The source declares the current language:

```eal
language "EAL/4"
```

Every imported module declares the same EAL/4 language. Older source versions,
undeclared references, conflicting destinations and acquisition cycles are
diagnosed before acquisition. `eal_describe` reports the source language,
execution features and method contracts; authoring checks those contracts before
constructing tools. Migrate older source and imports explicitly before running.

## Acquisition and pure checks

Evidence can call a concrete tool, or compute a bounded pure derivation. Each
`bind` copies a normalised value into a declared input position using RFC 6901
JSON pointers. An empty pointer selects the whole value. The metadata around an
observation is not part of its normalised value.

```eal
evidence audit {
  tool identifier_audit
  kind calculation
  environment fixture
  max_age 3600
  input {"items":null}
  bind "/items" from snapshot "/items"
  require count == 3
}
evidence exact_count {
  derive object("count", count(items), "reported", reported)
  kind calculation
  environment fixture
  max_age 3600
  input {"items":null,"reported":null}
  bind "/items" from snapshot "/items"
  bind "/reported" from audit "/count"
  require count == 3
  require count == reported
}
```

The dependency scheduler acquires a shared source once for the selected
collection. A pure derivation retains upstream identities and original age.
It is not a second observation of the world. A measured negative can supply data
to a counter-evidence computation; rejection of a support predicate does not
erase acquired data. Unavailable, malformed or stale required input cannot become
fresh successful support through a derivation.

Pure functions include `count`, `sum`, `all`, `any`, `min`, `max`, `abs`,
`quantity` and explicit JSON object construction with `object(key,value,...)`.
`at(array,index)` requires a valid zero-based integer index. `flatten` flattens
exactly one level of nested arrays and preserves other elements without coercion. `contains` and `distinct`
use exact typed JSON identity: object key order is irrelevant; integers, floats,
booleans and strings remain different domains, and strings are case-sensitive.
These operations obey the interpreter's JSON and expression budgets.

`equal(left,right)` compares complete JSON values using the same typed identity.
`exact_sum`, `exact_mean`, `exact_min`, `exact_max` and `exact_count` operate on an
array of selected inputs (a sole selected array supplies its elements). They
return `eal.exact-number/1` records with string numerator and denominator fields,
and retain compatible units. `exact_scale(value,numerator,denominator,from,to)`
uses explicit integer-string factors and unit names. `exact_compare(value,
threshold,operator,unit)` compares rational values exactly. These functions use
bounded decimal-rational arithmetic, distinct from ordinary binary64 arithmetic;
empty numeric samples, conflicting units, zero denominators and exhausted digit
budgets are errors. `all_values` and `any_values` use the same selected-input
shape with strict Boolean elements; empty universal and existential checks return
true and false respectively.

## Direct tool authority

A known MCP operation can be selected directly:

```toml
[tools.snapshot_source]
kind = "mcp_stdio"
version = "1"
argv = ["python3", "tools/snapshot.py"]
inherit_env = ["PATH"]
timeout_seconds = 10
max_output_bytes = 65536

[tools.snapshot_source.direct]
tool = "snapshot"
response_format = "structured"
value_pointer = "/value"
observed_at_pointer = "/observed_at"
```

EAL sends the evidence input as the MCP tool arguments. The official MCP client
performs connection initialisation, discovery and invocation. Discovery checks
the selected interface; it does not grant new access or establish source truth.
Paths resolve against the manifest's directory, so the package can move while the
run directory remains separate. Credentials are supplied by explicitly named
environment entries, not embedded in EAL.

A command binding uses the existing bounded JSON stdin request and observation
envelope stdout contract:

```toml
[tools.identifier_audit]
kind = "command"
version = "1"
argv = ["python3", "tools/audit.py"]
inherit_env = ["PATH"]
timeout_seconds = 10
max_output_bytes = 65536
```

Retained `json_file` observations must include the original `observed_at`,
`context` and `request`. Rereading a file does not refresh its measurement age.
Implementation/schema pins and access limits are operator configuration; a
successful response is not a proof that an external population is complete.

## Authored queries and measured input

A complete native query is program data. For example, a finite conjunction query
belongs in the claim's proposition, not in a tool result that merely echoes it:

```eal
proposition {
  subject "retrieved fixture-v1 snapshot"
  quantity "proposition"
  unit "1"
  scope "finite snapshot; case-sensitive string equality"
  valid_from "2026-01-01T00:00:00Z"
  valid_until "2030-01-01T00:00:00Z"
  query {"premises":["count","distinct"],"conclusion":{"and":["count","distinct"]}}
  result entailed == true
}
```

In EAL/4, a complete query may execute without an observation binding.
The method result explicitly identifies `input_origin: "authored_program"` and
does not invent an evidence or observation ID. Evidence, premise claims and
assumptions in the argument still have to support their respective roles. The
correspondence between empirical statements and propositional atoms remains an
explicit authored interpretation; finite entailment does not verify that prose.

For methods that use measured data, `binding evidence_id` selects a normalised
object. Authored query fields supply fixed method parameters, such as an interval
confidence level. If an observation also supplies a query field, it must be
exactly equal; the runtime does not silently overwrite contradictory values.
The merged input is checked against EAL's authoritative method contract. Explicit
`EAL/typed-input/1` envelopes additionally check subject, quantity, unit, scope and
interval correspondence through the existing typed-binding contract.

`methods::validate_query` provides authoring-time query validation. It checks
schemas and limits, and checks complete built-in queries with their authoritative
bounded implementation. Partial sample queries are checked without inventing
success counts or treatment arrays. Extension callbacks are not executed during
this validation.

## Portable acceptance packages

`examples/executable/finite-inventory/` acquires a named synthetic snapshot over
MCP, invokes a custom command audit, independently checks its count and
distinctness in EAL, and performs a finite deductive conjunction. Only the exact
retrieved finite snapshot is claimed. A page flag or a tool's `complete: true`
does not establish completeness of an unobserved external catalogue. The Python
MCP server is a deliberately small protocol fixture, not a production server;
the client is the official SDK implementation.

`examples/executable/reasoning-modes/` acquires one synthetic observation and uses
its explicitly identified parts for the following computations:

| Mode | Executed operation | Limit |
|---|---|---|
| Deductive | Finite propositional conjunction | Authored atom/premise correspondence |
| Inductive | Wilson interval for 90 successes in 100 trials | Assumed binomial sampling model |
| Abductive | Posterior for two explicit hypotheses | Supplied priors, likelihoods and candidate set |
| Causal | Randomised treatment/control mean contrast | Supplied design/applicability condition |
| Counterfactual | Intervention in an acyclic affine model | Same supplied exogenous values |
| Analogical | Exact correspondence over named features | Does not prove transfer to a new system |
| Practical | Identified scoped review of two options | Supplied objective, capacity constraint and review judgement |

The review identity, criterion, target, scope, verdict and reasons are checked.
These are synthetic conformance records, not manufactured real-world reviews.
Rejected, inconclusive, wrong-scope and wrong-reviewer records cannot support the
practical claim. An evidence-supported objection targets the common applicability
assumption and affects the corresponding native and structured arguments.

The acceptance suite copies only source, manifest and tools to a fresh directory
and executes `eal-host` through MCP. It checks positive numerical results, source
failure, original stale timestamps, negative measurements, conflicting method
queries, applicability challenges and altered audit output. It does not invoke
EngArg or an authoring model.

```sh
cargo +1.90.0 build --workspace --bins --locked
cargo +1.90.0 test -p eal-mcp --test executable_packages --locked
cargo +1.90.0 test -p eal-core --test executable_method_inputs --test executable_expressions --locked
```

## Retained scoped reviewer records

`eal-collector review-json --inputs review-inputs.json` imports an operator-selected
file with schema `eal.review-inputs/2`. Each review is `{request,value}`. The
request must equal the exact acquisition identity, including context. Its input
has explicit nonempty `criterion`, `target` and `scope`. The value repeats those
three fields and supplies `reviewer`, `verdict`, a nonempty list of `reasons`, and
original UTC `reviewed_at`. Optional `provenance` and `evidence_refs` are retained.
The three verdicts are `supported`, `rejected` and `inconclusive`; no boolean
`accepted` compatibility branch exists. Original review time becomes the
observation time. Scope/identity checks establish correspondence, not truth or
reviewer independence; explicit source and reviewer policies remain visible in
the EAL evidence checks.

A declaration with `kind scoped_review` also applies the shared strict review
contract inside EAL, including when a record is projected from a batched source.
Its authored `input` declares the expected `criterion`, `target` and `scope`;
its value contains the complete review. Malformed records cannot supply bound
inputs. Original `reviewed_at` participates in the retained observation clock
and freshness checks, so a fresh enclosing response cannot refresh an old review.
A valid rejected or inconclusive review remains data, while its authored support
predicate determines whether it supports the claim. Generic `kind review` has
no such reserved admission contract.
