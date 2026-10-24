# Optional acquisition adapters

This package owns bounded HTTP transport, known-tool command construction, pinned JSON artifact import, retained review correspondence and specialised trace inspection.
It depends on the pure collector contract and the narrow process supervisor package.
The EAL language core has no dependency on this package.

Command execution always uses the independent process supervisor.
Set `EAL_COMMAND_SUPERVISOR` to an absolute companion path or install the companion beside the application or on an absolute PATH directory.
The available companion names are `eal-process-supervisor`, the compatibility alias `eal-command-supervisor` and the packaged EngArg facade `engarg-process-supervisor`.
During `cargo test`, integration harnesses in a Cargo `deps` directory also locate the packaged companion in its parent profile directory.
Missing or invalid companions fail explicitly.
Cancellation drops the command ownership token and asks the sole supervisor implementation to stop and reap descendants.

The known-tool compatibility profile forwards a bounded named set of existing operator configuration variables.
Arbitrary instrument or laboratory tools use the general operator-controlled registry rather than this profile.
The `scoped` module is an explicitly selected compatibility profile for Kubernetes and Grafana reads.
It does not infer an infrastructure requirement for other arguments.

Retained artifact import resolves an operator-selected base directory once when
any artifact path is relative. Relative bases and directory aliases identify the
same canonical directory. Relative artifacts remain beneath that directory and
cannot use parent traversal or leaf symbolic links. Absolute pinned paths are
independent operator grants and do not require an otherwise unused base to exist.

Trace inspection identifies candidate spans with matching attributes on the same span.
`inspect_trace_with_keys` accepts operator-selected attribute keys.
The DPE rollout selector is a specialised compatibility preset.
Neither selector establishes provenance, response completeness, deployment current state or fleet coverage.

EngArg owns its case digests, evidence-need mappings and review obligations.
Its old known-tool and scoped commands delegate here during migration and are removed at EngArg 0.11.
The runtime `eal-command-supervisor` executable alias is removed in EAL 0.2 after installations migrate to `eal-process-supervisor`.

`profiles` owns compatibility adapter identifiers, required input names, observation predicates,
credential forwarding names, fixed child environment, versions and execution allowances.
Both adapter admission and EngArg's temporary facade bindings consume those definitions.
EngArg separately chooses its neutral default authoring capabilities and its unresolved custom stub policy.
