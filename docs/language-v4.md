# Express the evidence dependency in EAL/4

An EAL source connects a claim to observations, inference rules, assumptions and
objections. The executable part of that source determines which observations
must be acquired before a dependent calculation or claim can be assessed. Start
with the [complete example](getting-started.md) if you have not yet run a package.

`language "EAL/4"` fixes these semantics for the root and every imported module.
A declaration has the same method-input meaning when compiled alone or imported,
and imported identifiers resolve lexically. Older roots and modules are rejected
before acquisition. Migrate the whole import closure together.

Typed observation envelopes retain explicit identity, unit and scope checks.
Acquired empirical fields can merge with authored query fields only when overlaps
agree. That rule prevents a tool result from silently changing the fixed query
whose result the argument interprets.

## Evidence inputs and dependencies

An evidence declaration selects exactly one external `tool` or a pure `derive`
expression. Its `kind`, `environment`, `max_age` and at least one `require`
predicate remain mandatory. An explicit derivation replaces an inherited context
tool. An explicit tool and derivation in one declaration are rejected.

```eal
language "EAL/4"
environment demonstration {
  require snapshot == "finite-v1"
}
tool snapshot_reader {
  version "1"
}
evidence snapshot {
  tool snapshot_reader
  kind observation
  environment demonstration
  max_age 3600
  input {}
  require complete == true
}
evidence count_check {
  derive object("count", count(items))
  kind calculation
  environment demonstration
  max_age 3600
  input {"items": null}
  bind "/items" from snapshot "/items"
  require count == 3
}
```

Each `bind` copies a selected normalised observation value into an existing
location in the literal input. Both pointers use RFC 6901, including the empty
root pointer and `~0`/`~1` key escapes. The source pointer addresses observation
`value`, never executor metadata. Missing destinations, malformed pointers,
overlapping destinations, unknown evidence and cross-environment dependencies
are static errors. Missing source values are acquisition errors, not null values.
Schema information supplied by tools is checked at the execution boundary. The
language does not pretend an unconnected tool's output shape is statically known.

Bindings resolve lexically inside modules and expanded patterns. Collection
follows the transitive acquisition closure with each parent before its consumers.
The claim planner retains each dependency once and distinguishes external tool
calls from pure derivations in its estimate. Acquisition cycles are rejected.
Objection cycles retain their existing argumentation semantics.

## Pure derivation contract

A derivation evaluates the closed EAL expression language against its resolved
input and must return a JSON object. `derive result` can select a constructed
input object. `object("count", count(items), "total", sum(values))` constructs
an object explicitly. Object keys are unique strings. Physical quantity values
cannot silently lose their unit metadata inside a JSON object. Scalar or unknown
results produce explicit diagnostics. There is no implicit result wrapper, shell
evaluation, network operation or general-purpose code execution in an expression.

The array functions `at(array,index)`, `flatten(array)`, `contains(array,value)`
and `distinct(array)` operate on bounded finite JSON. `at` uses a nonnegative
integer index and rejects missing entries. `flatten` removes one array level
and preserves other elements without coercion. Identity comparisons preserve JSON scalar types
and exact integers. These operations do not certify that the original source
population was complete or representative.

Pure derived records retain their dependencies and original measurement age.
A later calculation must not make old observations fresh. Source predicates,
method checks and interpretation assumptions remain separate obligations. A
calculation cannot confer external source authority on its output.

## Authored method inputs

For a typed conclusion in EAL/4, omitting `binding evidence_id` selects
the proposition's authored `query` as a complete method input. Static validation
checks it against the installed method's full input schema. This permits, for
example, a deductive query without requiring a collector to echo its fixed
formulae. Evidence, premises, applicability assumptions and objections still
control the argument's support.

An explicit method evidence binding supplies acquired empirical fields. The
runtime checks their correspondence with the authored query. A method result
records the evaluated input separately from observation identity. Authored
formulae and empirical observations never become interchangeable evidence.

## Identity, formatting and ownership

The compiler owns name resolution, static shape checks, dependency validation
and method-contract checks. Pure expression and method evaluators do not invoke
external tools. Runtime adapters own external transport and normalisation.

Authored formatting preserves the typed AST. Semantic formatting checks the
composed executable meaning after reparsing. `semantic_format::executable_digest`
hashes validated executable declarations, including bindings, derivations,
propositions and objections. It excludes source locations, import deployment
paths and retained authoring syntax. The source-byte digest remains separate:
semantic equality never rewrites historical source identity or observation age.

The implementation uses one compiler pipeline with independent recognition,
composition and static-validation stages. The bounded expression interpreter is
a pure strategy selected by the evidence declaration. External adapter selection
belongs to operator configuration. Adding another transport does not require
changing argument semantics, and adding another method does not require a new
collector protocol.


Continue with [executable programmes](executable-programs.md) to connect these
semantics to method inputs and reviews, or with [direct tools](direct-tools.md)
to bind an acquisition. Return to the [documentation path](index.md).
