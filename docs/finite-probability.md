# Check an exact finite-model consequence

`finite_probability/1` enumerates a supplied finite Bayesian network with exact
rational arithmetic. It checks a mathematical consequence of the supplied
definitions. Model transcription, interpretation, empirical adequacy and the
execution of another implementation remain distinct obligations.

The [example programme](../examples/finite-probability/case.eal) is a complete
EAL/4 argument. It computes the stipulated diagnostic posterior `152/201` and
evidence probability `201/10000`, then compares the posterior with `3/4`
exactly. It needs no collector, generated observation or assumed review verdict:

```sh
eal-core --workspace examples/finite-probability validate case.eal
eal-core --workspace examples/finite-probability evaluate case.eal \
  --context '{}' --now 2026-10-30T12:00:00Z
```

Omitting `--records` supplies the empty observation map. Embedders can use
`methods::assess_input_with_limits` with the same installed method descriptor.
The method is part of the default registry, so source, CLI and MCP executions
share its contract.

## State the model and its mapping

The query has exactly four fields:

- `nodes`: an array of definitions with `name`, `states`, ordered `parents` and
  flattened conditional `probabilities`;
- `evidence`: a mapping from node names to selected state labels;
- `target`: the queried `variable` and `state`;
- `comparison`: an `operator` and exact rational `threshold`.

Probabilities and thresholds are integer or `numerator/denominator` strings.
Denominators must be positive. JSON floats, decimal strings and booleans are not
silently converted. Node names are identifiers; state labels are distinct,
nonblank strings and retain their exact meaning.

Each CPD row follows the parents in declared order, with the last parent varying
fastest. Within each parent configuration, child states vary fastest in their
declared order. Every row must sum exactly to one. The graph must be acyclic,
and all parents, query variables and evidence states must exist. Node array
order affects enumeration and the inspection trace, but not the represented
distribution. An incorrectly permuted state or parent mapping can describe a
different valid model; the method reports that model's result. A comparison with
an independently specified intended mapping is still necessary.

## Inspect the exact result

`defined` reports whether the evidence probability is positive. When it is,
`posterior_exact`, `normalizer_exact` and `numerator_exact` use
`eal.exact-number/1` records with string numerators and denominators.
`satisfies` evaluates the authored comparison using those exact values.
`posterior` and `normalizer` are binary64 display approximations, which can
round or underflow; they do not govern `satisfies`.

Zero-probability evidence yields `defined: false`, `satisfies: false` and no
posterior fields. The calculation still completed. These flags do not assert
that an undefined conditional probability is a numerical zero. An adverse,
defined posterior can also return `satisfies: false`; neither case supplies
support for a claim requiring `satisfies == true`.

The result includes the node/state/parent mapping, target, evidence event,
assignment count and matching-assignment count. The ordinary method result
retains the authored input digest and method identity. These identify the
calculation; they do not authenticate external premises.

## Keep proofs separate from observation freshness

A complete typed `finite_probability/1` query may form a zero-support argument.
An empty-premise typed `deductive/1` query may similarly establish a tautology.
Untyped prose, structured review, partial queries and empirical or statistical
methods retain the usual evidence/premise requirement. This exception creates
no observation and changes neither `prose_verified` nor physical applicability.

Rechecking immutable definitions performs the calculation without assigning a
new measurement time to old evidence. Explicit proposition intervals and
environment conditions still apply to the authored claim. Changing a definition
or query changes its identity and requires recomputation. Runtime, measurement
and review records retain their independent scope and original clocks.

Finite propositional equivalence needs no additional method: ask `deductive/1`
with no premises to establish the conjunction of both implications. A completed
negative result retains a countermodel. This checks the supplied formal
statements; the connection to intended prose remains a separate obligation.

## Bound the computation

Models contain 1–12 nodes, 2–8 states per node and at most 65,536 joint
assignments. Rational inputs contain at most 128 ASCII characters. Intermediate
numerators and denominators are limited to 8,192 bits. The existing method-work,
JSON, schema and byte budgets also apply. Work or intermediate-size exhaustion
returns `incomplete` with no partial result. Malformed models return an explicit
method failure; they never become complete positive evidence.
