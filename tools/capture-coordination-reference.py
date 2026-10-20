#!/usr/bin/env python3
"""Retain exact, synthetic coordination contracts from the pinned Python source.

This development-only oracle executes production ledger, State, JsonBus and
task/answer contracts. It launches no workers, MCP servers, providers or studies.
The State clock alone is fixed. There are no identity or payload exclusions.
"""
from __future__ import annotations

import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from unittest.mock import patch

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]
NOW = "2040-01-01T00:00:00.000000Z"
SOURCE_FILES = (
    "src/eal/coordination/ledger.py", "src/eal/coordination/state.py",
    "src/eal/coordination/bus.py", "src/eal/coordination/contracts.py",
    "src/eal/coordination/profiles.py", "src/eal/coordination/response-schema.json",
)


def message(identity="request", sender="reviewer", recipient="assessor", **extra):
    return {"schema": "EAL/MAS-coordination/3", "kind": "request", "message_id": identity,
            "task_id": "synthetic-task", "task_digest": "a" * 64,
            "sender": sender, "recipient": recipient, "reply_to": None, "attempt": 0, **extra}


def scenarios(canonical):
    result = []

    def add(group, name, steps):
        # Each occurrence is independent: replayed operations have different
        # observations even when their request values are identical.
        result.append({"group": group, "name": name, "steps": [deepcopy(step) for step in steps]})

    payload = {"scope": "synthetic", "nested": [1, 1.5, None, True], "literal": {"span": 7}}
    output = {"raw_response": '{"synthetic":true}', "accepted": True}
    replies = [message("reply-2", "assessor", "reviewer"), message("reply-1", "assessor", "reviewer")]
    reserve = {"op": "reserve_effect", "key": "run/assessor/request", "payload": payload}
    complete = {"op": "complete_effect", "key": reserve["key"], "result": output, "outbox": replies}
    add("ledger", "reservation-commit-reopen-immutable", [reserve, {"op": "reopen"},
        {**reserve, "payload": {"literal": {"span": 7}, "nested": [1, 1.5, None, True], "scope": "synthetic"}},
        complete, {"op": "reopen"}, reserve, complete,
        {**complete, "result": {**output, "accepted": False}},
        {**complete, "outbox": list(reversed(replies))}, reserve,
        {"op": "drain"}, reserve, {"op": "drain"}])
    add("ledger", "unknown-never-sends-or-acknowledges", [reserve, {"op": "reopen"}, reserve,
        {"op": "drain"}, reserve])
    for label, first, changed in (("bool-integer", True, 1), ("integer-float", 1, 1.0),
                                  ("float-signed-zero", -0.0, 0.0), ("array-order", [1, 2], [2, 1])):
        add("ledger", "identity-" + label, [
            {"op": "reserve_effect", "key": "effect", "payload": first},
            {"op": "reserve_effect", "key": "effect", "payload": changed},
            {"op": "reserve_effect", "key": "effect", "payload": first}])
    add("ledger", "collision-after-commit", [reserve, complete,
        {**reserve, "payload": {**payload, "scope": "changed"}}, reserve])
    add("ledger", "stable-key-keeps-whitespace", [
        {"op": "reserve_effect", "key": " effect ", "payload": None},
        {"op": "reserve_effect", "key": "effect", "payload": None},
        {"op": "reserve_effect", "key": "\u001c\u001d\u001e\u001f", "payload": None},
        {"op": "reserve_effect", "key": "", "payload": None}])
    add("ledger", "invalid-completion-no-partial-output", [
        complete, reserve,
        {**complete, "outbox": [replies[0], replies[0]]},
        {**complete, "outbox": [{"message_id": ""}]},
        {**complete, "outbox": [None]}, reserve, complete])
    for name, failure in (("send-first", {"fail_send": "reply-2"}),
                          ("send-second", {"fail_send": "reply-1"}),
                          ("ack", {"fail_ack": reserve["key"]})):
        add("ledger", "drain-" + name + "-recovery", [reserve, complete,
            {"op": "drain", **failure}, {"op": "reopen"}, reserve, {"op": "drain"}, reserve])
    add("ledger", "empty-rejected-outbox-acknowledges", [reserve,
        {**complete, "result": {"accepted": False, "raw_response": "invalid but retained"}, "outbox": []},
        {"op": "drain"}, reserve])
    add("ledger", "callbacks-can-open-writer", [reserve, complete,
        {"op": "drain", "reentrant": True},
        {"op": "reserve_effect", "key": "side/reply-2", "payload": {"synthetic": True}},
        {"op": "reserve_effect", "key": "side/reply-1", "payload": {"synthetic": True}}])

    config = {"schema": "EAL/synthetic-coordination-config/1", "nested": {"limit": 2}}
    add("state", "frozen-configuration-events-cancellation", [
        {"op": "config"}, {"op": "initialize", "config": config}, {"op": "config"},
        {"op": "initialize", "config": config},
        {"op": "event", "kind": "synthetic", "content": {"span": {"line": 9}, "at": "literal"}},
        {"op": "cancelled"}, {"op": "cancel"}, {"op": "cancelled"},
        {"op": "reopen"}, {"op": "config"}, {"op": "cancel"}, {"op": "events"}])
    finding = {"finding_id": "synthetic-finding", "status": "unsupported", "count": 0}
    add("state", "issued-findings-immutable-typed", [
        {"op": "issue", "finding": finding}, {"op": "finding", "identity": finding["finding_id"]},
        {"op": "issue", "finding": {"count": 0, "status": "unsupported", "finding_id": finding["finding_id"]}},
        {"op": "issue", "finding": {**finding, "count": False}},
        {"op": "finding", "identity": "absent"}, {"op": "reopen"},
        {"op": "finding", "identity": finding["finding_id"]}])
    add("state", "calls-reserved-before-effect-per-actor-category", [
        {"op": "reserve_call", "actor": "assessor", "category": "gateway", "operation": "query", "maximum": 2},
        {"op": "reserve_call", "actor": "assessor", "category": "gateway", "operation": "query", "maximum": 2},
        {"op": "reserve_call", "actor": "assessor", "category": "gateway", "operation": "lookup", "maximum": 2},
        {"op": "reserve_call", "actor": "reviewer", "category": "gateway", "operation": "lookup", "maximum": 2},
        {"op": "reserve_call", "actor": "assessor", "category": "mcp", "operation": "eal_sources", "maximum": 1},
        {"op": "reserve_call", "actor": "assessor", "category": "mcp", "operation": "eal_assess_known", "maximum": 1},
        {"op": "reopen"}, {"op": "calls"}])
    add("state", "effects-retain-original-payload-and-order", [reserve,
        {"op": "effect", "key": "absent"}, {"op": "effect", "key": reserve["key"]},
        {"op": "reserve_effect", "key": "second", "payload": {"literal": {"payload_digest": "literal"}}},
        complete, {"op": "effects"}, {"op": "drain"}, {"op": "effects"}])

    create = {"op": "create", "run_id": "run", "participants": ["reviewer", "assessor"]}
    send = {"op": "send", "run_id": "run", "payload": message(), "actor": "reviewer"}
    receive = {"op": "receive", "run_id": "run", "recipient": "assessor"}
    ack = {"op": "acknowledge", "run_id": "run", "message_id": "request", "recipient": "assessor"}
    status = {"op": "status", "run_id": "run"}
    add("bus", "ordered-redelivery-and-repeat-ack", [create,
        *[{**send, "payload": message(identity)} for identity in ("request-2", "request-1", "request-3")],
        {**receive, "limit": 2}, {"op": "reopen"}, receive,
        *[{**ack, "message_id": identity} for identity in ("request-2", "request-1", "request-3")],
        {"op": "reopen"}, {**ack, "message_id": "request-2"}, receive, status])
    add("bus", "stable-retry-before-quota-and-after-ack", [{**create, "max_messages": 1},
        send, {"op": "reopen"}, send, {**send, "payload": message(task_digest="b" * 64)},
        {**send, "payload": message("extra")}, receive, ack, send,
        {**send, "payload": message("extra")}, status])
    add("bus", "ack-requires-exact-addressed-receipt", [create, send, ack,
        {**receive, "recipient": "reviewer"}, receive,
        {**ack, "recipient": "reviewer"}, {**ack, "message_id": "unknown"}, receive, status])
    add("bus", "frozen-identities-rejected-without-mutation", [create,
        {**send, "actor": "assessor"}, {**send, "actor": "intruder"},
        {**send, "payload": message(recipient="intruder")},
        {**receive, "recipient": "intruder"}, {**status, "run_id": "wrong"}, status])
    add("bus", "frozen-configuration-reopen", [create, create, {"op": "reopen"},
        {**create, "participants": ["assessor", "reviewer"]},
        {**create, "max_messages": 9}, status])
    unicode_message = message(text="Δé\n\\\"\u0000", nested={"z": 1, "a": True})
    encoded = canonical({"message_id": unicode_message["message_id"], "run_id": "run",
                         "sender": "reviewer", "recipient": "assessor", "version": "1",
                         "kind": "request", "payload": unicode_message})
    for adjustment in (-1, 0):
        add("bus", "utf8-envelope-byte-bound-" + ("below" if adjustment else "exact"), [
            {**create, "max_bytes": len(encoded.encode("utf-8")) + adjustment},
            {**send, "payload": unicode_message}, status])
    evidence = {"execution_mode": "software_test", "execution_id": "synthetic-command/turn1",
                "actual_agent_execution": False, "elapsed_ms": 10, "tokens": None, "cost_usd": None}
    reserve_call = {"op": "reserve", "run_id": "run", "actor": "assessor", "call_id": "assessor-call"}
    finish = {"op": "finish", "run_id": "run", "actor": "assessor", "call_id": "assessor-call", "evidence": evidence}
    add("bus", "worker-call-budget-and-immutable-evidence", [{**create, "max_calls": 2},
        reserve_call, status, {"op": "reopen"}, {**finish, "actor": "reviewer"},
        {**finish, "call_id": "absent"}, finish, finish,
        {**finish, "evidence": {**evidence, "elapsed_ms": 11}},
        {**reserve_call, "actor": "reviewer", "call_id": "reviewer-call"},
        {**reserve_call, "actor": "reviewer", "call_id": "extra-call"}, status])
    return result


def contract_cases(TaskSpec, validate_anchor, make_finding, validate_answer, canonical_digest):
    cases = []
    task = {"task_id": "synthetic-task", "question": "Does this synthetic registered claim hold?",
            "entry_id": "synthetic-entry", "claims": ["ready", "safe"], "profile": "registered_claims/1",
            "requirements": {"context": {"deployment": "synthetic", "sample": 1}}}

    def add(name, op, value, **extra):
        cases.append(deepcopy({"name": name, "op": op, "value": value, **extra}))

    add("generic-valid", "task", task)
    for label, fields in (("extra", {"extra": True}), ("empty-claims", {"claims": []}),
                          ("duplicate-claims", {"claims": ["ready", "ready"]}),
                          ("nonstring-claim", {"claims": [False]}), ("too-many-claims", {"claims": [f"c{i}" for i in range(17)]}),
                          ("unknown-profile", {"profile": "uninstalled/1"}),
                          ("context-override", {"requirements": {"context": {}, "now": NOW}}),
                          ("context-array", {"requirements": {"context": []}}),
                          ("question-whitespace", {"question": " synthetic"}),
                          ("unicode-text-exact", {"task_id": "Δ" * 64}),
                          ("unicode-text-over", {"task_id": "Δ" * 65})):
        add(label, "task", {**task, **fields})
    for count in (30, 31):
        nested = None
        for _ in range(count): nested = [nested]
        add(f"requirements-depth-{count + 2}", "task", {**task, "requirements": {"context": {"nested": nested}}})
    for count in (4093, 4094):
        add(f"requirements-nodes-{count + 3}", "task", {**task, "requirements": {"context": {"items": [0] * count}}})
    overhead = len(json.dumps({"padding": ""}, ensure_ascii=False).encode("utf-8"))
    for size in (16384, 16385):
        add(f"context-bytes-{size}", "task", {**task, "requirements": {"context": {"padding": "x" * (size - overhead)}}})
    workload = {"service": "synthetic-service", "build_id": "synthetic-build", "run_id": "synthetic-report",
                "concurrent_clients": 2, "dataset": "synthetic-data"}
    api = {**task, "profile": "api_load/1", "requirements": {
        "context": {key: workload[key] for key in ("service", "build_id", "dataset")},
        "workload": workload, "criteria": {"minimum_requests": 100, "maximum_sample_p95_ms": 200,
            "maximum_error_rate_percent": 1, "evidence_max_age_seconds": 30},
        "claim_outputs": {"ready": "passes", "safe": "fails"}}}
    add("api-valid", "task", api)
    for label, path, value in (
        ("api-clients-bool", ("workload", "concurrent_clients"), True),
        ("api-clients-float", ("workload", "concurrent_clients"), 2.0),
        ("api-request-count-float", ("criteria", "minimum_requests"), 100.0),
        ("api-negative-age", ("criteria", "evidence_max_age_seconds"), -1),
        ("api-count-huge", ("criteria", "minimum_requests"), 10 ** 101),
        ("api-output-invalid", ("claim_outputs", "ready"), "proved"),
        ("api-output-array", ("claim_outputs", "ready"), []),
    ):
        changed = deepcopy(api)
        changed["requirements"][path[0]][path[1]] = value
        add(label, "task", changed)
    base = {"schema": "EAL/MAS-coordination-task/3", "task": task, "source_digest": "a" * 64,
            "method_registry_fingerprint": "b" * 64, "registered_context": task["requirements"]["context"], "pinned_at": NOW}
    anchor = {**base, "task_digest": canonical_digest(base)}
    add("fixed-anchor-valid", "anchor", anchor)
    add("anchor-tampered-question", "anchor", {**anchor, "task": {**task, "question": "changed synthetic question"}})
    add("anchor-unknown-field", "anchor", {**anchor, "extra": True})
    add("anchor-uppercase-digest", "anchor", {**anchor, "source_digest": "A" * 64})
    add("anchor-invalid-time", "anchor", {**anchor, "pinned_at": "2040-99-99T00:00:00Z"})
    rows = {claim: {"assessment_id": f"synthetic-assessment-{claim}", "status": status,
                    "assessed_at": NOW, "collected_count": 1, "reused_count": 0}
            for claim, status in zip(task["claims"], ("supported", "contested"))}
    check = {"ok": True, "code": None, "detail": "Synthetic correspondence only"}
    finding = make_finding(anchor, check, rows)

    def answer(finding, conclusion):
        return {"task_digest": anchor["task_digest"], "finding_id": finding["finding_id"],
                "claims": {claim: {key: finding["assessments"].get(claim, {}).get(key)
                    for key in ("assessment_id", "status", "assessed_at")} for claim in task["claims"]},
                "scope": {"context": task["requirements"]["context"], "requirements": task["requirements"]},
                "conclusion": conclusion, "qualification": "conditional_on_registered_argument"}

    add("finding-complete-content-identity", "finding", anchor, check=check, assessments=rows)
    valid = answer(finding, "assessed")
    add("answer-supported-and-contested", "answer", valid, anchor=anchor, finding=finding)
    for label, mutation in (
        ("qualification-proof", lambda v: v.update(qualification="verified_prose")),
        ("conclusion-upgrade", lambda v: v.update(conclusion="proved")),
        ("extra-answer-field", lambda v: v.update(extra=True)),
        ("claim-status-upgrade", lambda v: v["claims"]["safe"].update(status="supported")),
        ("claim-time-refresh", lambda v: v["claims"]["ready"].update(assessed_at="2040-01-02T00:00:00Z")),
        ("claim-extra-field", lambda v: v["claims"]["ready"].update(collected_count=1)),
        ("scope-type-coercion", lambda v: v["scope"]["context"].update(sample=True)),
        ("missing-claim", lambda v: v["claims"].pop("safe")),
        ("missing-task-identity", lambda v: v.update(task_digest="c" * 64)),
    ):
        changed = deepcopy(valid)
        mutation(changed)
        add(label, "answer", changed, anchor=anchor, finding=finding)
    for status in ("unsupported", "out_of_scope"):
        changed_rows = deepcopy(rows)
        changed_rows["ready"]["status"] = status
        changed_finding = make_finding(anchor, check, changed_rows)
        add("answer-preserves-" + status, "answer", answer(changed_finding, "assessed"), anchor=anchor, finding=changed_finding)
    incomplete = make_finding(anchor, check, {"ready": rows["ready"]})
    add("answer-partial-selection-unresolved", "answer", answer(incomplete, "unresolved"), anchor=anchor, finding=incomplete)
    for code in ("source_changed", "context_mismatch", "workload_mismatch", "criteria_mismatch", "evidence_unavailable", "effect_outcome_unknown"):
        disposition = make_finding(anchor, {"ok": False, "code": code, "detail": "Synthetic disposition only"})
        conclusion = "unresolved" if code in {"evidence_unavailable", "effect_outcome_unknown"} else "out_of_scope"
        add("answer-disposition-" + code, "answer", answer(disposition, conclusion), anchor=anchor, finding=disposition)
        if code == "effect_outcome_unknown":
            disposition["collected_count"] = disposition["reused_count"] = None
            disposition["finding_id"] = "finding_" + canonical_digest({k: v for k, v in disposition.items() if k != "finding_id"})
            add("answer-unknown-counts-remain-null", "answer", answer(disposition, "unresolved"), anchor=anchor, finding=disposition)
    for case in cases:
        try:
            op, value = case["op"], deepcopy(case["value"])
            if op == "task": output = TaskSpec.from_dict(value).to_dict()
            elif op == "anchor": output = validate_anchor(value).to_dict()
            elif op == "finding": output = make_finding(value, case["check"], case["assessments"])
            else: output = validate_answer(case["anchor"], case["finding"], value)
            case["expected"] = {"value": output}
        except (ValueError, TypeError, KeyError, OSError) as error:
            case["expected"] = {"error": str(error), "kind": type(error).__name__}
    return cases


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = args.reference.resolve()
    if subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip() != REVISION:
        raise SystemExit("Reference checkout must be pinned at " + REVISION)
    if subprocess.check_output(["git", "-C", str(reference), "status", "--porcelain", "--", "src/eal", "grammar", "tests"], text=True):
        raise SystemExit("Reference source, grammar and tests must remain unchanged")
    sys.path.insert(0, str(reference / "src"))
    from eal.coordination.ledger import EffectLedger, canonical_json
    from eal.coordination.state import State
    from eal.coordination.bus import JsonBus
    from eal.coordination.contracts import TaskSpec, validate_anchor, make_finding
    from eal.coordination.profiles import validate_answer
    from eal.evaluator import canonical_digest

    canonical_inputs = ["null", "true", "1", "1.0", "1e0", "-0.0", "0.0", "1e-7", "1e20", "1e21",
        '{"z":1,"a":[true,1,1.0,null,"Δé\\n"]}',
        '{"$serde_json::private::Number":"1","span":{"line":7}}', "1" + "0" * 100]
    canonical_cases = [{"input": raw, "expected": canonical_json(json.loads(raw))} for raw in canonical_inputs]
    captured = scenarios(canonical_json)
    for case in captured:
        with TemporaryDirectory() as directory:
            path = Path(directory) / "state.sqlite"
            group = case["group"]
            constructor = {"ledger": EffectLedger, "state": State, "bus": JsonBus}[group]
            instance = constructor(path)
            with patch("eal.coordination.state.utc_now", lambda: NOW):
                for step in case["steps"]:
                    call = deepcopy({k: v for k, v in step.items() if k != "expected"})
                    op = call.pop("op")
                    callbacks = []
                    try:
                        if op == "reopen": instance = constructor(path); value = None
                        elif op == "drain":
                            def send(item):
                                callbacks.append({"send": deepcopy(item)})
                                if call.get("fail_send") == item["message_id"]:
                                    raise ValueError("Synthetic send confirmation unavailable")
                                if call.get("reentrant"):
                                    EffectLedger(path).reserve_effect("side/" + item["message_id"], {"synthetic": True})
                            def ack(key):
                                callbacks.append({"ack": key})
                                if call.get("fail_ack") == key:
                                    raise ValueError("Synthetic ACK confirmation unavailable")
                            value = instance.drain_outbox(send, ack)
                        elif op == "create":
                            participants = call.pop("participants")
                            value = instance.create(call.pop("run_id"), *participants, **call)
                        elif op == "initialize": value = instance.initialize(call["config"])
                        elif op == "event": value = instance.event(call["kind"], call["content"])
                        elif op == "issue": value = instance.issue(call["finding"])
                        elif op == "finding": value = instance.finding(call["identity"])
                        elif op == "reserve_effect": value = instance.reserve_effect(call["key"], call["payload"])
                        elif op == "complete_effect": value = instance.complete_effect(call["key"], call["result"], call["outbox"])
                        else: value = getattr(instance, op)(**call)
                        step["expected"] = {"value": value}
                    except (ValueError, TypeError, KeyError, OSError) as error:
                        step["expected"] = {"error": str(error), "kind": type(error).__name__}
                    if op == "drain": step["expected"]["callbacks"] = callbacks
    contracts = contract_cases(TaskSpec, validate_anchor, make_finding, validate_answer, canonical_digest)
    output = {"schema": "EAL/coordination-reference/1", "source_revision": REVISION,
        "source_sha256": {name: hashlib.sha256((reference / name).read_bytes()).hexdigest() for name in SOURCE_FILES},
        "comparison": "Exact canonical bytes, SHA-256 digests, errors, complete public snapshots and ordered callback traces. Only the State clock is fixed; no output fields are scrubbed. All labels are synthetic and no external execution or measurement occurs.",
        "clock": NOW, "canonical": canonical_cases, "scenarios": captured, "contracts": contracts}
    encoded = json.dumps(output, ensure_ascii=False, allow_nan=False, indent=2) + "\n"
    path = ROOT / "fixtures/coordination/reference.json"
    if args.check:
        if path.read_text(encoding="utf-8") != encoded: raise SystemExit("Coordination reference drift")
    else: path.write_text(encoded, encoding="utf-8")
    print(json.dumps({"canonical": len(canonical_cases), "scenarios": len(captured),
                      "steps": sum(len(case["steps"]) for case in captured), "contracts": len(contracts)}))


if __name__ == "__main__":
    main()
