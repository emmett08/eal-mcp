# Finite ASPIC reasoning

`eal-aspic` implements the pinned `argumentation/aspic/2` theory contract,
`EAL/3-compiled-aspic/5` source translation and `aspic-view/3` export. These are
explicit formal operations alongside the authored EAL evaluator. An EAL source
uses `method "argumentation/aspic/2"` only when the host installs that registry.
The default eight-method registry remains unchanged.

## Explicit theories

A theory contains ordinary or axiom premises, strict or defeasible rules,
directed contrary pairs and a goal atom. Names, ranks, schemas and resource
limits are checked before construction. Acyclic theories construct every
supported antecedent combination. Cyclic theories construct finite
premise-founded arguments whose branches contain no repeated conclusion.
Unfounded cycles contribute no arguments. Separate named rules and alternative
derivations retain their identities, including shared subarguments.

The solver records each undermining, rebutting and undercutting witness against
the attacked subargument, then solves the resulting defeat graph. An undercut
or a one-way contrary defeats independently of preference. A reciprocal
contrary defeats when the attacker is not strictly weaker than the attacked
subargument under the selected policy.

| Policy | Comparison |
| --- | --- |
| `minimum_rank` | Minimum rank among ordinary premises and defeasible rules |
| `last_link_rank` | Minimum rank among last defeasible rules; ordinary premises when both last-rule sets are empty |
| `last_link_partial` | Explicit acyclic premise/rule priorities with strict elitist set lifting over the same last-link sets |

Empty comparison sets are strongest. Tied and incomparable sets retain attacks.
The reported `strength` remains the minimum numerical fallible rank under every
policy, so it is an inspection field when a last-link policy is selected.
Contrary strict/axiom closure and extensions containing contrary conclusions
are rejected by this particular finite profile.

Grounded semantics returns the least fixed-point extension. Preferred semantics
enumerates every maximal admissible extension. Stable semantics enumerates
every conflict-free extension that attacks every outside argument. Search is
exact within the captured budget; an exhausted search returns no partial
extension family.

| Query outcome | Criterion |
| --- | --- |
| `accepted` | Some extension contains a goal argument for a credulous query; every extension contains some goal argument for a sceptical query |
| `rejected` | Every goal argument is attacked in every extension |
| `undecided` | A goal argument exists and neither selected acceptance nor rejection holds |
| `unconstructed` | The goal has no constructed argument and the extension family is nonempty |
| `no_extension` | The selected semantics has no extension; neither query Boolean is true |

Grounded labels and grounded query fields remain present when preferred or
stable semantics is selected. Sceptical acceptance quantifies over conclusions,
so different extensions may contain different derivations of the same goal.

## Checked EAL compilation

Compilation accepts exact source text, observations, context, an explicit
assessment time, registry and limits. It first performs ordinary EAL assessment.
Available evidence becomes an ordinary premise; locally usable authored routes
become rules. Assumption validation, backing, alternative premises and scoped
objections preserve their declared relationships. Missing, stale, failed and
out-of-scope observations retain their distinct availability causes and produce
no measured premise. A marked structural axiom represents an empty snapshot or
an authored relation without observations.

Reviewed `strict`, `rank`, `contrary` and `prefer` declarations supply the formal
policies. Unannotated ordinary premises and defeasible rules have rank 500.
Reviewed priorities select partial last-link preference unless an incompatible
override is requested, which fails. The compiler rejects an objection that
would undercut a strict route, and a source already using the explicit ASPIC
method. It infers no logical relation from prose.

`CompiledTheory` owns private theory, source-map and assessment values. Its
checked getters return detached values, and it offers no public unchecked
constructor or deserialiser. Snapshot identity binds the exact source,
assessment, context, registry, backend, theory, limits and all present record
digests, including unusable records. Formal route status and the independent
authored claim status are both returned.

## Recomputed export

The exporter checks the complete supplied formal result by recomputing the
theory under its captured limits. Captured limits can tighten the exporter's
host policy but cannot raise it. Edited labels, ranks, subarguments, extensions
and defeat witnesses fail equality checks. The exporter also checks source-map
structure, declaration correspondence, reviewed annotations and availability
codes against the retained schema.

Exports state `formal_result: "recomputed"` and `provenance: "supplied"`.
Recomputation establishes the supplied graph's formal result. Source hashes,
observation identities and review references remain supplied provenance rather
than authenticated observations. The [native visualiser adapter](native-visualiser.md)
connects the explicit theory contract to the companion editor.

## Native commands and APIs

```sh
cargo run --locked -p eal-cli --bin eal-core -- aspic fixtures/aspic/example-theory.json
cargo run --locked -p eal-cli --bin eal-core -- --methods aspic describe
```

The explicit-theory example is synthetic. `aspic` reads an object containing
exactly `theory` and emits `theory` plus `formal`, ready for `export-aspic`.
The development command forms are:

```text
eal-core aspic THEORY_JSON
eal-core compile-aspic SOURCE --claim NAME --now ISO_TIME
  [--context JSON|@FILE] [--records FILE]
  [--semantics grounded|preferred|stable]
  [--query-mode credulous|sceptical]
  [--preference minimum_rank|last_link_rank|last_link_partial]
eal-core export-aspic RESULT_JSON
```

Common `--workspace` and TOML `--limits` options apply. Source or JSON input can
be `-` for standard input. Output goes to standard output. Registry profile
`aspic` installs the default eight methods plus ASPIC; `all` also installs the
four engineering methods. Explicit theory options belong inside its JSON
`options` object. The [production service](applications.md) also exposes checked
formal operations. `eal-aspic-solve` accepts the same theory request directly on
stdin and emits the checked view through its separately bounded process protocol.

Library callers use `solve_aspic`, `validate_theory`, `CompileOptions`,
`compile_eal_aspic`, `export_aspic_view`, `aspic_registry` or `with_aspic`.
Native callbacks now receive `&ExecutionLimits` and return typed `MethodError`.
This Rust API makes captured budgets available without implicit global state.
ASPIC failures distinguish `AspicError::Invalid` from `AspicError::Incomplete`.
The CLI emits a failed `incomplete` response with empty `details` and exit code 1
for unfinished formal computation. Invalid requests produce an error and exit
code 2. Method assessment retains `status: "incomplete"`, causing the enclosing
authored assessment to set `complete: false`.

Local graph `resource_limit` diagnostics also produce typed `Incomplete` during
compilation. The Python compiler wraps that particular failure as
`CompilationError` before checking completeness; the native diagnostic text
remains the same while its error category identifies unfinished computation.

Formal construction, arguments, antecedents, atoms, contrary pairs, defeat
witnesses and extension search have independent budgets. Public solver inputs
also enforce the selected JSON traversal limits, including the native depth
ceiling. This strengthens direct-call admission when a host selects JSON
budgets smaller than the source validator's fixed 100,000-node/depth-64 profile.
The schema profile then remains the pinned source profile. Compiler and exporter
also enforce the ASPIC contract's 256 KiB input and 1 MiB response limits.

Native implementation identity identifies a retained Rust source bundle and its
semantic helpers. It is distinct from the Python backend identity. The core
checks deterministic work and data budgets. The [runtime supervisor](runtime.md)
owns process deadlines and resource limits for supervised methods. The standalone
visualiser adapter remains a pure process whose HTTP owner enforces its deadline
and cancellation. Timeout metadata alone provides neither guarantee.

## Verification scope

Normal Rust tests compare retained, pinned Python solver, compiler, registry and
export cases without Python, collectors or a clock. Compiler comparisons omit
only declared backend metadata and hashes depending on it, then independently
recompute every native identity. Solver outputs and theory digests compare
exactly. Independent subset definitions check complete extension families,
query quantifiers, defeat witnesses and grounded labels for every directed
graph through three nodes, including self-attacks.

The optional development command `python3 tools/check-aspic-consumer.py
/path/to/src/domain/aspic.ts /path/to/eal-core` requires Python `jsonschema`
and Node 24, fetches nothing, and verifies the pinned consumer file's Git blob
before executing its unchanged graph-checking function bodies. The retained
fixture cases produced 148 native views (87 explicit and 61 compiled); all
passed those checks, the separately validated published schema and the
consumer's size/count policies, while seven forged-graph controls failed.
That static check is complemented by the companion consumer's Ajv, Vue, static
browser and actual native live-bridge suites, described in the integration guide.
Consumer policy additionally caps
imports at 4 MiB, 4,096 arguments and 65,536 defeat witnesses, so arbitrary
raised producer budgets do not guarantee browser import acceptance.

These checks establish the tested finite profile. They do not establish every
ASPIC+ rationality condition, truth of external premises, deductive validity of
an authored strict rule or general correspondence between authored and formal
assessment. Implemented runtime, MCP and visualiser boundaries are recorded in
[port status](port-status.md).
