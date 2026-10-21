#!/usr/bin/env python3
"""Exercise pinned native packet machinery; never generate model-study outcomes.

The workspaces and their operator-only observations are temporary. The registered
requests go through the actual eal-host -> MCP stdio -> eal-mcp path. This is a
calibration diagnostic, not a workload sample or a model-retention experiment.
"""
import argparse
import hashlib
import json
import platform
import sqlite3
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

REFERENCE = "de3a99dfb1003f43f75d7bfe18007179ecb4f4aa"
CANDIDATE = "71661ea05225157961fe32b355db62dac42e3bba"
NOW = "2040-01-01T00:00:01Z"
LATER = "2040-01-01T00:00:02Z"
SECRET = "NATIVE_PROBE_RAW_OBSERVATION_MUST_STAY_PRIVATE"
CONTEXT = {"site": "bench"}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def compact(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def utc():
    return datetime.now(timezone.utc).isoformat()


def command(args, *, input_text=None, expect=0, cwd=None):
    result = subprocess.run(args, input=input_text, text=True, capture_output=True,
                            timeout=240, cwd=cwd)
    if result.returncode != expect:
        raise RuntimeError(f"{args[0]} returned {result.returncode}: "
                           f"{result.stdout[:1500]} {result.stderr[:1500]}")
    return result


def revision(root, expected):
    actual = command(["git", "rev-parse", "HEAD"], cwd=root).stdout.strip()
    if actual != expected:
        raise RuntimeError(f"Expected revision {expected}, found {actual}")
    if command(["git", "status", "--porcelain", "--untracked-files=no"], cwd=root).stdout:
        raise RuntimeError("Pinned checkout has modified tracked files")
    return actual


HEADER = '''language "EAL/3"
environment lab {
  require site == "bench"
}
tool reader {
  version "1"
}
evidence observation {
  tool reader
  kind test
  environment lab
  max_age 864000
  require ok == true
}
reasoning authored {
  method "structured/1"
  rationale "The calibration predicate supplies conditional authored support."
}
'''


def source(kind):
    root = ('claim ready {\n  statement "The calibrated service can proceed in the '
            'bench environment."\n  environment lab\n}\n')
    if kind == "simple":
        return HEADER + root + "argument route = [evidence observation] via authored => ready\n"
    if kind == "count_pressure":
        return HEADER + root + "".join(
            f"argument route{i:02d} = [evidence observation] via authored => ready\n"
            for i in range(40))
    if kind != "byte_pressure":
        raise ValueError(kind)
    pieces = [HEADER, root]
    for i in range(6):
        sentence = (f"Calibration premise {i} applies to release {i} in the bench "
                    "environment and requires the recorded acceptance predicate. ")
        statement = (sentence * 15)[:1800]
        pieces.append(f'claim premise{i} {{\n  statement {json.dumps(statement)}\n'
                      '  environment lab\n}\n')
        pieces.append(f"argument premise_route{i} = [evidence observation] via authored => premise{i}\n")
    pieces.append("argument combined = [premises " + ", ".join(f"premise{i}" for i in range(6))
                  + "] via authored => ready\n")
    return "".join(pieces)


def setup(workspace, kind):
    workspace.mkdir()
    workspace.joinpath("case.eal").write_text(source(kind), encoding="utf-8")
    workspace.joinpath("tools.toml").write_text(
        "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
        encoding="utf-8")
    observation = {"value": {"ok": True, "private": SECRET},
                   "observed_at": "2040-01-01T00:00:00Z", "context": CONTEXT,
                   "request": {"tool": "reader", "tool_version": "1", "input": {},
                               "context": CONTEXT}}
    workspace.joinpath("observation.json").write_text(compact(observation), encoding="utf-8")


class Runtime:
    def __init__(self, root, workspace):
        self.bin = root / "target" / "debug"
        self.workspace = workspace

    def cli(self, *args):
        result = command([str(self.bin / "eal"), "--workspace", str(self.workspace),
                          "--registry", str(self.workspace / "tools.toml"), *args])
        return json.loads(result.stdout)

    def register(self):
        return self.cli("register", "case.eal", "--entry-id", "demo", "--claim", "ready",
                        "--context", compact(CONTEXT))

    def assess(self, now=NOW):
        return self.cli("assess-known", "demo", "--claim", "ready", "--now", now)

    def host(self, request, expect=0):
        result = command([str(self.bin / "eal-host"), "--workspace", str(self.workspace),
                          "--registry", str(self.workspace / "tools.toml"),
                          "--exposure", "registered", "--known-entry", "demo"],
                         input_text=compact(request), expect=expect)
        return json.loads(result.stdout), len(result.stdout.encode("utf-8"))

    def retrieve(self, assessment_id, max_bytes, max_statement_bytes):
        response, host_bytes = self.host({"operation": "get_known_packet", "entry_id": "demo",
            "assessment_id": assessment_id, "claim": "ready",
            "packet_limits": {"max_bytes": max_bytes,
                              "max_statement_bytes": max_statement_bytes}})
        registered = response["result"]
        # Measure the actual compact Rust JSON carried by MCP, not a Python
        # reserialization whose floating-point spelling could differ.
        raw = response["content"][0]["text"]
        start = raw.index('"packet":') + len('"packet":')
        parsed, end = json.JSONDecoder().raw_decode(raw, start)
        assert parsed == registered["packet"]
        packet_bytes = len(raw[start:end].encode("utf-8"))
        assert packet_bytes <= max_bytes
        assert SECRET not in raw
        return registered, {"max_bytes": max_bytes, "max_statement_bytes": max_statement_bytes,
            "packet_utf8_bytes": packet_bytes, "registered_envelope_utf8_bytes": len(raw.encode("utf-8")),
            "host_inspection_envelope_utf8_bytes": host_bytes,
            "packet_sha256": sha(raw[start:end].encode("utf-8")),
            "summary_complete": parsed["summary_complete"], "omitted": parsed["omitted"],
            "claims": len(parsed.get("claims", {})),
            "premise_claims": len(parsed.get("premise_claims", {})),
            "arguments": len(parsed.get("arguments", {}))}


def records(workspace):
    # Operator-only instrumentation. The model or registered client never gets
    # this database or its raw observations.
    with sqlite3.connect(f"file:{workspace / '.eal/runs.sqlite3'}?mode=ro", uri=True) as db:
        return dict(db.execute("SELECT kind, count(*) FROM records GROUP BY kind"))


def normalise_packet(packet):
    """Remove run-generated IDs while preserving repeated identity relations."""
    replacements = {packet["assessment_id"]: "<ASSESSMENT>",
                    packet["collection_id"]: "<COLLECTION>"}
    for name, item in packet.get("evidence", {}).items():
        if item.get("observation_id"):
            replacements[item["observation_id"]] = f"<OBSERVATION:{name}>"

    def visit(value):
        if isinstance(value, dict):
            return {k: visit(v) for k, v in value.items()}
        if isinstance(value, list):
            return [visit(v) for v in value]
        return replacements.get(value, value) if isinstance(value, str) else value
    return visit(packet)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate-root", type=Path, required=True)
    parser.add_argument("--reference-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    started = utc()
    start_ns = time.monotonic_ns()
    roots = {"reference": args.reference_root.resolve(), "candidate": args.candidate_root.resolve()}
    binaries = {}
    for name, expected in [("reference", REFERENCE), ("candidate", CANDIDATE)]:
        revision(roots[name], expected)
        command(["cargo", "build", "--locked", "-p", "eal-cli", "-p", "eal-mcp",
                 "-p", "eal-runtime", "--bins"], cwd=roots[name])
        binaries[name] = {binary: sha((roots[name] / "target/debug" / binary).read_bytes())
                          for binary in ["eal", "eal-host", "eal-mcp", "eal-command-supervisor", "eal-method-worker"]}
    report = {"status": "native_machinery_diagnostic_only", "protocol": "EAL-CONTEXT-001",
        "harness_sha256": sha(Path(__file__).read_bytes()), "invocation": ["python", *sys.argv],
        "reference_sha": REFERENCE, "candidate_sha": CANDIDATE,
        "binary_sha256": binaries, "started_at_utc": started,
        "python": platform.python_version(), "platform": platform.platform(),
        "model_sessions": 0, "provider_usage": None,
        "provider_usage_reason": "No language model or provider endpoint is invoked by this diagnostic.",
        "limitations": ["These hand-constructed fixtures calibrate native packet machinery only.",
            "They do not implement or estimate the 24-scenario long-session model pilot.",
            "Default parity normalises assessment, collection and observation IDs only.",
            "The old executable is used only for packet compatibility, not a timed workflow comparison."],
        "checks": {}, "fixtures": {}}
    checks = report["checks"]
    with tempfile.TemporaryDirectory(prefix="eal-context-native-") as tmp:
        base = Path(tmp)
        # Independently generated stores at both revisions exercise actual
        # default production projections. Fixture source and clock are common.
        for kind in ["simple", "byte_pressure", "count_pressure"]:
            assessments = {}
            for name in ["reference", "candidate"]:
                workspace = base / f"{name}-{kind}"
                setup(workspace, kind)
                runtime = Runtime(roots[name], workspace)
                runtime.register()
                assessments[name] = runtime.assess()
            ref_packet = normalise_packet(assessments["reference"]["packet"])
            candidate_packet = normalise_packet(assessments["candidate"]["packet"])
            checks[f"{kind}_default_reference_parity"] = ref_packet == candidate_packet
            assert checks[f"{kind}_default_reference_parity"]
            report["fixtures"][kind] = {"source_sha256": sha(source(kind).encode("utf-8")),
                "normalised_default_packet_sha256": sha(compact(candidate_packet).encode("utf-8"))}
            runtime = Runtime(roots["candidate"], base / f"candidate-{kind}")
            initial = assessments["candidate"]
            before = records(runtime.workspace)
            ladder = []
            for max_bytes, max_statement_bytes in [(4096, 1024), (16384, 1024), (65536, 4096)]:
                retrieved, reading = runtime.retrieve(initial["assessment_id"], max_bytes, max_statement_bytes)
                assert retrieved["assessment_id"] == initial["assessment_id"]
                assert retrieved["assessed_at"] == NOW
                assert retrieved["source_digest"] == initial["source_digest"]
                assert retrieved["context_fingerprint"] == initial["context_fingerprint"]
                assert retrieved["packet"]["method_registry_fingerprint"] == initial["packet"]["method_registry_fingerprint"]
                ladder.append(reading)
            after = records(runtime.workspace)
            checks[f"{kind}_retrieval_no_records_added"] = before == after
            assert before == after
            report["fixtures"][kind].update({"retrieval_ladder": ladder,
                "records_before_retrieval": before, "records_after_retrieval": after})
            if kind == "byte_pressure":
                checks["byte_pressure_4096_reports_omission"] = not ladder[0]["summary_complete"]
                checks["byte_pressure_65536_recovers_statements"] = ladder[2]["summary_complete"]
                assert checks["byte_pressure_4096_reports_omission"]
                assert checks["byte_pressure_65536_recovers_statements"]
            if kind == "count_pressure":
                checks["count_pressure_remains_incomplete_at_65536"] = not ladder[2]["summary_complete"]
                assert checks["count_pressure_remains_incomplete_at_65536"]
            if kind != "simple":
                continue
            # Editing the current source must not rewrite a stored historical
            # assessment; a later current assessment gets a different identity.
            runtime.workspace.joinpath("case.eal").write_text(
                source(kind).replace("The calibrated service can proceed", "The revised calibrated service can proceed"),
                encoding="utf-8")
            later = runtime.assess(LATER)
            historical, historical_reading = runtime.retrieve(initial["assessment_id"], 16384, 1024)
            checks["source_edit_creates_new_assessment"] = (
                later["source_digest"] != initial["source_digest"] and later["assessment_id"] != initial["assessment_id"])
            checks["historical_retrieval_preserves_original_revision_and_time"] = (
                historical["source_digest"] == initial["source_digest"] and historical["assessed_at"] == NOW
                and historical["packet"] == initial["packet"])
            report["fixtures"][kind]["historical_retrieval_after_edit"] = historical_reading
            # A valid unexposed catalogue entry is denied by the registered tool
            # even when the stored assessment otherwise exists.
            runtime.workspace.joinpath("unexposed.eal").write_text(
                runtime.workspace.joinpath("case.eal").read_text(encoding="utf-8"), encoding="utf-8")
            runtime.cli("register", "unexposed.eal", "--entry-id", "unexposed", "--claim", "ready",
                        "--context", compact(CONTEXT))
            rejected, _ = runtime.host({"operation": "get_known_packet", "entry_id": "unexposed",
                "assessment_id": later["assessment_id"], "claim": "ready"}, expect=1)
            checks["registered_allowlist_rejection"] = rejected.get("is_error") is True
            report["fixtures"][kind]["unexposed_entry_rejection"] = rejected
            # Exercise the real ModelContextBuilder path separately: prepare is
            # a new assessment and is not used as a historical-retrieval proxy.
            prepared = runtime.cli("model-context", "demo", "--claim", "ready", "--now", LATER,
                                   "--question", "What does this calibrated evidence establish?")
            messages = prepared["messages"]
            checks["native_model_context_messages_available"] = isinstance(messages, list) and bool(messages)
            checks["raw_observation_secret_excluded"] = SECRET not in compact(prepared)
            report["fixtures"][kind]["model_context_messages_sha256"] = sha(compact(messages).encode("utf-8"))
    assert all(checks.values()), checks
    report["checks_passed"] = len(checks)
    report["checks_failed"] = 0
    report["finished_at_utc"] = utc()
    report["elapsed_seconds"] = (time.monotonic_ns() - start_ns) / 1e9
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(compact({"status": report["status"], "checks_passed": len(checks),
                   "model_sessions": 0, "output": str(args.output)}))


if __name__ == "__main__":
    main()
