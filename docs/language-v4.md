# EAL/4 executable language

`language "EAL/4"` declares the current language. Evidence dependencies,
bounded pure derivations and authored method queries are ordinary language
semantics. The header has no optional semantic selector. Every imported module
must also declare EAL/4. Older source versions are rejected before collection;
re-export or explicitly migrate the source and all imports together.

A fixed EAL/4 declaration has the same method-input meaning when compiled alone
or imported. Imported identifiers are resolved lexically. Current typed
observation envelopes retain their explicit identity, unit and scope checks;
raw empirical inputs merge with authored query fields only when overlaps agree.

## Evidence inputs and dependencies

An evidence declaration selects exactly one external `tool` or a pure `derive`
expression. Its `kind`, `environment`, `max_age` and at least one `require`
predicate remain mandatory. An explicit derivation replaces an inherited context
tool; an explicit tool and derivation in one declaration are rejected.

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
Schema information supplied by tools is checked at the execution boundary; the
language does not pretend an unconnected tool's output shape is statically known.

Bindings resolve lexically inside modules and expanded patterns. Collection
follows the transitive acquisition closure with each parent before its consumers.
The claim planner retains each dependency once and distinguishes external tool
calls from pure derivations in its estimate. Acquisition cycles are rejected;
objection cycles retain their existing argumentation semantics.

## Pure derivation contract

A derivation evaluates the closed EAL expression language against its resolved
input and must return a JSON object. `derive result` can select a constructed
input object. `object("count", count(items), "total", sum(values))` constructs
an object explicitly. Object keys are unique strings; physical quantity values
cannot silently lose their unit metadata inside a JSON object. Scalar or unknown
results produce explicit diagnostics. There is no implicit result wrapper, shell
evaluation, network operation or general-purpose code execution in an expression.

The array functions `at(array,index)`, `flatten(array)`, `contains(array,value)`
and `distinct(array)` operate on bounded finite JSON. `at` uses a nonnegative
integer index and rejects missing entries. `flatten` removes one array level
and preserves other elements without coercion. Identity comparisons preserve JSON scalar types
and exact integers. These operations do not certify that the original source
population was complete or representative.

Pure derived records retain their dependencies and original measurement age;
a later calculation must not make old observations fresh. Source predicates,
method checks and interpretation assumptions remain separate obligations. A
calculation cannot confer external source authority on its output.

## Authored method inputs

For a typed conclusion in EAL/4, omitting `binding evidence_id` selects
the proposition's authored `query` as a complete method input. Static validation
checks it against the installed method's full input schema. This permits, for
example, a deductive query without requiring a collector to echo its fixed
formulae. Evidence, premises, applicability assumptions and objections still
control the argument's support.

An explicit method evidence binding supplies acquired empirical fields; the
runtime checks their correspondence with the authored query. A method result
records the evaluated input separately from observation identity. Authored
formulae and empirical observations never become interchangeable evidence.

## Identity, formatting and ownership

The compiler owns name resolution, static shape checks, dependency validation
and method-contract checks. Pure expression and method evaluators do not invoke
external tools. Runtime adapters own external transport and normalisation.

Authored formatting preserves the typed AST; semantic formatting checks the
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
