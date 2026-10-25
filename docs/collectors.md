# Native evidence collectors

For OpenAPI, Datadog, Prometheus, Loki, GitHub CI and retained report profiles,
see [API and delivery evidence](api-collectors.md).
These additions are selected through catalogue version 3 or `describe-api`.

`eal-collector` supplies read-only Kubernetes, GitHub and Tempo wrappers for the
existing `command` collector contract. Install it beside the runtime companions.

Authoring clients can read versioned capability descriptions through
`eal-collector describe`. See [collector authoring contracts](collector-contracts.md)
for the input schemas and operator-owned command templates.

```sh
cargo install --path crates/eal-process --locked
cargo install --path crates/eal-collectors --locked
eal-collector --help
```

## Register your own collector

Users can register any executable that implements the EAL collector contract.
The Kubernetes, GitHub and Tempo providers are bundled conveniences. A custom
collector runs through the generic `command` binding and requires no EAL rebuild.

Add its name, version and fixed executable arguments to operator-owned
`tools.toml`.

```toml
[tools.custom_status]
kind = "command"
version = "1"
argv = ["/absolute/path/custom-status", "--read-only"]
timeout_seconds = 30
max_output_bytes = 1048576
inherit_env = []
parallel_safe = false
```

Declare the same tool name and version in the EAL source. Its evidence selects
the tool and supplies the request input.

```eal
language "EAL/3"
environment production {
  require stage == "production"
}
tool custom_status {
  version "1"
}
evidence service_status {
  tool custom_status
  kind service_health
  environment production
  max_age 60
  input {"service":"example"}
  require healthy == true
}
```

With context `{"stage":"production"}`, the executable receives this JSON
request on stdin.

```json
{"evidence_id":"service_status","environment":"production","tool":"custom_status","tool_version":"1","input":{"service":"example"},"context":{"stage":"production"}}
```

Write one strict JSON observation to stdout. The `value` field is mandatory.

```json
{"value":{"healthy":true}}
```

The optional envelope fields are `observed_at`, `context`, `request` and
`details`. Supplied `context` must match the requested scoped context. Supplied
`request` must match the acquisition object containing `tool`, `tool_version`,
`input` and `context`. Include the original `observed_at` for earlier or cached
measurements. Return a successful measured negative in `value`, such as
`{"healthy":false}`. Report acquisition failure with a nonzero exit status.
Keep diagnostics on stderr so stdout contains only the observation.

Pass this registry through `--registry tools.toml` to the CLI or MCP server.
Attach the evidence to an argument and use `eal_collect`, `eal_collect_claim` or
registered `eal_assess_known` through the existing MCP interface. Registering a
collector means adding its operator binding. The separate `eal register`
command registers an EAL source and its selected claims in the source catalogue.

For an existing report, a `json_file` binding can import an observation instead.
File envelopes also require `observed_at`, `context` and `request` to preserve
measurement age and identity. See [trusted collector configuration and runtime
bounds](runtime.md#trusted-collectors) for environment selection, dependency pins,
output limits and reuse.

## Bundled collectors

The operator fixes executable paths, endpoint, inventory and resource selectors
in `tools.toml`. An EAL source supplies the corresponding bounded input. A
selector mismatch fails before external execution. The wrapper reads one strict
JSON request on stdin and writes one EAL observation envelope on success. A
failure exits with status 2 and leaves stdout empty. Fixed error messages retain
failure categories without copying command stderr, credentials or HTTP bodies.

| Provider | Fixed operator selection | Admitted EAL input | Observation |
| --- | --- | --- | --- |
| `kubernetes` | `kubectl`, supervisor, inventory, resource, kind, namespace and name | Matching `resource`, `namespace` and `name` | Resource presence and selected identity/status in every inventory context |
| `github` | `gh`, supervisor, repository and exact 40-character commit SHA | Matching `repository` and `commit` | Exact commit and tree identity |
| `tempo` | Search endpoint, inventory, rollout kind/name, attribute mapping and credential-variable names | Integer `start_unix` and `end_unix` | Matching trace identities and search metadata for every mapped telemetry cluster |

The example in [`examples/catalog-sync-operator`](../examples/catalog-sync-operator)
contains a source, registry, scoped context and inventory template. Replace every
`REPLACE` value, choose the actual namespace/API group and replace the source's
`1`/`2` search window with the intended Unix-second interval before collecting.
Set both `inventory_sha256` predicates to the same approved inventory file hash.
This makes evidence from different inventory bytes fail the selected argument.
Select `rollout_observed` for the Kubernetes/Tempo claim. The separate
`revision_identified` claim exercises exact GitHub lookup. Establishing which
revision was deployed additionally requires a deployment-controller revision or
immutable workload-image correspondence.

```sh
eal --workspace ./examples/catalog-sync-operator \
  --registry ./examples/catalog-sync-operator/tools.toml \
  collect case.eal --claim rollout_observed --context @context.json
# Supply the returned collection_id in the next command.
eal --workspace ./examples/catalog-sync-operator \
  --registry ./examples/catalog-sync-operator/tools.toml \
  reason case.eal --collection COLLECTION_ID --context @context.json
```

Alternatively register the source/context and use `assess-known`, or expose the
registered claim through `eal_assess_known`. Existing observation identity,
freshness, binding checks and reuse apply unchanged.

## Inventory and identity

An inventory contains a nonempty `revision` and 1–256 clusters. Each cluster has
distinct `id`, `context` and `tempo_cluster` strings. Its SHA-256 and revision
appear in each result. The operator establishes the inventory's production
authority and completeness. The wrapper checks the declared entries and cannot
discover omitted production clusters. `tools.toml` should pin the inventory and
collector/dependency files with `pinned_files` after deployment.

The Kubernetes wrapper executes `kubectl get` with fixed selectors,
`--ignore-not-found` and JSON output. A successful empty response records measured
resource absence. Authentication failures, nonzero exits and malformed responses
produce acquisition failures. Returned resources must match name, namespace,
kind and API group and retain UID, resourceVersion, generation and status.
Resource existence and controller health remain distinct properties. The
example's positive predicate concerns existence.

## Tempo scope and limitations

The wrapper uses the official
[Tempo HTTP search API](https://grafana.com/docs/tempo/latest/api_docs/#search).
It URL-encodes `q`, `start`, `end` and `limit`. A single spanset combines the
operator's rollout kind/name with the inventory's cluster attribute mapping.
Attribute names follow the scoped identifiers documented in
[TraceQL construction](https://grafana.com/docs/tempo/latest/traceql/construct-traceql-queries/).
Each admitted `span.` or `resource.` attribute has nonempty dotted components.
An empty component fails configuration admission before an HTTP request.
The admitted window is at most 24 hours by default. Operators can choose up to
30 days. The result limit is 1–1000 per cluster.

Tempo returns the first matching results and documents that repeated searches
can return different subsets. The wrapper therefore always retains
`search_exhaustive = false` and `negative_inference = "unavailable"`. A nonempty
result supports an existential match within its declared window. Empty results
record that this query returned no match. They leave overall trace absence
unresolved. Limit hits and reported partial-job counts are retained separately.
The HTTP search response contains trace summaries. The retained trace ID
identifies a returned trace matching the query. Fetching its complete spans
requires a separate trace-by-ID observation.
HTTP errors, reported error fields, missing explicit `traces`, invalid trace IDs,
oversized responses and transfers that exceed the deadline fail acquisition.
The retained `all_matched` predicate supports positive bounded queryability
only. It does not establish trace freshness beyond the supplied interval or
correspondence to the Kubernetes resource's namespace, UID or deployed revision.

## Process, network and credential ownership

Command variants reuse the native runtime supervisor for deadlines, combined
stream bounds and process-group cleanup. Their child environments start empty.
Repeat `--inherit-env NAME` for variables required by kubeconfig authentication
plugins or GitHub credentials. The surrounding tool binding must also allow
those names through `inherit_env`. Pin plugin executables when they affect the
observation's reproducibility. The aggregate input, response and envelope bounds
remain finite across the complete inventory.

Tempo credentials come from operator-named environment variables. Bearer and
optional `X-Scope-OrgID` headers remain outside stored configuration values and
output metadata. HTTPS is required for remote endpoints. Plain HTTP is admitted
for loopback integration tests and local query frontends. Redirects and proxy
discovery are disabled. An operator-selected endpoint remains a trusted evidence
source. These wrappers retain observations under that trust assumption.

Known credential inspection uses the shared bounded JSON inspector. It covers
values and keys containing successfully decoded JSON text, including repeated
encoding and Unicode escapes. Matching numeric, boolean and null values also
fail acquisition. Ambiguous encoded JSON and inspection exhaustion return a fixed
error and no observation.
The same captured credentials guard the final observation envelope, including
generated query metadata and supplied request/context fields. Credential values
are never re-read or included in that envelope. Each inspection allows a combined
depth of 128, 100,000 values/keys and 128 MiB of aggregate scan/decode input.
Unrelated JSON-like prose remains unchanged. The inspector detects known values
and does not establish the absence of arbitrary transformed exfiltration.

## Preservation and verification

The shared abstraction is the existing EAL acquisition envelope. Each provider
retains the distinctions required by its claim: resource identity/presence,
commit/tree identity or time-scoped existential trace matches. Provider-specific
selection stays in its own module. Selection dispatch contains no provider
algorithm. Frozen evaluator, storage and MCP contracts remain unchanged.

Inventory reads use the runtime's bounded regular-file reader. FIFO and device
paths fail before reading. Parent aliases are resolved while the final file is
opened without following symlinks. One aggregate deadline starts before provider
validation and inventory loading and supplies each subsequent external request.
The outer runtime supervisor remains the process containment boundary.

Local tests exercise real supervised fake commands and loopback HTTP responses.
They cover exact arguments, environment restriction, mismatched selectors,
inventory duplication, positive/empty observations, nonzero exits, malformed
payloads, deadlines, matching envelopes and bounded TraceQL construction. Live
cluster and Tempo credentials are required for environment acceptance.

## Native contract versions and resource scope

`eal-collector describe` preserves the published version 1 catalogue. Select the
revised contracts with `eal-collector describe --catalogue-version 2`. The emitted
commands include `--contract-version 2`, and requests must declare tool version
`2`. Version 1 descriptors and their namespaced input/generation requirement
remain supported until `eal-collectors` 1.0. New consumers should select version 2.
The wire catalogue itself remains `eal.collector-catalogue/1`.

Version 2 offers separate `kubernetes` and `kubernetes_cluster` descriptors.
Namespaced input contains `resource`, `scope="namespaced"`, `namespace` and `name`.
Cluster-scoped input contains `resource`, `scope="cluster"` and `name`.
The operator command selects the same scope with `--scope`. Missing namespace
never grants cluster scope. All-namespaces listing is outside these named-resource
contracts. Both variants check name, kind, API group, UID and resource version.
Generation is optional in version 2, and absent generation remains absent.
A supplied generation must be a nonnegative signed 64-bit integer.

Configurations become immutable after admission. `Config::new` validates typed
`ProviderConfig` records and `CollectorLimits` using the same invariants as CLI
parsing. Raw option maps are local to CLI conversion. Consumers replacing the
previous mutable `Config` fields should construct a provider record and submit it
through `Config::new`. `with_limits` accepts only an admitted `CollectorLimits`.
The legacy Tempo `--inherit-env` flag remains accepted until 1.0 and has no
command effect because that provider uses HTTP. Its credentials continue to use
`--token-env` and `--tenant-env`.
Operator executable, supervisor and inventory paths require bounded absolute
UTF-8 paths. Programmatic admission rejects non-UTF-8 paths before their use,
and command execution preserves the selected executable spelling exactly.

## Observation timing

Envelope `details.acquisition_interval` records acquisition start and completion,
an oldest-observation bound and whether wall time moved backwards. The envelope
`observed_at` uses the earliest sampled acquisition time, so a slow sequential
fleet read cannot make the first observation appear as recent as the last read.
Version 2 Kubernetes and Tempo results also retain an acquisition interval for
each cluster, including successful measured absence or an empty search.
These intervals describe when the response was acquired. Tempo's query window
and trace event times retain their separate meanings.

Fleet readings are sequential acquisitions. They do not establish a simultaneous
fleet state. Partial acquisition failure returns an error and emits no successful
aggregate observation. Reuse retains the original `observed_at` and intervals.
Historical observations without intervals retain their recorded timestamp and
unknown timing detail. Monotonic `Instant` deadlines remain independent of wall
clock metadata. A backwards wall clock is reported and the oldest bound remains
conservative, rather than silently rejuvenating an observation.

The optional collector package depends on the narrow process and adapter packages.
It does not require the EAL runtime, language parser or database for collection.

## Shared graph quantity projection

An acquisition response can itself be an array without being a graph fan-out.
Use derivation `project_value` to select named fields against that whole response:
`{"kind":"project_value","fields":{"uid":"/0/uid"}}` selects the first item's
UID. The existing `project` operation preserves its per-record array behavior:
after a `for_each` acquisition, `{"kind":"project","fields":{"uid":"/uid"}}`
produces an array of UIDs, including an empty array for an empty fan-out. Both
operations retain source lineage, original observation times and coverage; absent
pointers leave evaluation pending. Graph authors must select the operation from
the acquisition's declared `for_each`, rather than infer fan-out from JSON shape.

## Collected reasoning inputs

A graph evaluation may use `{"kind":"method_input","method":"inductive/1"}`
with exactly one input reference. This admits an already collected
`EAL/typed-input/1` envelope against the installed method's input contract,
quantity, unit and ordered time interval. The evaluation records
`method_result_evaluated: false` and `scientific_adequacy_verified: false`.
Admission supplies no conclusion, no proof of sampling or experimental design,
and no validation of prose correspondence. Missing evidence, an absent causal
model, an unknown method or a malformed input remains pending with a diagnostic.

The ordinary `support-graph` collector can return that input for an EAL typed
claim using this acquisition request input:

```json
{
  "schema": "engarg.reasoning-input/1",
  "case_digest": "CASE_SHA256",
  "execution_plan_sha256": "PLAN_SHA256",
  "requirement_id": "sample_input",
  "output": {"step_id": "sample_envelope", "pointer": ""},
  "premise_bindings": [
    {
      "premise_id": "sample_ground",
      "input_pointer": "/trials",
      "requirement_id": "sample_support",
      "correspondence": {"kind": "identity_value", "source_input_index": 0, "source_pointer": ""}
    }
  ]
}
```

The two digests must be exact lower-case SHA-256 identities and match the export
context and pinned plan. `output` must exactly match the input reference of the
selected method-input evaluation. That evaluation must be satisfied. The
selected source or derived value must itself be the complete typed envelope;
the request cannot supply replacement metadata or a replacement payload.
Projection retains the original oldest observation time, immutable source and
observation identities, and request/context correspondence. It replays pure
derivations and checks projected acquisition values against retained originals.
Premise bindings execute the union of the input requirement's and each named
premise requirement's acquisition closures under the same deadline and request
budget. Every premise criterion must be satisfied. `identity_value` compares the
payload's selected formal operand with the actual selected requirement input,
using the declared input index and a relative `source_pointer`. Comparison uses
canonical exact JSON without coercion or implicit unit conversion. A payload
claiming 90 successes cannot use a satisfied but different 10-success measurement.
Missing evidence, a false criterion or unequal values cannot emit a method input.
The combined observation retains the oldest original clock and all consulted
source and observation identities. Input admission cannot certify its own premises.

For a deductive `/premises/N` operand, `{"kind":"accepted_premise"}` instead
requires a separately collected, satisfied criterion. The mapping from that
criterion to the declared formula remains authored. Other modes require actual
value identity. Independent premise support and named assumptions must also be
dependencies of the EAL argument. A pointer's presence alone establishes neither
truth nor correspondence with prose.

EAL then checks the envelope against the claim's subject, quantity, unit, scope,
interval and formal query, runs the selected kernel, and evaluates the claim's
result predicate. A valid input whose computed result fails that predicate is
retained as a measured result. It does not become supported merely because input
admission succeeded. The earlier `engarg.support-acquisition/1` request continues
to return its requirement-evaluation wrapper unchanged.

## Discovery dependencies

A dependency binding without `role` retains its original record-selection
meaning and serialises unchanged. Discovery metadata can instead declare an
explicit role with scope correspondence:

```json
{
  "output": {"step_id": "api_discovery", "pointer": "/api_version"},
  "argument_pointer": "/api_version",
  "role": {
    "kind": "discovery",
    "scope_bindings": {"/cluster": "/cluster", "/required_kind": "/kind"}
  }
}
```

The correspondence maps JSON pointers in the source's declared scope to equal
values in the consumer's declared scope. Every source selector must either occur
unchanged under the same key or have an explicit correspondence. Shared keys
must agree even when another mapping is supplied. Empty mappings, missing or
null selectors and contradictory identities fail plan validation before IO.
The example requires predeclared `/api_version` argument space and compatible
source and consumer scopes.

Execution admits discovery references through single-source `extract`,
`project` and `project_value` chains. Arithmetic, multi-source derivations,
fan-out origins and cyclic chains cannot relabel a selection as discovery.
The qualified planner must additionally establish that the source contract is a
discovery operation, retaining its original clocks and provenance. The runtime
checks the pinned source identities and scope correspondence. Discovery retains
its source and observation identities and original clock in the consumer's
lineage. Its metadata supplies no population coverage. A consumer's witness
therefore comes from its own measurement and any actual selection ancestors,
while an unspecified discovery coverage cannot erase that witness. This role
does not itself qualify a protocol or establish an empirical witness.

## Shared support execution deadlines

`eal-collector support-graph --plan plan.json --config evidence.json --store sources`
starts a monotonic budget on entry to the support facade. The default is 300 seconds.
`--timeout-ms` selects an integer budget from 1 to 86400000 milliseconds. Input and
configuration validation, source-store initialisation, acquisition, derivation and
final evaluation consume that same budget. An already-expired request admits no
source acquisition. `SourceStore::open_until` and `EvidenceExecutor::execute_until`
accept the same absolute deadline for programmatic callers. Synchronous store
opening uses a separate finite 30-second lock-wait budget.

Key initialisation and source-store I/O run on blocking workers. Lock waits poll
without blocking the async scheduler and stop admitting work after expiry. An I/O
operation admitted before expiry can finish after the caller returns. Its worker
keeps the key or source lease until completion. Reconciliation refuses an active
source lease, and an uncertain acquisition remains marked for explicit operator
reconciliation before another acquisition can begin.

An immutable observation snapshot is published and synced before the source's
reuse index exposes its identity. Each publication checks the deadline before
renaming. Expiry after snapshot publication can leave a retained snapshot without
a new reuse index. Expiry after an admitted rename can expose the index while the
invocation reports exhaustion. The pending marker retains the uncertain outcome
until reconciliation or successful completion. Retained observations keep their
original timestamps and signatures.

Execution reports carry `budget_exhausted` separately from criterion satisfaction.
An expired invocation has `complete: false`, including expiry during final
assessment. An independently admissible observation can still satisfy a criterion.
Collector projections expose `execution_complete` and `budget_exhausted` alongside
that criterion result. Freshness uses the actual lookup time unless the caller
supplies an explicit assessment cut-off. The maximum age is inclusive, and a future
observation or an age above that bound leaves the affected criterion pending.
Elapsed-time tests include documented scheduler tolerance. These bounds describe
cooperative admission and caller response, rather than cancellation of filesystem
operations already in progress.

A fan-out shared by witness and complete-enumeration or statistical consumers
continues after its first match to seek the requested population coverage. A
requirement projection that needs only the witness can still stop early. If later
acquisition fails or exhausts the budget, the retained positive prefix keeps its
original observation age and lineage. Pure final assessment can use that prefix
after the deadline, including its applicability and challenge checks. Population
consumers remain pending until their own coverage conditions hold. This final
assessment admits no further acquisition or publication.

After expiry, recovery admits at most 128 structural or identity derivations.
It shares a 16 MiB budget for conservatively estimated JSON input and output
bytes across those derivations. A bounded counting writer checks selected values
before cloning them. Projection estimates include the number of output fields,
so repeated `collect` or `project` operations cannot expand without consuming
the budget. Numeric transforms remain incomplete after expiry because their
intermediate arithmetic cost needs a separate bound. Completed observations and
their reported request and byte counts survive interruption, including responses
received after the first witness. These are bounds on recovery work and encoded
values, rather than a strict wall-clock grace period or a process-memory limit.
