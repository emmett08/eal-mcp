# Carry an authored claim through evidence execution

[Documentation](index.md) · [Coordination](coordination.md) · [Packets](packets.md)

A complete investigation crosses several application boundaries. It authors an
argument, resolves its tool bindings, exports an executable programme and runs
that programme against evidence. An execution ticket retains what each stage
consumed and produced, so a reviewer can connect the final assessment to the
original argument and the actual collection.

`eal-coordinate ticket-run` runs four ordered stages: `author`, `integrate`,
`export` and `execute`. The operator chooses one worker executable and its argv.
EngArg provides a worker that authors the argument, validates direct bindings,
builds a standalone EAL package and calls MCP `eal_execute`. The engineer-facing
[EngArg guide](https://github.com/emmett08/engarg/blob/main/docs/USAGE.md) explains
how to prepare that application's inputs. This chapter specifies the coordinator
boundary shared by any conforming worker.

## Freeze the inputs before starting

```sh
eal-coordinate ticket-run --spec /absolute/path/to/ticket.json \
  --run /absolute/path/to/new-journal
```

The command admits an `eal.ticket/2` specification into a new private journal.
Its fields are `ticket_id`, `workspace`, `worker_argv`, `environment`,
`timeout_seconds`, `stage_timeout_seconds`, `context` and `inputs`. The journal
must be separate from the worker's workspace. `environment` names variables to
forward, while the frozen specification excludes their values.

The coordinator pins the worker executable and every declared input. Before a
new stage, it checks input identities and all previously accepted artifacts
again. Inputs and outputs must be canonical regular files within the ticket
workspace. Each receipt records the artifact's kind, path, encoded byte count
and SHA-256 digest. These checks give the next stage an identified input rather
than a mutable filename alone.

| Bound | Allowance |
| --- | ---: |
| Individual artifact | 16 MiB |
| All artifacts | 64 MiB, at most 128 artifacts |
| Worker executable | 256 MiB, hashed incrementally |
| Application context or stage-outcome summary | 65,536 encoded bytes each |
| Total deadline | At most 86,400 seconds |
| Stage deadline | At most 3,600 seconds and within the total deadline |

Put larger application inputs and results in retained artifacts. Context and
outcome fields hold bounded summaries and references.

## Implement a stage worker

The worker receives one `eal.ticket-stage/2` object containing the ticket identity,
stage, workspace, application context, prior artifact receipts and call ID. It
returns a live receipt envelope:

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

The empty artifacts array illustrates the shape only. A completed stage must
supply these artifact kinds:

| Stage | Required artifacts | Purpose |
| --- | --- | --- |
| `author` | `analysis` | Retain the authored argument. |
| `integrate` | `analysis_ready`, `integration` | Retain the argument admitted for export and its integration result. |
| `export` | `source`, `tools`, `build` | Identify the executable EAL, operator bindings and package receipt. |
| `execute` | `execution`, `collection`, `assessment` | Connect actual execution, acquired observations and the resulting assessment. |

The application validates domain contents. The coordinator checks stage identity,
ordering, ownership, bounds and retained bytes. Missing configuration or
unavailable evidence can yield a `blocked` stage with an explanation and partial
artifacts.

## Separate investigation completion from claim support

A completed execute stage requires a valid, complete assessment and an explicit
claim status. It verifies correspondence between EAL source, TOML, package
identity, execution envelope, collection and assessment. Admission recomputes
the executable programme digest and checks the packaged context fingerprint.
The separately retained collection and assessment must equal their values in the
execution envelope, and the summary identifies one collection and its assessment.

Admission derives the selected claim's evidence closure from the retained source.
Every selected request requires a successful collection record and usable evidence
verdict. Unrelated evidence can remain pending. A completed measurement that
refutes its predicate still completes the investigation. Missing, expired or
failed acquisition remains unresolved. Consequently, an unsupported claim can
belong to a completed ticket. Completion reports that the required work produced
an assessable result, not that the intended claim won.

## Inspect or continue a retained ticket

The controller reserves a stage effect before launching the supervised worker.
It retains the first exact response before admission and publication. Repeating
`ticket-run` with the same specification rechecks completed artifacts and continues
only into stages that have never been reserved.

An interrupted effect stays `uncertain` and is never automatically repeated.
A rejected response, failed command or blocked stage stops the ticket. Retrying
uncertain work requires a separately identified new ticket, chosen after the
operator has inspected the retained result.

```sh
eal-coordinate ticket-status --run /absolute/path/to/journal
eal-coordinate ticket-export --run /absolute/path/to/journal \
  --output /absolute/path/to/status.json
eal-coordinate ticket-cancel --run /absolute/path/to/journal
```

Cancellation prevents new stages and interrupts an active supervised worker.
Status and export recheck response bytes and artifact identities. The final
ticket status remains separate from the final claim status. Use the
[assessment packet](packets.md) to communicate what that claim status means and
which qualifications a reader must retain.
