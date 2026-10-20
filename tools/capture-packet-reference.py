#!/usr/bin/env python3
"""Development-only packet and model-context witnesses from the unchanged source lock."""
from __future__ import annotations
import argparse
import ast
from copy import deepcopy
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]


def capture(reference: Path):
    if subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip() != REVISION:
        raise ValueError("Reference revision differs from the source lock")
    if subprocess.check_output(["git", "-C", str(reference), "status", "--porcelain", "--", "src/eal", "grammar"], text=True):
        raise ValueError("Reference production sources must be unchanged")
    sys.path.insert(0, str(reference / "src"))
    from eal.packets import AssessmentPacketBuilder, PacketLimits
    from eal.model_context import ModelContextBuilder, CONTEXT_INSTRUCTION
    from eal.evaluator import canonical_digest
    tree = ast.parse((reference / "tests/test_packets.py").read_text())
    function = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "_assessment")
    namespace = {}
    exec(compile(ast.Module(body=[function], type_ignores=[]), "<source-test-fixture>", "exec"), namespace)
    base = namespace["_assessment"]()
    base["arguments"]["experiment"]["reasoning_result"]["details"]["samples"] = [1, 2, 3]
    base["assessed_at"] = "2026-09-28T10:00:00Z"
    cases = []

    def case(name, assessment=None, claims=("pressure",), limits=None, collection=None):
        assessment = deepcopy(base if assessment is None else assessment)
        record = {"name": name, "assessment": assessment, "claims": claims, "limits": limits or {}, "collection": collection}
        try:
            packet = AssessmentPacketBuilder(limits=PacketLimits(**(limits or {}))).build(assessment, claims=claims, collection=collection)
            record["packet"] = packet
            claim = claims[0] if claims else next(iter(packet["claims"]), None)
            if claim:
                registered = {"packet": packet, "assessed_at": assessment.get("assessed_at"), "assessment_id": assessment["assessment_id"], "claim": claim, "status": assessment["claims"][claim]["status"]}
                record["model_context"] = ModelContextBuilder().build(registered)
        except (ValueError, TypeError, OverflowError, KeyError, AttributeError) as error:
            record["error"] = str(error)
        cases.append(record)

    case("support-challenge-binding")
    case("automatic-selection", claims=None)
    for label, value in [("false", False), ("true", True), ("numeric", 1), ("null", None), ("text", "true")]:
        a = deepcopy(base); a["claims"]["pressure"]["prose_verified"] = value; case("prose-" + label, a)
    a = deepcopy(base); a["claims"]["pressure"]["statement"] = "€" * 9; a["claims"]["other"]["statement"] = "é" * 5
    a["arguments"]["experiment"]["dependencies"]["premises"] = ["other"]
    case("unicode-statement-prefix", a, limits={"max_statement_bytes": 7})
    a = deepcopy(base); del a["claims"]["pressure"]["statement"]; case("missing-statement", a)
    a = deepcopy(base); a["objections"]["sensor_fault"]["premises"] = ["other"]
    a["arguments"]["other_argument"]["dependencies"]["evidence"] = ["other_probe"]
    a["evidence"]["other_probe"] = {"status": "available", "run_id": "original-reading"}
    case("objection-premise-alternative-route", a)
    a = deepcopy(base); a["arguments"]["experiment"]["dependencies"]["premises"] = ["other"]
    a["arguments"]["alternative"] = deepcopy(a["arguments"]["other_argument"])
    a["arguments"]["alternative"]["dependencies"]["premises"] = ["pressure"]
    case("alternative-routes-and-cycle", a)
    a = deepcopy(base); a["reasoning"] = {"causal_reason": {"backing": ["backing_probe"]}}
    a["evidence"]["backing_probe"] = {"status": "unavailable", "availability_issues": ["predicate_not_met"]}
    case("reasoning-backing-negative-evidence", a)
    for label, issues in [("unknown", ["SECRET", "predicate_not_met"]), ("duplicate", ["stale_observation"] * 4), ("all", ["out_of_scope", "missing_observation", "invalid_observation", "tool_error", "stale_observation", "predicate_not_met"])]:
        a = deepcopy(base); a["evidence"]["trial"].update(status="unavailable", availability_issues=issues)
        case("issues-" + label, a, limits={"max_availability_issues": 1})
    a = deepcopy(base); a["evidence"]["trial"]["predicate_failures"] = [{"path": f"field{i}", "issue": "missing_field", "value": "SECRET"} for i in range(10)] + [{"path": "secret path", "issue": "not_met"}, {"path": "x", "issue": "SECRET"}]
    case("predicate-diagnostic-omissions", a)
    a = deepcopy(base); a["assumptions"]["calibration"].update(time_status="expired", valid_from="2026-09-28T11:00:00+01:00", valid_until="2026-09-28T11:00:00.123456+01:00")
    case("assumption-time-utc", a)
    for label, method in [("unknown", "extension/private/1"), ("wrong-version", "causal/2"), ("not-run", None)]:
        a = deepcopy(base); r = a["arguments"]["experiment"]["reasoning_result"]; r["method"] = method
        if method is None: r["status"] = "not_run"
        case("method-" + label, a)
    a = deepcopy(base); r = a["arguments"]["experiment"]["reasoning_result"]
    r.update(method="structured/1", predicates=[{"holds": False, "reason": "SECRET"}] * 19)
    r["details"].update(authored=False, mechanically_proved=False); r["binding"]["actual"] = 0
    case("false-zero-and-predicate-bound", a)
    output_values = {"entailed": False, "consistent_premises": True, "valuations": 8,
                     "satisfying_premise_valuations": 0, "estimate": 0, "lower": -1.0, "upper": 1.0,
                     "best_posterior": 0.75, "posterior": {"PRIVATE_INPUT_LABEL": 0.75},
                     "standard_error": 0.0, "treatment_mean": 1, "control_mean": 1,
                     "factual": 1, "counterfactual": 0, "difference": -1,
                     "match_fraction": 0, "complete": False, "holds": False, "coverage": True,
                     "authored": True, "mechanically_proved": False, "sample_size": 10,
                     "successes": 0, "confidence": 0.95, "treatment_size": 5, "control_size": 5,
                     "feature_count": 5, "matched": 0, "private": "SECRET"}
    for method in ["structured", "deductive", "inductive", "abductive", "causal", "counterfactual", "analogical", "temporal"]:
        a = deepcopy(base); a["arguments"]["experiment"]["reasoning_result"].update(method=method + "/1", details=output_values)
        case("builtin-outputs-" + method, a)
    a = deepcopy(base)
    a["arguments"]["experiment"]["dependencies"]["evidence"] = [f"reading{i:03}" for i in range(60)]
    for i in range(60): a["evidence"][f"reading{i:03}"] = {"status": "available", "run_id": f"original-{i}"}
    case("observation-and-evidence-count-omissions", a)
    a = deepcopy(base); a["arguments"]["experiment"]["dependencies"]["premises"] = ["other"]
    case("all-projection-count-omissions", a, limits={"max_arguments": 1, "max_objections": 1, "max_evidence": 1, "max_claims": 1, "max_outputs": 1})
    a = {"valid": True, "assessment_id": "stored", "assessed_at": base["assessed_at"], "claims": {}, "arguments": {}}
    for i in range(140):
        name = f"claim{i:03}"; arg = f"argument{i:03}"
        a["claims"][name] = {"status": "supported", "statement": "A bounded route claim."}
        a["arguments"][arg] = {"status": "supported", "conclusion": name, "dependencies": {"premises": [f"claim{i + 1:03}"] if i < 139 else []}, "reasoning_result": {"status": "supported", "method": "structured/1", "details": {"authored": True}}}
    case("bounded-premise-and-observation-traversal", a, claims=["claim000"], limits={"max_bytes": 100000})
    qualified = deepcopy(base)
    names = {name: "suite.case." + name for group in ["claims", "arguments", "assumptions", "objections", "evidence"] for name in qualified[group]}
    names.update(causal_reason="suite.case.causal_reason", structured_reason="suite.case.structured_reason")
    def qualify(value):
        if isinstance(value, dict): return {names.get(k, k): qualify(v) for k, v in value.items()}
        if isinstance(value, list): return [qualify(v) for v in value]
        if isinstance(value, str): return "objection:" + names.get(value[10:], value[10:]) if value.startswith("objection:") else names.get(value, value)
        return value
    case("qualified-dependencies", qualify(qualified), claims=[names["pressure"]])
    context = {"site": "bench"}
    collection = {"collection_id": "collection-1", "source_digest": "a" * 64, "context": context, "records": {"trial": {"run_id": "original-reading", "status": "ok", "collected_at": "2026-09-28T09:59:00Z", "value": "SECRET"}}}
    a = deepcopy(base); a.update(collection_id="collection-1", context_fingerprint=canonical_digest(context)); a["evidence"]["trial"].update(run_id="original-reading", availability_issues=["stale_observation"])
    case("original-observation-age", a, collection=collection)
    for label, collected, status in [("fraction", "2026-09-28T09:59:59.123456Z", "ok"), ("future", "2026-09-28T10:00:01Z", "ok"), ("bad-time", "yesterday", "ok"), ("failed-attempt", "2026-09-28T09:59:00Z", "error")]:
        c = deepcopy(collection); c["records"]["trial"].update(collected_at=collected, status=status); case("age-" + label, a, collection=c)
    for key, value in [("collection_id", "other"), ("source_digest", "b" * 64), ("context", {"site": "elsewhere"}), ("records", {"trial": "bad"})]:
        c = deepcopy(collection); c[key] = value; case("collection-mismatch-" + key, a, collection=c)
    c = deepcopy(collection); c["records"]["trial"]["run_id"] = "forged"; case("observation-id-mismatch", a, collection=c)
    for field, value in [("valid", False), ("complete", False), ("assessment_id", "contains space"), ("claims", []), ("arguments", []), ("evidence", []), ("collection_id", "invalid/id")]:
        a = deepcopy(base); a[field] = value; case("bad-assessment-" + field, a)
    for name in ["", "suite..claim", "suite.café", "a" * 129, "suite.claim;exec"]:
        a = {"valid": True, "assessment_id": "stored", "claims": {name: {"status": "unsupported", "statement": "Bounded synthetic claim."}}}; case("bad-name-" + repr(name), a, claims=None)
    for value in [[], ["pressure", "pressure"], ["absent"]]: case("bad-selected-" + repr(value), claims=value)
    for field, value in [("max_bytes", 4095), ("max_claims", 0), ("max_outputs", 129), ("max_statement_bytes", 4097)]: case("bad-limit-" + field, limits={field: value})
    for field, value in [("status", "unknown"), ("availability_issues", {}), ("predicate_failures", {})]:
        a = deepcopy(base); a["evidence"]["trial"][field] = value; case("bad-evidence-" + field, a)
    a = deepcopy(base)
    for i in range(15):
        name = f"additional_claim_{i:02}"; arg = f"additional_argument_{i:02}"
        a["claims"][name] = {"status": "supported", "statement": "Long retained statement. " * 40}
        a["arguments"][arg] = {"status": "supported", "conclusion": name, "dependencies": {}, "reasoning_result": {"status": "supported", "method": "structured/1", "details": {"authored": True}}}
    case("packet-byte-fallback", a, claims=list(a["claims"])[:16], limits={"max_bytes": 4096})
    return {"schema": "eal-applications-packets-reference/1", "source_revision": REVISION,
            "provenance": "Synthetic packet and context projections; source production code unchanged", "context_instruction": CONTEXT_INSTRUCTION, "cases": cases}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path); parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    output = ROOT / "fixtures/applications/packets-reference.json"
    encoded = json.dumps(capture(args.reference.resolve()), ensure_ascii=False, indent=2, allow_nan=False) + "\n"
    if args.check:
        if not output.exists() or output.read_text() != encoded: raise SystemExit("Packet/context oracle drift")
    else:
        output.parent.mkdir(parents=True, exist_ok=True); output.write_text(encoded)


if __name__ == "__main__": main()
