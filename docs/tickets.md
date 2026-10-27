# Retained authoring and evidence-execution tickets

`eal-coordinate ticket-run --spec FILE --run DIRECTORY` runs four ordered application stages: `author`, `integrate`, `export` and `execute`. The operator selects one worker executable and its argument vector. EngArg supplies a worker which authors the argument, validates direct tool bindings, builds a standalone EAL package and invokes `eal_execute` through MCP.

The coordinator freezes an `eal.ticket/2` specification in a new private journal. It pins the worker executable and all declared input files. Before every new stage it checks the input identities and every accepted output artifact again. Inputs and outputs must be canonical regular files within the ticket workspace. Each artifact records its kind, path, encoded byte count and SHA-256 digest. Artifact files are bounded to 16 MiB individually and 64 MiB in aggregate, with at most 128 artifacts. The worker executable has a separate 256 MiB bound and is hashed incrementally. The journal directory is distinct from the worker's workspace.

The specification declares `ticket_id`, `workspace`, `worker_argv`, `environment`, `timeout_seconds`, `stage_timeout_seconds`, `context` and `inputs`. Application context and stage-outcome summaries each have a 65,536-byte encoded bound. Larger inputs and results belong in retained artifacts. The `environment` list names variables to forward. Their values are excluded from the frozen specification. The total deadline is at most 86,400 seconds. Each stage has at most 3,600 seconds and remains within the frozen total deadline.

The worker receives an `eal.ticket-stage/2` JSON object containing the ticket identity, stage, workspace, application context, prior artifact receipts and call identifier. It returns a live receipt envelope:

```json
{
  "response": {
    "schema_version": "eal.ticket-stage-result/2",
    "ticket_id": "example-ticket",
    "stage": "author",
    "status": "completed",
    "artifacts": [],
    "outcome": null
  },
  "execution": {
    "execution_id": "independent-host-execution-identity",
    "model": null,
    "usage": null
  }
}
```

The illustration omits artifact entries. Admission requires these kinds for each completed stage:

| Stage | Required artifacts |
| --- | --- |
| `author` | `analysis` |
| `integrate` | `analysis_ready`, `integration` |
| `export` | `source`, `tools`, `build` |
| `execute` | `execution`, `collection`, `assessment` |

The application validates each artifact's domain contents. The coordinator validates stage identity, ordering, file ownership, bounds and exact retained bytes. A completed execute stage also supplies a valid complete assessment outcome and explicit claim status. A claim assessed as unsupported can therefore belong to a completed ticket. Missing configuration or unavailable evidence can produce a `blocked` stage with a retained explanation and partial artifacts.

Completed execution checks correspondence between the exported source and TOML, package identity, execution envelope, collection and assessment. It recomputes the executable program digest and checks the packaged context fingerprint. Separately retained collection and assessment artifacts must equal the corresponding values in the execution envelope. The summary identifies one collection and its assessment.

Execution admission derives the selected claim's evidence closure from the retained EAL source. Each selected request requires a successful collection record and a usable evidence verdict. Independent evidence outside that closure can remain pending. A measured result which refutes its predicate still completes the investigation. Acquisition failure, expiry and missing observations remain unresolved.

The controller reserves a stage effect before launching the supervised worker. It retains the first exact response bytes before admitting and publishing the result. Repeating `ticket-run` with the same specification revalidates completed artifacts and continues only into stages which have never been reserved. An interrupted effect remains `uncertain` and is never automatically repeated. A rejected response, failed command or blocked stage stops the ticket. A separately identified new ticket is required to retry uncertain work.

```sh
eal-coordinate ticket-status --run /absolute/path/to/journal
eal-coordinate ticket-export --run /absolute/path/to/journal --output /absolute/path/to/status.json
eal-coordinate ticket-cancel --run /absolute/path/to/journal
```

Cancellation prevents new stages and interrupts an active supervised worker. Status and export recheck retained response bytes and artifact identities. The final ticket status and the final claim status are separate fields.
