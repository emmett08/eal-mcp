# Share an assessment without losing its qualifications

[Documentation](index.md) · [Applications](applications.md) · [Coordination](coordination.md)

An assessment may contain more source, observations and diagnostics than a reader
or model needs in one interaction. A packet gives that consumer a bounded view,
while retaining the status and qualifications required to interpret it. The
stored assessment remains the source of the result. Enlarging or retrieving a
packet changes its presentation, not the underlying evidence or assessment time.

`AssessmentPacketBuilder` projects selected claims from a valid assessment. It
retains their statements, status, typed result comparison, support and challenge
references, and an explicit prose-verification flag. Traversal follows premise
claims, alternative arguments, assumptions, objections, backing and evidence
within declared bounds. A formal result does not by itself verify that the
statement accurately describes the proposition.

## Read completeness before interpreting a result

Completed assessments use `EAL/assessment-packet/2`. Computationally incomplete
assessments use `EAL/assessment-packet/3`, with global `complete=false` and
`assessment_complete=false` on every selected claim. Both qualifications survive
the compact navigation fallback. Consumers check packet version, method state
and qualifier consistency. Contradictory qualifications or unknown versions fail
admission.

`summary_complete` answers a different question: did the packet omit any
presentation detail? It cannot establish that a method finished. An assessment
can complete while its packet omits details, and an incomplete assessment can
have a complete projection of its retained information.

Method entries retain status, bounded identities, provenance digests, admitted
scalar outputs, binding qualifications and predicate outcomes. Output paths come
from configured method contracts, with built-in declarations as a fallback.
An extension without a contract contributes no output values. Dynamic map paths
ending in `.*` are excluded because their keys could disclose input labels.
A fixed set of built-in scalar qualifications is also retained.

Raw observations, process streams, complete source, formal queries, arbitrary
extension fields and diagnostic prose stay outside the packet. Each packet
identifies its stored assessment and links to the full explanation, providing an
explicit way for an authorised operator to inspect the omitted material.

## Choose a presentation budget

The default `PacketLimits` describe a compact result suitable for one retrieval:

| Limit | Default |
| --- | ---: |
| Compact UTF-8 JSON | 16 KiB |
| Selected claims | 16 |
| Arguments | 24 |
| Objections | 32 |
| Assumptions | 32 |
| Evidence entries | 48 |
| Method outputs | 16 |
| Availability or predicate diagnostics | 6 |
| Observation links per claim | 16 |
| Statement bytes | 1,024 |

Count limits range from 1 through 128 and statement limits from 1 through 4,096
bytes. The total byte allowance must be at least 4,096. Premise traversal visits
at most 128 arguments, and a claim's observation traversal visits at most 256
graph nodes. Projection preserves source ordering and accounts for omissions.

A truncated statement ends on a UTF-8 boundary, reports its omitted byte count
and sets `prose_verified=false`. A missing statement is an explicit omission.
Dropped counts, routes and diagnostics appear in `omitted` and make
`summary_complete=false`.

If detailed projection exceeds the byte allowance, a navigation packet retains
claim statuses and the explanation link, drops statements and records
`details: "packet_byte_limit"`. If even that packet exceeds the allowance,
construction fails. Consumers can therefore distinguish a smaller successful
view from an absent result.

## Retrieve a registered result again

A registered MCP client retains `entry_id`, `assessment_id` and `claim` from
`eal_assess_known`. It can later pass those identifiers to `eal_get_known_packet`.
The returned `EAL/registered-packet/1` envelope includes the packet, source,
context and method identities, status and original `assessed_at`.

`eal_assess_known`, `eal_get_known_packet` and the operator's `eal_packet` accept
optional presentation limits:

```json
{"max_bytes": 8192, "max_statement_bytes": 512}
```

`max_bytes` ranges from 4,096 through 65,536 and `max_statement_bytes` from 1
through 4,096. Omission uses defaults of 16,384 and 1,024. Empty options preserve
the default projection. Unknown fields and non-integer values are rejected before
acquisition. The budget counts compact JSON UTF-8 bytes. MCP framing and the
registered envelope add overhead, and bytes are not model tokens.

If an 8 KiB response reports `packet_byte_limit`, retrieve the same stored result
with a larger allowance:

```json
{
  "entry_id": "demo",
  "assessment_id": "ASSESSMENT_ID_FROM_RESPONSE",
  "claim": "works",
  "packet_limits": {"max_bytes": 65536, "max_statement_bytes": 4096}
}
```

Retrieval requires an exposed entry, a claim selected in the exact registered
source and method revision, and a stored context fingerprint matching the entry.
Historical source revisions remain retrievable without refreshing current files.
The projecting runtime must still have the assessment's method registry because
its output contracts determine what may be disclosed.

Retrieval performs no collection or reassessment. Observation ages and freshness
remain classifications at the original assessment time. Call `eal_assess_known`
for a current result. A larger byte or statement allowance can recover details
omitted for those reasons, while count and traversal ceilings still apply. Raw
observations, arbitrary method fields and unrelated claims stay outside the
registered response. `full_explanation` remains an operator capability.

Embedded consumers use `ReasoningService::packet_with_limits`,
`RegisteredAssessmentHost::retrieve_packet` and
`RegisteredAssessmentOptions::packet_limits`. The caller of the registered host
must enforce its entry allowlist. The host also decides when to discard or
restore conversation context.

## Prepare model input from the retained result

`ModelContextBuilder` accepts a registered assessment with a checked packet.
It preserves statements, scalar method outputs, qualifications and omissions,
replaces observation references with evidence IDs and removes opaque observation
IDs. `ModelContextAdapter::prepare` returns an inspection envelope with
`assessment`, `context` and `messages`. Send the `messages` array to the model.
Retain the assessment references separately for later retrieval. Sending the
whole inspection envelope would repeat the context in several representations.

An optional collection must match the assessment's collection ID, source and
canonical context fingerprint. Every projected observation must have the assessed
evidence's run ID. Age uses original `collected_at`, including reused observations.
Successful records retain observation time, age and freshness. Failed acquisition
retains its attempt time and `no_observation`. Invalid timestamps remain explicit.

The builder assigns evidence requirements according to their retained meaning:

| Outcome | Condition |
| --- | --- |
| `met` | Evidence is available. |
| `not_met` | Its sole issue is `predicate_not_met`, and no availability issues were omitted. |
| `unresolved` | Availability or a decisive predicate result remains unavailable. |

Evaluation, projection and context admission share a closed issue vocabulary.
`dependency_unavailable` means a missing, invalid or stale upstream dependency.
`predicate_unresolved` means the predicate could not be decided. Neither is a
measured negative. Scoped-review and pure-derivation integrity failures retain
`invalid_observation`, and original-age failures retain `stale_observation`.
Predicate diagnostics can include `invalid_expression`. An empty diagnostic path
identifies the whole authored expression without disclosing its source text.

The availability-issue omission count applies to the packet as a whole. If it is
positive, every unavailable entry stays unresolved because the consumer cannot
determine which qualification was omitted. This can withhold a definite negative
for another unavailable entry. Available evidence remains met, and complete pure
predicate failures remain not met. An unsupported claim establishes neither its
opposite nor a missing observation. `CONTEXT_INSTRUCTION` communicates these
interpretation rules to the model adapter.

## Keep the store boundary authoritative

Admission checks finite JSON, depth 128 and a 64 MiB input envelope before
recursive copying. It checks projected field shapes, bounded metadata,
correspondence, prose qualifications and agreement between omissions and
completeness. Unknown availability issues, nested raw payloads and malformed
groups fail admission.

These shape checks cannot authenticate an arbitrary caller-supplied `Value`,
verify authored prose or rerun a method. The service obtains assessment and
collection from its durable store. The context builder relies on the packet
builder and configured registry to admit method outputs. That ownership chain
is what makes a projected value interpretable as a retained assessment result.

Native checks cover projection, extension output allowlists, finite/depth limits,
forged fields, false and zero scalar outputs, original ages, and the distinctions
between negative, stale and missing evidence. The
[interface contracts](interface-contracts.md) explain where those checks belong,
and [development](development.md) gives the required verification commands.
