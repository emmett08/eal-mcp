# Assessment packets and model context

`eal-runtime::packets::AssessmentPacketBuilder` projects a completed, valid
assessment into `EAL/assessment-packet/2`. The packet identifies its stored
assessment and retains a link to the full explanation. Selected claims retain
their final status, authored statement, explicit prose verification flag, typed
result comparison, support and challenge references. Premise claims, alternative
arguments, assumptions, objections, reasoning backing and evidence are followed
within the projection's traversal bounds. Formal support does not establish that
an authored statement correctly describes the formal proposition.

Method results retain status, bounded identities, provenance digests, checked
scalar outputs, binding qualifications and predicate outcomes. Output paths come
from the configured method registry, with built-in declarations as a fallback.
An extension without a registered contract contributes no output values. Dynamic
map paths ending in `.*` are excluded because their keys may disclose input
labels. A fixed set of built-in scalar qualifications is also retained. Raw
observations, process streams, full source, formal queries, arbitrary extension
fields and diagnostic prose are excluded.

`PacketLimits` defaults to 16 KiB of compact UTF-8 JSON, 16 selected claims, 24
arguments, 32 objections, 32 assumptions, 48 evidence entries, 16 method outputs,
six availability or predicate diagnostics, 16 observation links per claim and
1,024 bytes per statement. Count limits are configurable from 1 through 128;
statement limits range from 1 through 4,096 bytes and the total byte limit must
be at least 4,096. Premise traversal visits at most 128 arguments; each claim's
observation traversal visits at most 256 graph nodes. Traversal preserves the
source implementation's ordering and omission accounting.

Truncated statements end on a UTF-8 boundary, report every omitted byte and set
`prose_verified` to false. Missing statements are explicit omissions. Counts,
routes and diagnostics omitted by the projection appear in `omitted`, and
`summary_complete` becomes false. If the detailed packet exceeds its byte
budget, a smaller navigation packet retains claim statuses and the explanation
link, removes statements and records `details: "packet_byte_limit"`. If that
packet also exceeds the budget, construction fails.

## Registered retrieval and presentation budgets

Registered MCP clients can retain `entry_id`, `assessment_id` and `claim` from
`eal_assess_known` and later call `eal_get_known_packet` with those identifiers.
The read returns an `EAL/registered-packet/1` envelope containing the bounded
packet, its source, context and method identities, status and original
`assessed_at`. This supports restoring assessment context after a host discards
an earlier tool response. Conversation history and the decision to discard or
reload context remain host responsibilities.

`eal_assess_known`, `eal_get_known_packet` and the operator's `eal_packet` accept
an optional `packet_limits` object:

```json
{
  "max_bytes": 8192,
  "max_statement_bytes": 512
}
```

`max_bytes` accepts integers from 4,096 through 65,536 and
`max_statement_bytes` accepts integers from 1 through 4,096. Omitted fields use
the existing defaults of 16,384 and 1,024. Empty options preserve the default
projection. Unknown fields and non-integer values are rejected before
acquisition. The byte budget covers the compact JSON packet, with MCP framing
and the surrounding registered envelope adding overhead. It measures UTF-8
bytes rather than model tokens. Dependency counts and traversal limits remain
as documented above.

For example, a client can request an 8 KiB packet during assessment, then read
the same stored result with a larger budget if `omitted.details` reports
`packet_byte_limit`:

```json
{
  "entry_id": "demo",
  "assessment_id": "ASSESSMENT_ID_FROM_RESPONSE",
  "claim": "works",
  "packet_limits": {"max_bytes": 65536, "max_statement_bytes": 4096}
}
```

Retrieval requires an exposed entry, a claim selected in the exact registered
source and method revision, and a stored context fingerprint matching that
entry's registered context. Historical source revisions remain retrievable
without refreshing current files. The projecting runtime must still have the
assessment's method registry, whose output declarations control disclosure.
Retrieval performs no collection or reassessment, so observation ages and
freshness classifications retain their meaning at the original assessment time.
Use `eal_assess_known` when a current assessment is required.

The existing `full_explanation` reference remains an operator capability.
Registered clients use `eal_get_known_packet` to expand projected details within
the documented limits. Larger byte and statement budgets can recover details
omitted by those budgets. Count and traversal ceilings continue to apply, and
`summary_complete` describes the existing projection contract.
Raw observations, arbitrary method fields and unrelated claims remain outside
the registered response. An incomplete packet requires the consumer to qualify
its answer or obtain further authorised evidence.

The embedded API provides `ReasoningService::packet_with_limits`,
`RegisteredAssessmentHost::retrieve_packet` and
`RegisteredAssessmentOptions::packet_limits`. The caller of the registered host
enforces its own entry allowlist. Existing default projection fixtures remain
unchanged. The new tool and optional budget fields extend the native MCP
catalogue beyond the frozen source catalogue.

## Model input

An optional collection must match the assessment's collection ID, source and
canonical context fingerprint. Each projected observation must have the assessed
evidence's run ID. Age uses the original `collected_at`, including for reused
observations. A successful record reports its observation time, age and freshness
classification; a failed acquisition reports its attempt time and
`no_observation`. Invalid timestamps remain explicit. Predicate failure,
unavailability, staleness and failed acquisition retain separate meanings.

`ModelContextBuilder` accepts a registered assessment containing a checked
packet. It preserves statements, scalar method results, qualifications and
omissions, replaces claim observation references with evidence IDs, and removes
opaque observation IDs. Evidence requirements are `met` for available evidence,
`not_met` when the sole issue is `predicate_not_met`, and `unresolved` otherwise.
An unsupported claim does not establish its opposite; a negative measurement is
different from a missing measurement. `CONTEXT_INSTRUCTION` states these rules
for the model adapter.

Native admission additionally checks finite JSON, a fixed depth ceiling of 128
and a 64 MiB input envelope before recursive copying. Malformed nested groups
return errors. Model-context admission checks projected field shapes and bounded
metadata, rejects nested raw payloads and inconsistent correspondence or prose
qualifications, and checks omissions against the completeness flag. These checks
do not authenticate an arbitrary caller-supplied `Value`, establish provenance
of prose, or independently re-run method computations. The service obtains the
assessment and collection from its durable store; that host boundary remains
authoritative. The context builder relies on the packet builder and configured
registry for method-output admission.

`ModelContextAdapter::prepare` returns an inspection envelope with `assessment`,
`context` and `messages`. Pass its `messages` array to the text model. Sending the
whole envelope repeats the context in multiple representations. Consumers can
retain assessment references separately for later retrieval.

Native tests cover packet projection and context construction, extension
allowlists, finite and depth admission, forged nested fields, false and zero
scalar values, original observation ages and the distinctions among negative,
stale and missing evidence.
