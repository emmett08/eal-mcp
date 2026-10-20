#!/usr/bin/env python3
"""Development orchestration: installed native coordination with no child PATH."""
from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile

SOURCE = '''language "EAL/3"
environment lab {
 require site == "bench"
}
tool reader {
 version "1"
}
evidence reading {
 tool reader
 kind test
 environment lab
 max_age 3600
 input {"sample":1}
 require ok == true
}
reasoning measured {
 method "structured/1"
 rationale "The supplied report supports the selected test result."
}
claim works {
 statement "The supplied test passed at this bench."
 environment lab
}
argument result = [evidence reading] via measured => works
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prefix", type=Path, required=True, help="Installed native binary directory")
    parser.add_argument("--worker", type=Path, required=True, help="Built coordination_software_worker example")
    parser.add_argument("--workspace", type=Path)
    args = parser.parse_args()
    binaries, worker = args.prefix.resolve(), args.worker.resolve()
    for name in ("eal", "eal-coordinate", "eal-mcp", "eal-mas-server", "mas-protocol-codec", "eal-command-supervisor", "eal-method-worker"):
        assert (binaries / name).is_file(), f"Missing installed binary: {binaries/name}"
    assert worker.is_file(), f"Missing native software worker: {worker}"
    root = args.workspace.resolve() if args.workspace else Path(tempfile.mkdtemp(prefix="eal-coord-installed-smoke-"))
    if root.exists() and (not root.is_dir() or any(root.iterdir())):
        parser.error("--workspace must name an empty directory or a new location")
    root.mkdir(parents=True, exist_ok=True)
    observed_at = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    context = {"site": "bench"}
    (root / "case.eal").write_text(SOURCE, encoding="utf-8")
    (root / "context.json").write_text(json.dumps(context), encoding="utf-8")
    registry = root / "tools.toml"
    registry.write_text("[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n", encoding="utf-8")
    (root / "observation.json").write_text(json.dumps({
        "observed_at": observed_at, "context": context,
        "request": {"tool": "reader", "tool_version": "1", "input": {"sample": 1}, "context": context},
        "value": {"ok": True, "secret": "collector-secret"}}), encoding="utf-8")
    (root / "worker-environment.json").write_text('{"PATH":""}', encoding="utf-8")
    environment = {"PATH": "", "LANG": "C.UTF-8", "NO_PROXY": "*"}
    launches = []

    def run(executable, arguments, *, code=0, error=False):
        assert environment["PATH"] == ""
        command = [str(binaries / executable), *map(str, arguments)]
        completed = subprocess.run(command, cwd=root, env=dict(environment), stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, timeout=60)
        launches.append({"executable": executable, "operation": str(arguments[0]) if arguments else None,
                         "returncode": completed.returncode})
        assert completed.returncode == code, (command, completed.returncode, completed.stdout.decode(), completed.stderr.decode())
        if error:
            assert completed.stdout == b"", completed.stdout
            result = json.loads(completed.stderr)
            assert isinstance(result.get("error"), dict), result
        else:
            result = json.loads(completed.stdout)
            assert isinstance(result, dict), result
        return result

    common = ["--workspace", root, "--registry", registry, "--methods", "engineering"]

    def cli(*arguments):
        return run("eal", [*common, *arguments])

    def coordinate(operation, directory, *arguments, **kwargs):
        return run("eal-coordinate", [operation, "--run", directory, *arguments], **kwargs)

    entry = cli("register", "case.eal", "--entry-id", "demo", "--claim", "works", "--context", "@context.json")
    assert entry["claims"] == ["works"]
    task = {"task_id": "installed-native-review", "question": "Did the supplied test pass?",
            "entry_id": "demo", "claims": ["works"], "profile": "registered_claims/1",
            "requirements": {"context": context}}
    server = {"workspace": str(root), "registry": str(registry), "database": None,
              "methods": "engineering", "limits": None}
    execute = ["--execution-mode", "software_test", "--worker-argv", json.dumps([str(worker)]),
               "--worker-environment", root / "worker-environment.json"]
    reports = []

    def spec(label, transport):
        path = root / (label + "-spec.json")
        path.write_text(json.dumps({"server": server, "task": task, "transport": transport}), encoding="utf-8")
        return path

    def exported(directory, label):
        path = root / (label + "-export.json")
        value = coordinate("export", directory, "--output", path)
        assert json.loads(path.read_bytes()) == value
        return value

    def effects(value):
        return {effect["key"]: effect for effect in value["effects"]}

    def completed(directory, label, expected_collected):
        status = coordinate("execute", directory, *execute)
        assert status["status"] == "completed", status
        value = exported(directory, label)
        rows = effects(value)
        finding = rows["assessment"]["result"]
        assert finding["check"]["ok"] is True
        assessment = finding["assessments"]["works"]
        expected = {"task_digest": value["config"]["anchor"]["task_digest"],
                    "finding_id": finding["finding_id"],
                    "claims": {"works": {key: assessment[key] for key in ("assessment_id", "status", "assessed_at")}},
                    "scope": {"context": context, "requirements": task["requirements"]},
                    "conclusion": "assessed", "qualification": "conditional_on_registered_argument"}
        assert status["answer"] == expected
        assert assessment["status"] == "supported"
        assert assessment["source_digest"] == entry["source_digest"]
        assert finding["collected_count"] == expected_collected
        assert finding["reused_count"] == 1 - expected_collected
        assert assessment["collected_count"] == expected_collected
        assert assessment["reused_count"] == 1 - expected_collected
        assert assessment["packet"] == finding["packets"]["works"]
        assert finding["packets"]["works"]["assessment_id"] == assessment["assessment_id"]
        assert finding["packets"]["works"]["source_digest"] == assessment["source_digest"]
        assert finding["explanations"]["works"]["assessed_at"] == assessment["assessed_at"]
        assert dt.datetime.fromisoformat(assessment["assessed_at"].replace("Z", "+00:00")) >= dt.datetime.fromisoformat(value["config"]["anchor"]["pinned_at"].replace("Z", "+00:00"))
        execution_ids, pids = set(), set()
        for role in ("assessor", "reviewer"):
            row = rows["worker:" + role]
            result = row["result"]
            raw = base64.b64decode(result["raw_base64"], validate=True)
            assert hashlib.sha256(raw).hexdigest() == result["raw_digest"]
            assert json.loads(raw) == result["response"]
            assert result["accepted"] is True and result["execution_mode"] == "software_test"
            assert result["execution_id"].startswith("command:" + row["payload"]["call_id"] + ":")
            evidence = result["evidence"]
            assert evidence["returncode"] == 0 and evidence["stdout_bytes"] == len(raw)
            assert evidence["call_id"] == row["payload"]["call_id"]
            assert evidence["model"] is None and evidence["usage"] is None and evidence["receipt_verification"] is None
            assert isinstance(evidence["supervisor_pid"], int) and evidence["supervisor_pid"] > 0
            assert evidence["output_truncated"] is False
            assert row["acknowledged"] is True
            assert result["response"]["consumed_message_ids"] == [message["message_id"] for message in row["payload"]["context"]["received_messages"]]
            execution_ids.add(result["execution_id"]); pids.add(evidence["supervisor_pid"])
        assert len(execution_ids) == len(pids) == 2
        assert value["transport"]["usage"]["calls"] == 2 and value["transport"]["usage"]["messages"] == 2
        assert all(message["acked"] for message in value["transport"]["messages"])
        calls = status["calls"]
        assert len([row for row in calls if row["category"] == "mcp"]) == 6
        assert len([row for row in calls if row["category"] == "gateway"]) == 2
        assert coordinate("resume", directory)["answer"] == expected
        assert coordinate("execute", directory, *execute)["answer"] == expected
        after = exported(directory, label + "-replayed")
        assert after["effects"] == value["effects"]
        assert after["status"]["calls"] == calls
        assert after["transport"]["usage"] == value["transport"]["usage"]
        return {"transport": value["config"]["transport"]["kind"], "run_id": status["run_id"],
                "assessment_id": assessment["assessment_id"], "assessed_at": assessment["assessed_at"],
                "status": status["status"], "collected_count": finding["collected_count"],
                "reused_count": finding["reused_count"], "worker_executions": sorted(execution_ids),
                "mcp_calls": 6, "gateway_calls": 2, "replay_added_calls_or_executions": False}

    for kind, transport in (("json", {"kind": "json"}), ("lab", {"kind": "lab", "runtime": str(binaries)})):
        for index in (1, 2):
            label = f"{kind}-{index}"
            directory = root / label
            started = coordinate("start", directory, "--spec", spec(label, transport))
            assert started["status"] == "ready_assessor"
            reports.append(completed(directory, label, int(kind == "json" and index == 1)))

    database = root / ".eal" / "runs.sqlite3"

    def application_counts():
        with sqlite3.connect(database) as connection:
            return dict(connection.execute("SELECT kind,COUNT(*) FROM records GROUP BY kind"))

    directory = root / "unknown-effect"
    coordinate("start", directory, "--spec", spec("unknown-effect", {"kind": "json"}))
    reserved = coordinate("reserve", directory, "--role", "assessor", "--execution-mode", "software_test")
    call_id = reserved["call_id"]
    before = application_counts()
    state_database = directory / "coordination.sqlite"
    with sqlite3.connect(state_database) as connection:
        connection.execute("""CREATE TRIGGER smoke_reject_assessment_receipt BEFORE UPDATE OF state ON effects
            WHEN NEW.effect_key='assessment' AND NEW.state='committed'
            BEGIN SELECT RAISE(ABORT,'injected assessment receipt commit failure'); END""")
    rejected = coordinate("query", directory, "--call-id", call_id, code=1, error=True)
    assert "injected assessment receipt commit failure" in rejected["error"]["message"]
    after_effect = application_counts()
    assert after_effect["assessment"] == before["assessment"] + 1, (before, after_effect)
    status_after_effect = coordinate("status", directory)
    mcp_before = [row for row in status_after_effect["calls"] if row["category"] == "mcp"]
    assert len(mcp_before) == 4
    with sqlite3.connect(state_database) as connection:
        assert connection.execute("SELECT state,result FROM effects WHERE effect_key='assessment'").fetchone() == ("reserved", None)
        connection.execute("DROP TRIGGER smoke_reject_assessment_receipt")
    unknown = coordinate("query", directory, "--call-id", call_id)
    assert unknown["check"]["code"] == "effect_outcome_unknown"
    assert unknown["assessments"] == unknown["packets"] == unknown["explanations"] == {}
    assert unknown["collected_count"] is None and unknown["reused_count"] is None
    assert application_counts() == after_effect
    assert [row for row in coordinate("status", directory)["calls"] if row["category"] == "mcp"] == mcp_before
    request = reserved["context"]["received_messages"][0]
    response = {"message": {"schema": "EAL/MAS-coordination/3", "kind": "disposition",
        "message_id": "unknown-assessment-handoff", "task_id": task["task_id"], "task_digest": unknown["task_digest"],
        "sender": "assessor", "recipient": "reviewer", "reply_to": request["message_id"], "attempt": 0,
        "finding_id": unknown["finding_id"], "code": "effect_outcome_unknown", "outcome": "incomplete"},
        "consumed_message_ids": [request["message_id"]]}
    response_path = root / "unknown-assessor.json"
    raw = (json.dumps(response, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    response_path.write_bytes(raw)
    retained = coordinate("record", directory, "--call-id", call_id, "--response", response_path,
                          "--execution-id", "software-fixture/injected-assessor", "--execution-mode", "software_test")
    assert retained["accepted"] is True and base64.b64decode(retained["raw_base64"]) == raw
    final = coordinate("execute", directory, *execute)
    assert final["status"] == "completed_unassessed"
    assert final["answer"] == {"task_digest": unknown["task_digest"], "finding_id": unknown["finding_id"],
        "claims": {"works": {"assessment_id": None, "status": None, "assessed_at": None}},
        "scope": {"context": context, "requirements": task["requirements"]}, "conclusion": "unresolved",
        "qualification": "conditional_on_registered_argument"}
    unknown_export = exported(directory, "unknown-effect")
    rows = effects(unknown_export)
    assert rows["assessment"]["status"] == "unknown" and rows["assessment"]["result"] is None
    assert rows["worker:assessor"]["result"]["execution_id"] == "software-fixture/injected-assessor"
    reviewer = rows["worker:reviewer"]["result"]
    assert reviewer["accepted"] is True and reviewer["execution_id"].startswith("command:")
    assert reviewer["evidence"]["returncode"] == 0
    assert unknown_export["transport"]["usage"]["calls"] == unknown_export["transport"]["usage"]["messages"] == 2
    assert application_counts() == after_effect
    assert [row for row in final["calls"] if row["category"] == "mcp"] == mcp_before
    coordinate("resume", directory)
    coordinate("execute", directory, *execute)
    replayed = exported(directory, "unknown-effect-replayed")
    assert replayed["effects"] == unknown_export["effects"]
    assert replayed["status"]["calls"] == unknown_export["status"]["calls"]
    assert application_counts() == after_effect
    result = {"result": "passed", "prefix": str(binaries), "worker": str(worker), "workspace": str(root),
        "observed_at": observed_at, "profile": "engineering", "child_path": "", "completed_runs": reports,
        "unknown_effect_recovery": {"status": final["status"], "conclusion": final["answer"]["conclusion"],
            "assessment_committed_before_missing_coordination_receipt": True, "additional_assessments": 1,
            "mcp_calls_before_and_after_recovery": len(mcp_before), "recovery_reassessed": False,
            "manual_assessor_retained_exactly": True, "reviewer_only_command": reviewer["execution_id"],
            "all_unassessed_fields_null": True, "resume_added_calls_or_executions": False},
        "direct_native_launches": len(launches)}
    (root / "smoke-result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
