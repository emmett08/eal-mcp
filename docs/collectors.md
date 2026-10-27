# Acquire a measurement that the argument can interpret

A collector bridges an external system and a declared evidence requirement. Its
result must preserve the measured value, its original time and the request it
answers. The argument can then decide whether that observation supports a claim.
This boundary is shared by custom instruments and the bundled infrastructure
adapters.

Start with the [direct manifest](direct-tools.md) when an existing operation can
supply the observation. This chapter defines the command envelope, then applies
it to Kubernetes, GitHub and Tempo. [API and delivery evidence](api-collectors.md)
continues with telemetry and CI reports.

## Write a command collector


A custom collector implements the JSON stdin/stdout contract and runs through
a `command` binding. Its implementation can use any language whose runtime the
operator has installed. EAL owns admission, process lifetime and observation
retention, so the collector can concentrate on its source.

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
language "EAL/4"
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

Attach the evidence to an argument, then run the package with
`eal-host run case.eal --tools tools.toml --workspace run`. For staged execution,
pass the same manifest with `--registry tools.toml` to the CLI or MCP server.
Adding the binding makes the operation available to that runtime. The optional
`eal register` command separately records a source and selected claims for
repeated assessment.

For an existing report, a `json_file` binding can import an observation instead.
File envelopes also require `observed_at`, `context` and `request` to preserve
measurement age and identity. See [trusted collector configuration and runtime
bounds](runtime.md#bind-a-collector-to-an-evidence-request) for environment selection, dependency pins,
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

The [deployment example](catalog-sync-operator.md) contains a source, manifest,
scoped context and inventory template. Replace every
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

## Check the boundary before trusting a deployment

The shared abstraction is the existing EAL acquisition envelope. Each provider
retains the distinctions required by its claim: resource identity/presence,
commit/tree identity or time-scoped existential trace matches. Provider-specific
selection stays in its own module. Selection dispatch contains no provider
algorithm. The same envelope preserves these distinctions across evaluator, storage and MCP
interfaces.

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
`2`. Version 1 remains the compatibility default, including its namespaced input and
generation requirement. Select version 2 for explicit resource scope and optional
generation.
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
parsing. Raw option maps are local to CLI conversion. Library callers construct a provider record and submit it through `Config::new`. `with_limits` accepts only an admitted `CollectorLimits`.
Tempo uses HTTP and rejects `--inherit-env`. Its credentials use
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

## Use the observation in the argument

[Executable programmes](executable-programs.md) connects these observations to
EAL/4 bindings, derived values and method queries. A count over a supplied array
establishes a property of that array. Extending it to a population requires a
source-completeness argument and its own evidence. Retained parent identities
and measurement ages make that inference inspectable when the result is reused.

## Optional authoring discovery

`eal-collector evidence-discover --config evidence.json` inspects configured MCP
and native candidates within explicit transport budgets. Discovery assists an
author in selecting a concrete binding. It supplies no execution authority and
is not required by `eal-host run`. `prepare-native-provider` validates configured
candidate metadata without invoking or qualifying the selected operation.


`eal-collector describe` publishes optional native descriptions without acquiring
evidence. Select catalogue version 3 for the API and delivery adapters, or use
`describe-api` for a pinned API source. The [authoring contract](collector-contracts.md)
defines the descriptor schema for clients that use it.

Return to the [documentation path](index.md).
