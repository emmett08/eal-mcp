# Ground source predicates and check applicability

`grounded/1` evaluates an explicit source-to-formula contract. It checks which
acquired fields give a propositional atom its truth value, then evaluates the
required formal premises under that valuation. A copied interpretation string
or an `accepted: true` result is insufficient when the required source field or
atom definition is absent.

The public pure Rust API is `eal_core::verification::evaluate`. Its four inputs
are independently required `Coverage`, an operator-owned `Contract`, a
`Context` containing observations and explicit assessment time, and
`ExecutionLimits`. The API performs no acquisition and reads no clock.
`evaluate_query` accepts the same inputs as the native method's JSON object.

## Separate the obligations

Coverage names the target, its mode, required premise formulae and required
source obligations. The consumer derives this coverage from its selected
argument. The proposal must not choose a smaller subset to make its own
verification pass.

The three modes preserve different meanings:

- `mathematical` treats formulae as definitions or hypotheses. Source grounding
  and applicability can be `not_required`; this does not prove the formula.
  The caller must separately establish the mathematical inference with its
  selected native method. The report explicitly records `inference_checked:
  false` at this boundary.
- `empirical` requires executable source predicates for every atom in the
  covered premise formulae. A measured false atom is a known valuation;
  evaluating a premise to false fails that premise. Missing source fields or
  atom definitions remain unknown.
- `defeasible` also requires an independently identified warrant or review
  obligation. Satisfying that declared obligation supplies defeasible support;
  it does not turn the warrant into a deductive theorem.

For example, the atom `low_voltage` can be defined by the acquired field
`voltage < 10`, with source identity `supply-S`, unit `V` and scope
`{"configuration":"C"}`. A reading of 5 V from that source grounds the atom
true; 12 V grounds it false. An unbound symbol, a missing voltage field or a
failed acquisition supplies no truth value. The native finite propositional
evaluator is shared with `deductive/1`; EAL does not parse arbitrary prose as
logic.

Predicates use EAL's closed expression interpreter and exact integer
comparisons. Decimal arithmetic retains EAL's binary64 semantics; rational
quantities retain their typed representation. Constant expressions without an
acquired field cannot masquerade as source grounding. Unknown or incompatible
operands do not become false observations.

## Check every relied-upon observation

Each source used by an atom or source obligation needs its own applicability
entry. The entry selects actual source fields with JSON pointers and states the
required identity, unit, scope and half-open validity interval. Unit comparison
is exact: this checker invents no unit conversion. A conversion must be performed
and identified before supplying a differently expressed observation.

The observation's original timestamp and source digest remain separate from its
value. The original measurement must lie within the applicability interval,
precede the assessment time and satisfy the maximum age. The source interval
must contain the required interval, and that interval must contain assessment
time. Missing facets, clocks or provenance remain unknown. Wrong-device,
wrong-unit, wrong-scope, stale, future-dated and expired observations fail.

Native argument evaluation supplies its actual explicit assessment time to
`grounded/1`. A timestamp embedded in an acquired payload cannot move that
evaluation into the past to make an old observation fresh. Direct users of the
pure API must likewise supply their authoritative evaluation time.
The native adapter also binds each embedded observation's source digest to its
actual retained evidence container and caps its age at the older of the
container's original clock and its supplied original clock. A missing original
timestamp stays missing. A fresh nested timestamp cannot refresh an old
retained container, even when that container passes a looser outer age limit.

## Inspect the result

The native method input has `coverage`, `contract`, `observations` and
`assessed_at`. An EAL proposition's authored query fixes `coverage` and
`contract`; measured input supplies the observations. The method requires an
evidence entry of kind `verification_case`. It has no observation-free theorem
admission exception.

Native details contain `verified` and a versioned
`eal.verification-report/1` report. The report retains each atom valuation,
formula result and source/applicability obligation. A completed negative check
has `verified: false` and report status `fail`. Missing source grounding is also
a completed check, with `verified: false` and report status `unknown`. Native
method completion therefore cannot substitute for the report's own verdict.
Computation exhaustion remains native `incomplete`.

Reports expose only fields read by the source predicates and selected
applicability metadata, alongside original clocks and source digests. They do
not expose the whole acquired value, transport settings or unrelated fields.
Inspection details represent integers outside the browser's safe range as
`eal.exact-number/1` objects with string numerator and denominator. Evaluation
uses the original exact values.

`verification::compose` aggregates reports for an independently supplied set of
selected dependencies. Missing or ambiguous child reports remain unknown;
failed children remain failed. It records each child report's canonical digest
without creating another measurement or claiming a new proof.

ASPIC export retains recorded reports for native grounded methods, including
source rules absent from the constructed graph. Report status, native method
completion, authored support and ASPIC acceptability remain separate results.

The contract makes a bounded interpretation executable and inspectable. The
consumer still owns the choice of denotation, scope and source authority.
Identity digests establish supplied correspondence, not independent source
authentication or unqualified truth about a physical system.
