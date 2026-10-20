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

Normal Rust tests replay 66 frozen source cases without Python: 39 exact packet
and context projections and 27 rejection cases. Native tests additionally cover
extension allowlists, finite and depth admission, forged nested fields, false and
zero scalar values, original observation ages and the distinctions among
negative, stale and missing evidence. The development-only
`tools/capture-packet-reference.py` verifies the pinned source revision and
unchanged production tree before recapturing these fixtures.
