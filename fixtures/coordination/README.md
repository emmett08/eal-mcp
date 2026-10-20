The synthetic coordination oracle is captured from production modules at
`e2d6f346a37f92483b9f9a13923b98feb053447b`. The capture records source hashes and
leaves that checkout unchanged. It launches no workers, MCP sessions, providers
or studies and supplies no actual measurement claims.

Regenerate or verify during development with the pinned MCP oracle environment:

```sh
/tmp/eal-app-oracle/bin/python tools/capture-coordination-reference.py ../earl-audit
/tmp/eal-app-oracle/bin/python tools/capture-coordination-reference.py ../earl-audit --check
```

The mailbox module imports the source MCP adapter, so this environment needs the
same pinned MCP dependencies as the application oracle even though the capture
never invokes that adapter. Normal Rust builds and tests use only the JSON file.

The corpus retains 13 exact canonical encodings, 26 durable scenarios with 175
steps, and 52 task, anchor, finding and answer contracts. The State clock is fixed
to the timestamp in the fixture. Anchors use explicitly synthetic fixed source
and registry fingerprints; their complete content and dependent digests are
checked. No output fields, payload keys, identities, statuses, counts, times,
diagnostics or callback traces are removed or normalized. Python exception
classes are retained as provenance; Rust's string error API compares their exact
messages. Persistence scenarios run on Unix, where private native stores are
supported; pure canonical and contract comparisons remain portable. The sole
diagnostic exception is the invalid-month anchor: Python says `month must be in
1..12`, while the native ISO parser says `Invalid ISO-8601 date`. The test asserts
both exact messages and rejection; it does not substitute a Python diagnostic.

Reservations authorize an external effect only when new. Reopening an unresolved
reservation reports an unknown outcome. Committed output and ordered outboxes
are immutable. Delivery retries cover failed send/ACK confirmation and partial
outboxes, while callbacks can open a separate database writer. Mailbox cases
retain ordered redelivery, received-before-ACK, stable-ID deduplication before
quota enforcement, exact UTF-8 envelope byte accounting and immutable completion
evidence. Answer cases retain claim status/time and distinguish scope mismatch,
unknown results and negative assessments.
