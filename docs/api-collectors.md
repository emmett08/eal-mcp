# API and delivery evidence

Catalogue version 3 adds Datadog, Prometheus, Loki, GitHub CI and retained report
measurements. Versions 1 and 2 retain their published descriptors and defaults.
The acquisition envelope, EAL grammar and runtime registry format stay unchanged.

```sh
eal-collector describe --catalogue-version 3 > engineering.collectors.json
```

## API operator configuration

API providers consume a bounded JSON file through `--config` and its SHA-256
through `--config-sha256`. The file fixes the origin, measurement and query.
An acquisition request must match that query and supply its time window.
Credentials are selected by environment-variable name and captured once.

For example, save the following as `datadog-source.json` after selecting your
actual Datadog site and service scope.

```json
{
  "schema": "eal.api-source/1",
  "origin": "https://api.datadoghq.eu",
  "headers_env": {
    "DD-API-KEY": "DD_API_KEY",
    "DD-APPLICATION-KEY": "DD_APP_KEY"
  },
  "max_window_seconds": 86400,
  "max_points": 10000,
  "source": {
    "kind": "datadog_logs",
    "query": "service:catalog-sync-operator env:production",
    "limit": 100
  }
}
```

Describe the admitted configuration before authoring an argument.
This command reads the selected files and performs no acquisition.
The descriptor carries the fixed query, configuration pin and permitted
credential-variable names. Executable selection remains an operator choice.

```sh
source_config="$PWD/datadog-source.json"
source_digest=$(sha256sum "$source_config" | cut -d ' ' -f 1)
eal-collector describe-api datadog --contract-version 2 \
  --measurement datadog_logs \
  --config "$source_config" --config-sha256 "$source_digest" \
  > datadog.collectors.json
```

Select this catalogue with EngArg's `--collector-catalogue` option.
Its required input is the following shape, with actual ordered Unix seconds
chosen for the investigation.

```json
{
  "query": "service:catalog-sync-operator env:production",
  "start_unix": 1792800000,
  "end_unix": 1792803600
}
```

The resulting EAL tool uses version `2` and an acquisition predicate of
`collected == true`. Claim-specific acceptance belongs in the argument and its
review obligations. When collecting through EngArg, forward approved variables
with `--collector-env DD_API_KEY --collector-env DD_APP_KEY`.
The runtime registry must permit the same inherited names.

| Provider command | Measurement / source kind | Additional operator fields |
| --- | --- | --- |
| `datadog` | `datadog_metrics` | `query`, `interval_seconds` |
| `datadog` | `datadog_logs` | `query`, `limit` |
| `datadog` | `datadog_spans` | `query`, `limit` |
| `datadog` | `datadog_monitor` | Positive integer `id` |
| `datadog` | `datadog_slo` | Literal SLO `id` |
| `prometheus` | `prometheus_range` / `prometheus` | `query`, `step_seconds` |
| `loki` | `loki_search` / `loki` | Literal log query and `limit` |

Set `source.kind` to the source kind in the table.
Prometheus-compatible query frontends, including Mimir, use the Prometheus
measurement. `bearer_env` optionally selects bearer authentication.
`headers_env` can select a tenant header through an operator-owned variable.
The origin is HTTPS with an empty path. HTTP is admitted for loopback fixtures.

API input contains `query`, `start_unix` and `end_unix` for query profiles.
Monitor input contains only its integer `id`. SLO input contains its `id` and
the ordered window. The configured maximum window is 1 second to 30 days.
Resolution and window admission bound requested metric points. Returned metric
rows have explicit sample arrays bounded by `max_points`.

## Retained meaning and operational bounds

The common HTTP adapter disables redirects and proxy discovery. It restricts
credentials to the configured origin and enforces one aggregate monotonic
deadline and response-byte allowance. Responses and complete observation
envelopes pass the shared credential inspector. HTTP errors, including 429,
fail acquisition with a status category. Response bodies stay out of diagnostics.
Automatic retry is absent, leaving retry timing and cost under caller control.

Datadog metrics use the v2 timeseries endpoint. Requested interval and returned
source metadata remain inspectable. Zero and null samples retain their original
values. Prometheus range queries preserve warnings and informational annotations.
The initial Prometheus profile admits float series. Native histogram series
require a separate measurement profile.

Datadog searches and Loki return one bounded page. Cursor and limit information
remain in the result. `search_exhaustive` is always false and
`negative_inference` is `unavailable`. Missing matches leave overall absence
unresolved. Search indexing, sampling, retention and permissions remain relevant
to claim applicability even when a returned page is complete.

Telemetry `observed_at` records the conservative acquisition start. Query windows
and event timestamps retain their separate meaning. Retained reports use their
original measurement time. None of these observations establishes simultaneous
state across independent systems.

## OpenAPI operations

The OpenAPI adapter imports one explicitly approved operation from a pinned
JSON description using OpenAPI 3.0 or 3.1. It supports GET and approved read-only
POST with JSON bodies and successful JSON responses. Local references are
expanded under depth, node and byte limits. Reference cycles and remote
references fail admission. Unsupported serialization fails explicitly.

Select this source in an API configuration file.

```json
{
  "schema": "eal.api-source/1",
  "origin": "https://engineering.example.org",
  "bearer_env": "ENGINEERING_API_TOKEN",
  "source": {
    "kind": "openapi",
    "specification": "/absolute/operator/service.openapi.json",
    "specification_sha256": "REPLACE_WITH_ACTUAL_LOWERCASE_SHA256",
    "operation_id": "getServiceStatus",
    "read_only": true
  }
}
```

Run `describe-api openapi --contract-version 2 --measurement openapi_operation`
with the configuration path and digest as above. The generated descriptor has
an operation/specification-specific ID and derives its typed input schema.
Input properties are `path`, `query` and `body` as declared by the operation.
Optional properties can be omitted. Supplied null is validated against the schema.
Path and query parameters currently use primitive schemas. A path parameter
occupies one complete segment and is percent-encoded. Header credentials remain
operator configuration. OpenAPI `servers` entries grant no destination authority.

The collector validates the response against the schema for its actual declared
successful HTTP status. It retains the original response under `observation`.
`coverage = "unspecified"` leaves evidence completeness to a separate contract.
The operator's `read_only` declaration supplies the operation authority.
Interface descriptions alone cannot establish the behaviour of the remote server.

For a service behind a base path, select `source.base_path` explicitly, for
example `"/v1"`. A description declaring a non-root server path requires this
operator choice. The value prefixes operation paths and grants no new origin.
Root-mounted services can use an explicit empty string. This prevents a declared
server base path from silently disappearing when the destination is overridden.

## GitHub delivery observations

`github_checks` uses the existing supervised `gh` selection with a fixed
repository and 40-character commit SHA. It returns raw check states and
conclusions. `--max-pages` admits 1–10 pages of at most 100 checks each.
One deadline and aggregate byte limit cover all pages. Repeated identities or
changing reported totals fail acquisition. `reported_pages_complete` describes
the returned total. `search_exhaustive` remains false because GitHub's endpoint
has source-side coverage limits. An expected-checks contract is still needed
to establish that every required check exists and passed.

`github_workflow` additionally fixes `--run-id` and `--attempt`.
The request includes matching integer `run_id` and `attempt` fields.
The returned repository, commit, run and attempt must match before an observation
is emitted. Pending and failed results remain successful adverse observations.
CI evidence and deployed artefact correspondence require their respective links.
Both CI collectors capture the selected command environment once for the whole
acquisition. Forwarded values are treated as sensitive and inspected in the final
envelope, including encoded JSON text. Credentials remain outside retained results.

## Reports

`report --contract-version 2 --manifest ABSOLUTE --manifest-sha256 SHA256`
imports an operator-selected regular file through a separately pinned JSON manifest.
The manifest has schema `eal.report-source/1` and these required fields:

| Field | Meaning |
| --- | --- |
| `repository`, `revision` | Asserted source repository and exact 40-character revision |
| `artifact`, `artifact_sha256` | Absolute report path and exact byte pin |
| `observed_at` | Original UTC RFC3339 measurement time |
| `format` | `junit`, `sarif` or `json` |

EAL input supplies matching `revision` and `report_sha256`.
The observation labels revision correspondence as operator-asserted.
The file pin proves correspondence to the selected bytes. Build provenance
requires separate evidence connecting those bytes to the asserted revision.

JUnit imports UTF-8 XML 1.0 and counts explicit testcase outcomes.
Suite summary attributes remain in the raw report. Failures, errors and skipped
cases remain distinct. Unsupported outcome elements and ambiguous outcomes fail
admission. Document types and custom entities are rejected. Limits are 1 MiB,
128 levels, 100,000 events and 10,000 testcases.
SARIF 2.1.0 imports require explicit runs, tool identities and result arrays.
The complete report retains invocation failures, findings and baseline details.
Generic JSON imports retain an object, which can carry benchmark or SBOM data.
Domain-specific interpretation and signature verification need their own contracts.

## Verification

Loopback HTTP fixtures check exact requests, response conformance, measured
negatives, query admission, credential inspection and source limits.
Supervised fake GitHub commands check revision/attempt correspondence.
Pinned report fixtures check adverse outcomes, original age and changed bytes.
These tests establish local contracts. Live service acceptance requires the
configured endpoint, actual permissions and relevant production observations.

Primary API references are the
[OpenAPI specification](https://spec.openapis.org/oas/v3.1.2.html),
[Datadog APIs](https://docs.datadoghq.com/api/latest/),
[Prometheus HTTP API](https://prometheus.io/docs/prometheus/latest/querying/api/),
[Loki HTTP API](https://grafana.com/docs/loki/latest/reference/loki-http-api/)
and [GitHub Checks API](https://docs.github.com/en/rest/checks/runs).
