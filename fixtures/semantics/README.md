# Cross-component semantic reference

`reference.json` retains 93 synthetic cases derived from production regression
tests at `earl@e2d6f346a37f92483b9f9a13923b98feb053447b`.
These observations exercise the interpreter and contain no experimental
measurements. The reference captures 89 compiled programmes, including 71 with
no static diagnostics, and four source-import failures.

Each case contains its exact source and digest, optional in-memory import map,
operator-selected budget overrides, explicit time/context, immutable observation
records, compiled semantic projection, diagnostics and evaluation result. The
projection includes all eight executable declaration tables, pattern/application
identifiers, source/import identities, generated identities, exact flat source
locations, reviewed argumentation directives and lowering diagnostic codes.
Standalone validation diagnostics and assessment diagnostics compare every field
without rewriting their source-span shape. Expression trees use the reference implementation's portable
`kind`, `value` and `arguments` representation. A Rust comparison adapter must
translate its expression enum spellings to this representation.

The capture omits `method_registry_fingerprint`: the Rust implementation has its
own backend identity. The native comparison also projects implementation-specific
members of the method contract attached to a typed argument binding:
`implementation_version`, `implementation_source_digest`,
`implementation_code_digest`, `implementation_artifact_digest` and
`identity_basis`. The Python and Rust built-in execution labels are mapped to
`builtin`. This projection applies only to the interpreter's binding contract,
while its schemas, units, quantities, input/output limits and output declarations
remain checked. All observation/request digests,
statuses, result payloads, graph labels and explanations remain available for
comparison. Integers require exact equality. Any tolerance for numerical
algorithm output must identify the affected path and its mathematical reason.
The native comparison permits at most eight binary64 representable values of
rounding difference for the specified quantile, mean, standard-error and
posterior result paths. Observation payloads, integers, identities, statuses and
predicate outcomes retain exact comparison.

The cases cover missing/false/malformed observations, sixteen identity-field
mutations, current tool-binding checks, equivalent timezones, the Unix epoch,
half-open assumption validity, age boundaries, all objection target categories,
independent derivations, circular attacks and defended assumptions. Composition
cases cover recursive lists, guarded termination, local forward references,
trusted imports, definition-relative imports inside templates, unused missing
imports and aggregate budgets. Literal user payloads with keys named `span`,
`method_contract` and serde interchange names remain unchanged. Typed proposition cases cover exact
pressure-unit conversion, formal-query correspondence, source identity and
validity intervals. Static cases include missing and wrong-kind references,
unknown methods, duplicate symbols, premise cycles, incompatible dimensions and
invalid result contracts. All eight built-in reasoning methods are exercised
through complete source-to-assessment cases, including negative findings and
invalid method payloads.

To regenerate with the pinned, unmodified Python source:

```sh
python3 tools/capture-semantic-reference.py ../earl-audit
python3 tools/capture-semantic-reference.py ../earl-audit --check
```

This is a development capture step. Native Rust comparisons consume retained
JSON and do not invoke Python, collectors, external imports or a clock.
