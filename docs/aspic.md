# Finite ASPIC reasoning

`eal-aspic` implements the `argumentation/aspic/2` theory contract,
`EAL/4-compiled-aspic/5` source translation and `aspic-view/3` export. These are
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
| `last_link_rank` | Minimum rank among last defeasible rules. Ordinary premises when both last-rule sets are empty |
| `last_link_partial` | Explicit acyclic premise/rule priorities with strict elitist set lifting over the same last-link sets |

Empty comparison sets are strongest. Tied and incomparable sets retain attacks.
The reported `strength` remains the minimum numerical fallible rank under every
policy, so it is an inspection field when a last-link policy is selected.
Contrary strict/axiom closure and extensions containing contrary conclusions
are rejected by this finite contract.

Grounded semantics returns the least fixed-point extension. Preferred semantics
enumerates every maximal admissible extension. Stable semantics enumerates
every conflict-free extension that attacks every outside argument. Search is
exact within the captured budget. An exhausted search returns no partial
extension family.

| Query outcome | Criterion |
| --- | --- |
| `accepted` | Some extension contains a goal argument for a credulous query. Every extension contains some goal argument for a sceptical query |
| `rejected` | Every goal argument is attacked in every extension |
| `undecided` | A goal argument exists and neither selected acceptance nor rejection holds |
| `unconstructed` | The goal has no constructed argument and the extension family is nonempty |
| `no_extension` | The selected semantics has no extension. Neither query Boolean is true |

Grounded labels and grounded query fields remain present when preferred or
stable semantics is selected. Sceptical acceptance quantifies over conclusions,
so different extensions may contain different derivations of the same goal.

## Checked EAL compilation

Compilation accepts exact source text, observations, context, an explicit
assessment time, registry and limits. It first performs ordinary EAL assessment.
Available evidence becomes an ordinary premise. Locally usable authored routes
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
than authenticated observations. The [native visualiser adapter](#visualiser-integration)
connects the explicit theory contract to the companion editor.

## Native commands and APIs

```sh
cargo run --locked -p eal-cli --bin eal-core -- aspic examples/aspic-theory.json
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
be `-` for standard input. Output goes to standard output. Method set
`aspic` installs the default eight methods plus ASPIC.  `all` also installs the
four engineering methods. Explicit theory options belong inside its JSON
`options` object. The [production service](applications.md) also exposes checked
formal operations. `eal-aspic-solve` accepts the same theory request directly on
stdin and emits the checked view through its separately bounded process protocol.

Library callers use `solve_aspic`, `validate_theory`, `CompileOptions`,
`compile_eal_aspic`, `export_aspic_view`, `aspic_registry` or `with_aspic`.
Native callbacks receive `&ExecutionLimits` and return typed `MethodError`.
This Rust API makes captured budgets available without implicit global state.
ASPIC failures distinguish `AspicError::Invalid` from `AspicError::Incomplete`.
The CLI emits a failed `incomplete` response with empty `details` and exit code 1
for unfinished formal computation. Invalid requests produce an error and exit
code 2. Method assessment retains `status: "incomplete"`, causing the enclosing
authored assessment to set `complete: false`.

Local graph `resource_limit` diagnostics also produce typed `Incomplete` during
compilation. The error category identifies unfinished computation.

Formal construction, arguments, antecedents, atoms, contrary pairs, defeat
witnesses and extension search have independent budgets. Public solver inputs
also enforce the selected JSON traversal limits, including the native depth
ceiling. Direct-call admission enforces the selected JSON budgets. Compiler and
exporter also enforce the ASPIC contract's 256 KiB input and 1 MiB response limits.

Native implementation identity identifies the Rust source bundle and its
semantic helpers. The core
checks deterministic work and data budgets. The [runtime supervisor](runtime.md)
owns process deadlines and resource limits for supervised methods. The standalone
visualiser adapter remains a pure process whose HTTP owner enforces its deadline
and cancellation. Timeout metadata alone provides neither guarantee.

## Verification scope

Rust tests check authored synthetic observations, source and observation
identities, freshness, typed queries and incomplete computations. Independent
subset definitions check complete extension families, query quantifiers, defeat
witnesses and grounded labels for every directed graph through three nodes,
including self-attacks. Export tests recompute results and reject forged labels,
subarguments and provenance.

The native consumer import check uses Node 24 or later and the companion
checkout's installed dependencies (`npm ci` in that checkout):

```sh
cargo run --locked -p eal-devtools --bin eal-smoke -- consumer \
  --consumer /absolute/consumer/src/domain/aspic.ts \
  --binary /absolute/installed/bin/eal-core \
  --node /absolute/node
```

The check verifies the pinned consumer module and adjacent schema Git blobs,
then calls the complete `parseAspicView` importer with its actual Ajv dependency.
Only the module's import declarations are relocated for Node; its function bodies
remain unchanged. Native scenarios exercise every supported semantics, query
mode and preference policy, including unresolved predicates and unavailable
dependencies. The check also validates the producer schema and requires rejection
of seven forged graphs and three malformed metadata controls. Its report
identifies the executable hash and distinguishes freshly generated views from
explicitly supplied cached views. The sibling `eal-process-supervisor` bounds
each child process. `--supervisor FILE` can select it explicitly.

Consumer imports are limited to 4 MiB, 4,096 arguments and 65,536 defeat
witnesses. The companion consumer's TypeScript, schema and browser checks cover
the remaining frontend boundaries, as described in the
[visualiser guide](#visualiser-integration).

These checks support the exercised finite contract. The truth of external
premises, validity of an authored strict rule and adequacy of the selected
formalisation require their own evidence.


## Visualiser integration

`eal-aspic-solve` accepts an explicit finite theory and emits a checked
`aspic-view/3` snapshot. The companion
[`aspic_visualisation`](https://github.com/emmett08/aspic_visualisation)
development server selects this executable explicitly. Rust owns argument
construction, extension search and recomputed export. The server owns HTTP,
request deadlines, concurrency and child cleanup. The browser retains its
schema and graph checks for both imported and freshly solved snapshots.

The companion development server requires its native solver bridge and a
configured absolute executable path. Static graph validation is checked against
the selected consumer source file and its exact Git blob identity.

### Installation and request protocol

Install `eal-cli` with the pinned Rust toolchain:

```sh
cargo install --path crates/eal-cli --locked
```

The standalone command reads one UTF-8 JSON object from stdin containing exactly
`theory`. Its theory follows the existing
[`argumentation/aspic/2` contract](aspic.md). It creates no catalogue, database or
collector process. A successful response contains only one checked view and a
newline on stdout, with exit status 0.

```sh
/absolute/installed/bin/eal-aspic-solve < theory-request.json > view.json
```

The adapter rejects duplicate JSON fields, trailing input, invalid UTF-8 and
unsupported theories. It reads at most 64 KiB plus one overflow-detection byte
and emits at most 4 MiB. Normal solver budgets still apply inside those byte
limits. Output is buffered until solving, export recomputation, schema checks
and the response-size check have succeeded.

Request, calculation and export failures leave stdout empty and emit one bounded
JSON object on stderr:

```json
{"error":{"code":"incomplete","message":"The selected resource budget was exhausted"}}
```

| Error code | Exit status | Meaning |
| --- | ---: | --- |
| `invalid_request` | 2 | The request, theory or export is outside the supported contract. |
| `incomplete` | 1 | Computation or response-size limits prevented a complete delivered view. |
| `unavailable` | 3 | The selected file or process IO was unavailable. |

A complete stable-semantics result can have no extensions. That result retains
`query_status: "no_extension"` and succeeds. An incomplete calculation supplies
no graph. A zero-argument result cannot satisfy the view schema's
minimum of one argument and is rejected during export.

An operator can supply `--limits /absolute/path/limits.toml`. The file must be a
regular TOML file of at most 16 KiB containing a `[limits]` table. These limits
select the finite construction and search limits. Browser requests cannot
select a budget file, raise the process byte limits or change server options.
The executable reads no ambient `EAL_LIMITS_FILE` setting.

### Companion development server

With the consumer integration checked out, enable its optional local editor:

```sh
ASPIC_SOLVER=1 \
ASPIC_SOLVER_EXECUTABLE=/absolute/installed/bin/eal-aspic-solve \
npm run dev -- --host 127.0.0.1
```

The configured executable must be an absolute path. There is no executable-name
search. The child receives a closed environment with an empty `PATH`. No
interpreter, provider credential or collector configuration is needed. Optional
server setting `EAL_LIMITS_FILE=/absolute/path/limits.toml` becomes the explicit
`--limits` argument.

The server accepts one same-origin JSON upload at a time, bounds upload and
solver duration, and terminates its owned solver after timeout or disconnect.
It validates the complete returned graph before returning a successful HTTP
response. A failed edit preserves the last accepted graph and its session
events. The optional bridge is disabled by default and absent from the static
production build. It remains a trusted local development facility.

### Check the pinned integration

The [workspace and installed checks](development.md) verify the EAL side of this
boundary. The consumer command above checks authored native cases through the
full schema, policy and graph import boundary, including ten rejection controls.
Node 24 or later executes the importer through the bounded supervisor.

That check pins consumer Git blob `f49b95a894e8c3cb6634d1dafeb8ca6de9b4128a` and
schema Git blob `e04b18e1cac3a33a532d152cdba52b038e655540` at
revision `6c8c5c2e052f3d27c0382f6de37eee1e96d0a69d`. A downstream validator change
requires an explicit identity update and renewed acceptance. The following
frontend commands describe that selected companion integration.

In the consumer checkout, run the declared TypeScript, unit, build and browser
checks, followed by the actual native bridge suite:

```sh
npm ci
npm run typecheck
npm test
npm run build
npx playwright install chromium
npm run test:browser
ASPIC_SOLVER_EXECUTABLE=/absolute/installed/bin/eal-aspic-solve npm run test:browser:native
```

The static suite exercises file imports and graph inspection. The native suite
submits theories through the local HTTP bridge and executable. These checks
cover software integration. Claims about external observations and decisions
require separate evidence.

Return to [applications](applications.md) for retained authored assessment or
to the [documentation path](index.md).
