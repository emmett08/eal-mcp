# Observatory execution and effort pilot

[Sample task #36](https://github.com/emmett08/eal-mcp/issues/36) connects the context
experiment check to its implementation PR and a private effort study. The pilot
uses these same identifiers in the action receipts and effort configuration:

| Field | Value |
| --- | --- |
| Protocol | `observatory-linked-pilot-v1` |
| Task | `eal-context-effort-pilot` |
| Task issue | `emmett08/eal-mcp#36` |
| Initial implementation branch | `feat/observatory-effort-pilot` |

The dedicated [pilot workflow](../.github/workflows/observatory-pilot.yml) runs on
relevant pull requests and supports manual dispatch after adoption. This is a
bounded exception to the manual workflow policy: the complete
`Rust language and runtime` workflow remains manual. The pilot installs Rust
1.90.0 and wraps the existing context-experiment command:

```sh
cargo +1.90.0 test --manifest-path experiments/eal-context-001/Cargo.toml --locked
```

Observatory is pinned to commit `38f022d2d2f6e015fa42a23f1bc335783e263708`.
The setup step captures the allowlisted GitHub execution identity; the wrapper
emits immutable start and completion records; the publication step runs even when
the selected check fails. Each job produces its own bundle. Cancellation or runner
loss can prevent publication, so the collector must also reconcile completed
runs, attempts and jobs. Missing evidence is not successful execution.

`github.actor`, the rerun actor and trigger event are platform evidence. The
declared controller remains `unknown` and the session remains unavailable unless
an explicit source supplies them. A human GitHub account does not prove a human
performed the work. This wrapper observes the selected command only.

## Link the private effort configuration

Use this object in the private effort configuration's `links` array. The private
collector must set `protocol_id` to the value above. Add the actual linked PR
number (or run `effort.py sync-links` once the PR exists); do not invent a
participant, activity source, observation window or response.

```json
{
  "task_id": "eal-context-effort-pilot",
  "repository": "emmett08/eal-mcp",
  "issue_number": 36,
  "branches": ["feat/observatory-effort-pilot"],
  "pull_requests": [],
  "projects": []
}
```

The Observatory [effort guide](https://github.com/emmett08/engineering-work-observatory/blob/38f022d2d2f6e015fa42a23f1bc335783e263708/docs/effort-estimation.md)
describes project-aware ActivityWatch collection, frozen predictions, randomly
sampled review forms and effort estimation. Activity and sampled responses belong
on a trusted private runner. Public reports should contain approved aggregates
and links, not raw activity or participant response files.

Until real observations support an estimate, human effort remains unavailable.
CI duration measures the command's runtime and cannot be converted to human
labour. An optional [effort observation issue form](../.github/ISSUE_TEMPLATE/effort-observation.yml)
allows occasional aggregate self-reports and explicit unknown values. Those
reports remain declared supporting evidence; they are not automatically imported
as calibration responses and do not replace the private, hash-bound sampled
review form. No full timesheet is required by this pilot.
