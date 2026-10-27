# Catalog Sync Operator evidence

This example checks the named Application resource in an approved cluster
inventory and asks Tempo for a trace matching the configured rollout query in
each mapped telemetry cluster. The selected time window bounds the trace claim.
The separate GitHub claim identifies an exact repository commit and its tree.

Install the native commands from the repository root into an operator-owned
prefix. The example registry uses `/opt/eal/bin`.

```sh
cargo install --root /opt/eal --path crates/eal-runtime --locked
cargo install --root /opt/eal --path crates/eal-cli --locked
cargo install --root /opt/eal --path crates/eal-collectors --locked
cargo install --root /opt/eal --path crates/eal-mcp --locked
```

Prepare the operator configuration before running the example.

1. Copy `production-clusters.example.json` to your chosen inventory path and
   replace its revision and every cluster entry with the complete approved
   production inventory. Each entry maps a cluster ID to its kubeconfig context
   and the exact telemetry cluster attribute value.
2. Update the absolute paths in `tools.toml` for the collector, supervisor,
   `kubectl`, `gh` and inventory. Set the actual Kubernetes API group, kind,
   namespace and resource name in both the registry and source input.
3. Replace the Tempo endpoint and confirm the three TraceQL attribute names in
   the registry match your instrumentation. Provide `TEMPO_TOKEN` and
   `TEMPO_TENANT` through the environment. For a single-tenant endpoint, remove
   `--tenant-env TEMPO_TENANT` and the corresponding inheritance entry.
4. Replace the repository and exact 40-character GitHub commit in both files
   when using `revision_identified`. Supply `GH_TOKEN` through the environment.
   Kubernetes authentication uses the explicitly inherited `KUBECONFIG`, `HOME`
   and `PATH` variables. Adjust that allowlist for your authentication plugins.
5. Set `start_unix` and `end_unix` in `case.eal` to the intended Unix-second
   interval. Its duration is at most 24 hours with the example's allowance.
6. Calculate the approved inventory hash and replace both
   `REPLACE_WITH_APPROVED_INVENTORY_SHA256` predicates in `case.eal` with that
   same value. Both observations must refer to those inventory bytes.

For the example paths, calculate dependency hashes with these commands.

```sh
sha256sum /etc/eal/production-clusters.json
sha256sum /opt/eal/bin/eal-collector
sha256sum /opt/eal/bin/eal-process-supervisor
sha256sum /usr/local/bin/kubectl
sha256sum /usr/local/bin/gh
```

The commented `pinned_files` blocks in `tools.toml` show the corresponding
dependency pins. Substitute each digest and uncomment the block to enforce it.
Add executable authentication plugins when their code affects acquisition.
Update approved hashes deliberately when changing the inventory or binaries.

From the repository root, plan and collect the Kubernetes/Tempo closure.

```sh
/opt/eal/bin/eal --workspace ./examples/catalog-sync-operator \
  --registry ./examples/catalog-sync-operator/tools.toml \
  plan case.eal --claim rollout_observed

/opt/eal/bin/eal --workspace ./examples/catalog-sync-operator \
  --registry ./examples/catalog-sync-operator/tools.toml \
  collect case.eal --claim rollout_observed --context @context.json
```

Use the returned `collection_id` as `COLLECTION_ID` in the assessment command.

```sh
/opt/eal/bin/eal --workspace ./examples/catalog-sync-operator \
  --registry ./examples/catalog-sync-operator/tools.toml \
  reason case.eal --collection COLLECTION_ID --context @context.json
```

For registered assessment, register the selected claim and its context once.
Compatible assessment reuses fresh observations with matching identities and
collects missing evidence.

```sh
/opt/eal/bin/eal --workspace ./examples/catalog-sync-operator \
  --registry ./examples/catalog-sync-operator/tools.toml \
  register case.eal --entry-id catalog-sync-operator \
  --claim rollout_observed --context @context.json

/opt/eal/bin/eal --workspace ./examples/catalog-sync-operator \
  --registry ./examples/catalog-sync-operator/tools.toml \
  assess-known catalog-sync-operator --claim rollout_observed

/opt/eal/bin/eal-mcp --workspace ./examples/catalog-sync-operator \
  --registry ./examples/catalog-sync-operator/tools.toml \
  --transport stdio --exposure registered --known-entry catalog-sync-operator
```

The registered MCP operation is `eal_assess_known` with
`{"entry_id":"catalog-sync-operator","claim":"rollout_observed"}`.

Resource presence and returned trace summaries establish the bounded properties
stated in `rollout_observed`. Production inventory authority and the mapping
between Kubernetes contexts and telemetry identities remain operator assumptions.
Controller health and correspondence to a resource namespace, UID or deployed
revision require further predicates and observations. Tempo's bounded first-N
search leaves overall trace absence unresolved when it returns an empty result.
See [collector contracts](../../docs/collectors.md) for the complete boundaries.
