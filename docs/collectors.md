# Native evidence collectors

`eal-collector` supplies read-only Kubernetes, GitHub and Tempo wrappers for the
existing `command` collector contract. Install it beside the runtime companions.

```sh
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

## Preservation and verification

The shared abstraction is the existing EAL acquisition envelope. Each provider
retains the distinctions required by its claim: resource identity/presence,
commit/tree identity or time-scoped existential trace matches. Provider-specific
selection stays in its own module. Selection dispatch contains no provider
algorithm. Frozen evaluator, storage and MCP contracts remain unchanged.

Local tests exercise real supervised fake commands and loopback HTTP responses.
They cover exact arguments, environment restriction, mismatched selectors,
inventory duplication, positive/empty observations, nonzero exits, malformed
payloads, deadlines, matching envelopes and bounded TraceQL construction. Live
cluster and Tempo credentials are required for environment acceptance.
