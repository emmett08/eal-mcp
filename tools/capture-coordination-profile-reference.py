#!/usr/bin/env python3
"""Freeze pure coordination requirement and answer checks; execute no workflows."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = args.reference.resolve()
    if subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip() != REVISION:
        raise SystemExit("Reference revision differs")
    if subprocess.check_output(["git", "-C", str(reference), "status", "--porcelain", "--", "src/eal", "tests"], text=True):
        raise SystemExit("Reference source must remain unchanged")
    sys.path.insert(0, str(reference / "src"))
    from eal.coordination.contracts import TASK_SCHEMA, TaskSpec, make_finding
    from eal.coordination.profiles import validate_requirements, validate_answer
    from eal.evaluator import canonical_digest

    claims = ["ready", "challenged"]
    generic = {"context": {"dataset": "synthetic", "count": 1}}
    api = {"context": {"service": "orders-api", "build_id": "build-1", "dataset": "synthetic"},
           "workload": {"service": "orders-api", "build_id": "build-1", "dataset": "synthetic", "run_id": "run-1", "concurrent_clients": 10},
           "criteria": {"minimum_requests": 100, "maximum_sample_p95_ms": 200, "maximum_error_rate_percent": 1, "evidence_max_age_seconds": 86400},
           "claim_outputs": {"ready": "passes", "challenged": "fails"}}
    requirements = []

    def requirement(name, value, profile="api_load/1"):
        case = {"name": name, "profile": profile, "claims": claims, "requirements": value}
        try:
            validate_requirements(profile, tuple(claims), value)
            case["error"] = None
        except (ValueError, TypeError) as error:
            case["error"] = str(error)
        requirements.append(case)

    requirement("generic", generic, "registered_claims/1")
    requirement("api", api)
    requirement("unknown_profile", generic, "other/1")
    for name, path, value in [
        ("unknown_requirement", ("now",), "2026-01-01T00:00:00Z"),
        ("context_array", ("context",), []),
        ("workload_missing", ("workload",), {}),
        ("workload_whitespace", ("workload", "run_id"), " run-1"),
        ("clients_boolean", ("workload", "concurrent_clients"), True),
        ("clients_float", ("workload", "concurrent_clients"), 10.0),
        ("clients_zero", ("workload", "concurrent_clients"), 0),
        ("clients_negative", ("workload", "concurrent_clients"), -1),
        ("clients_big_integer", ("workload", "concurrent_clients"), 10**1000),
        ("context_extra", ("context", "extra"), True),
        ("criteria_missing", ("criteria",), {}),
        ("criteria_boolean", ("criteria", "maximum_sample_p95_ms"), True),
        ("criteria_string", ("criteria", "maximum_error_rate_percent"), "1"),
        ("criteria_negative", ("criteria", "maximum_sample_p95_ms"), -1),
        ("criteria_huge", ("criteria", "minimum_requests"), 10**1000),
        ("criteria_float_count", ("criteria", "minimum_requests"), 100.0),
        ("criteria_float_limit", ("criteria", "maximum_sample_p95_ms"), 200.0),
        ("criteria_changed", ("criteria", "maximum_sample_p95_ms"), 201),
        ("freshness_changed", ("criteria", "evidence_max_age_seconds"), 42),
        ("outputs_missing", ("claim_outputs",), {"ready": "passes"}),
        ("output_wrong", ("claim_outputs", "ready"), "unknown"),
        ("output_array", ("claim_outputs", "ready"), []),
    ]:
        changed = deepcopy(api)
        target = changed
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        requirement(name, changed)

    task = TaskSpec.from_dict({"task_id": "bounded-review", "question": "What do these claims support?", "entry_id": "sample", "claims": claims, "profile": "registered_claims/1", "requirements": generic})
    anchor = {"schema": TASK_SCHEMA, "task": task.to_dict(), "source_digest": "a" * 64,
              "method_registry_fingerprint": "b" * 64, "registered_context": generic["context"], "pinned_at": "2026-09-23T12:00:00Z"}
    anchor["task_digest"] = canonical_digest(anchor)
    rows = {claim: {"assessment_id": "assessment-" + claim, "status": status, "assessed_at": "2026-09-23T12:00:01Z", "collected_count": 1, "reused_count": 0} for claim, status in zip(claims, ("supported", "contested"))}
    finding = make_finding(anchor, {"ok": True, "code": None, "detail": "Matched"}, rows)
    answer = {"task_digest": anchor["task_digest"], "finding_id": finding["finding_id"],
              "claims": {claim: {key: row[key] for key in ("assessment_id", "status", "assessed_at")} for claim, row in rows.items()},
              "scope": {"context": generic["context"], "requirements": generic}, "conclusion": "assessed", "qualification": "conditional_on_registered_argument"}
    answers = []

    def authored(name, value, selected=finding):
        answers.append({"name": name, "anchor": anchor, "finding": selected, "answer": value, "diagnostics": validate_answer(anchor, selected, value)})

    authored("matching", answer)
    authored("not_object", [])
    changed = deepcopy(answer); changed.pop("qualification"); authored("missing_field", changed)
    for name, path, value in [
        ("task_identity", ("task_digest",), "c" * 64),
        ("finding_identity", ("finding_id",), "finding_" + "d" * 64),
        ("qualification", ("qualification",), "proved"),
        ("context_type", ("scope", "context", "count"), True),
        ("requirements_omitted", ("scope", "requirements"), {}),
        ("claims_missing", ("claims",), {}),
        ("claim_array", ("claims", "ready"), []),
        ("claim_status", ("claims", "ready", "status"), "unsupported"),
        ("claim_identity", ("claims", "challenged", "assessment_id"), "replacement"),
        ("claim_time", ("claims", "challenged", "assessed_at"), "2026-09-23T12:00:02Z"),
        ("wrong_conclusion", ("conclusion",), "unresolved"),
    ]:
        changed = deepcopy(answer); target = changed
        for key in path[:-1]: target = target[key]
        target[path[-1]] = value; authored(name, changed)
    for code, conclusion in [("source_changed", "out_of_scope"), ("evidence_unavailable", "unresolved"), ("effect_outcome_unknown", "unresolved")]:
        selected = make_finding(anchor, {"ok": False, "code": code, "detail": "Unavailable"})
        changed = deepcopy(answer); changed["finding_id"] = selected["finding_id"]; changed["conclusion"] = conclusion
        changed["claims"] = {claim: {"assessment_id": None, "status": None, "assessed_at": None} for claim in claims}
        authored(code, changed, selected)
        changed = deepcopy(changed); changed["claims"]["ready"]["status"] = "unsupported"
        authored(code + "_fabricated_status", changed, selected)
    result = {"schema": "eal-coordination-profile-reference/1", "source_revision": REVISION,
              "source_sha256": hashlib.sha256((reference / "src/eal/coordination/profiles.py").read_bytes()).hexdigest(),
              "requirements": requirements, "answers": answers}
    output = json.dumps(result, ensure_ascii=False, allow_nan=False, indent=2) + "\n"
    path = ROOT / "fixtures/coordination/profile-reference.json"
    if args.check:
        if path.read_text() != output: raise SystemExit("Coordination profile reference drift")
    else:
        path.parent.mkdir(parents=True, exist_ok=True); path.write_text(output)
    print(json.dumps({"requirements": len(requirements), "answers": len(answers)}))


if __name__ == "__main__":
    main()
