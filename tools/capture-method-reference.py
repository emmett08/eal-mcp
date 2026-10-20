#!/usr/bin/env python3
"""Capture synthetic method witnesses from the unchanged pinned Python implementation.

Development only. Rust tests read the retained JSON without running Python.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]


def cases():
    result = []

    def add(mode, name, payload=None, *, evidence=None, premises=None):
        kinds = {"deductive": "logical_case", "inductive": "sample", "abductive": "hypotheses", "causal": "experiment", "counterfactual": "causal_model", "analogical": "analogy", "temporal": "trace"}
        result.append({"name": name, "method": f"{mode}/1", "evidence": evidence if evidence is not None else ([] if mode == "structured" else [{"id": "case", "kind": kinds[mode], "value": payload}]), "premises": premises or []})

    add("structured", "structured-empty")
    for status in ("supported", "contested", "unsupported"):
        add("structured", f"structured-premise-{status}", premises=[{"status": status}])
    add("structured", "structured-evidence", evidence=[{"id": "e", "kind": "observation", "value": {"x": 1}}])
    add("structured", "structured-malformed-evidence", evidence=[{"id": "e", "kind": "observation"}])
    add("structured", "structured-duplicate-evidence", evidence=[{"id": "e", "kind": "x", "value": 1}] * 2)
    add("deductive", "modus-ponens", {"premises": ["p", {"implies": ["p", "q"]}], "conclusion": "q"})
    add("deductive", "affirm-consequent", {"premises": ["q", {"implies": ["p", "q"]}], "conclusion": "p"})
    add("deductive", "inconsistent", {"premises": ["p", {"not": "p"}], "conclusion": "q"})
    add("deductive", "tautology", {"premises": [], "conclusion": {"or": ["p", {"not": "p"}]}})
    for i, conclusion in enumerate((True, {"implies": ["p"]}, {"and": []}, {"xor": ["p", "q"]})):
        add("deductive", f"bad-formula-{i}", {"premises": [], "conclusion": conclusion})
    add("deductive", "too-many-atoms", {"premises": [f"p{i}" for i in range(13)], "conclusion": "p0"})
    for k, n, confidence in ((20, 25, .95), (0, 10, .95), (10, 10, .95), (1, 2, .001), (1, 2, .999999), (500_000_000, 1_000_000_000, .9)):
        add("inductive", f"wilson-{k}-{n}-{confidence}", {"successes": k, "trials": n, "confidence": confidence})
    for i, payload in enumerate(({"successes": True, "trials": 2, "confidence": .95}, {"successes": 0, "trials": 0, "confidence": .95}, {"successes": 3, "trials": 2, "confidence": .95}, {"successes": 1, "trials": 2.0, "confidence": .95}, {"successes": 1, "trials": 2, "confidence": 1})):
        add("inductive", f"bad-binomial-{i}", payload)
    for label, candidates in (("tie", [("fault", .1, .9), ("healthy", .9, .1)]), ("best", [("fault", .2, .8), ("healthy", .8, .1)]), ("underflow", [("a", 1e-200, 1e-200), ("b", 1., 0.)]), ("impossible", [("a", .5, 0), ("b", .5, 0)]), ("bad-prior", [("a", .4, .5), ("b", .5, .1)]), ("duplicate", [("a", .5, .5), ("a", .5, .1)])):
        add("abductive", f"posterior-{label}", {"observed": "alarm", "candidates": [{"name": name, "prior": p, "likelihood": l} for name, p, l in candidates]})
    for label, treatment, control in (("ordinary", [3, 5, 7], [1, 2, 3]), ("large-neighbours", [2**60 + 2] * 2, [2**60 + 1] * 2), ("tiny-se", [0., 1e-200], [0., 0.]), ("fractional", [1, 2], [0, 0]), ("mean-underflow", [5e-324, 0.], [0., 0.])):
        add("causal", f"contrast-{label}", {"assignment": "randomised", "treatment": treatment, "control": control})
    add("causal", "bad-assignment", {"assignment": "observational", "treatment": [3, 5], "control": [1, 2]})
    scm = {"variables": {"x": {"intercept": 1, "coefficients": {}, "noise": 2}, "y": {"intercept": 4, "coefficients": {"x": 2}, "noise": 1}, "z": {"intercept": 0, "coefficients": {"y": 3}, "noise": -1}}, "intervention": {"variable": "x", "value": 5}, "outcome": "z"}
    add("counterfactual", "scm-baseline", scm)
    for label in ("zero-cycle", "unknown-parent", "large-integer", "mixed-float"):
        value = json.loads(json.dumps(scm))
        if label == "zero-cycle":
            value["variables"]["x"]["coefficients"]["z"] = 0
        elif label == "unknown-parent":
            value["variables"]["x"]["coefficients"]["missing"] = .5
        elif label == "large-integer":
            value["variables"]["x"]["intercept"] = 2**80
            value["intervention"]["value"] = 2**80 + 1
        else:
            value["variables"]["x"]["intercept"] = 1.5
        add("counterfactual", f"scm-{label}", value)
    for label, source, target in (("matching", {"x": 1}, {"x": 1.}), ("typed", {"x": True}, {"x": 1}), ("missing", {"x": 1}, {}), ("nonscalar", {"x": [1]}, {"x": [1]}), ("integer-float-precision", {"x": 2**60 + 1}, {"x": float(2**60 + 1)})):
        add("analogical", f"features-{label}", {"relevant_features": ["x"], "source": source, "target": target})
    for label, events, end in (("complete", [(0, 1), (1, 2), (2, 3)], 2), ("negative", [(0, 1), (1, 9), (2, 3)], 2), ("gap", [(0, 1), (2, 3)], 2), ("missing-end", [(0, 1), (1, 2)], 2), ("singleton", [(0, 1)], 0), ("bad-order", [(1, 1), (0, 1)], 1), ("integer-time-precision", [(2**60, 1), (2**60 + 1, 1)], 2**60 + 1)):
        start = events[0][0] if label == "integer-time-precision" else 0
        add("temporal", f"trace-{label}", {"start": start, "end": end, "max_gap": 1, "events": [{"time": t, "value": v} for t, v in events], "property": {"operator": "lt", "value": 5}, "semantics": "sampled"})
    for method in ("inference", "a'b/1", "unknown\n/1"):
        result.append({"name": f"unknown-method-{len(result)}", "method": method, "evidence": [], "premises": []})
    add("deductive", "quoted-operator", {"premises": [], "conclusion": {"bad'operator": ["p", "q"]}})

    def optional(identifier, kind, name, payload):
        result.append({"name": name, "method": identifier, "optional": True, "evidence": [{"id": "case", "kind": kind, "value": payload}], "premises": []})

    for label, count, p95, errors in (("boundary", 100, 200, 1), ("latency-fails", 100, 201, 1), ("small-sample", 99, 100, 0), ("empty", 0, None, None), ("unknown-count", None, None, None), ("missing-latency", 100, None, 0), ("empty-with-statistic", 0, 0, None)):
        optional("engineering/api-load-criteria/1", "api_report", f"api-{label}", {"request_count": count, "p95_ms": p95, "error_rate_percent": errors})
    for label, edges, forbidden, horizon in (("sorted-shortest", [[0, 2], [0, 1], [2, 3], [1, 3]], [3], 2), ("short-horizon", [[0, 1], [1, 2]], [2], 1), ("initial-forbidden", [], [0], 0), ("cycle", [[0, 1], [1, 0]], [3], 8), ("duplicate-edge", [[0, 1], [0, 1]], [3], 2), ("outside-endpoint", [[0, 4]], [3], 2)):
        optional("engineering/reachability/1", "finite_graph", f"reachability-{label}", {"node_count": 4, "edges": edges, "start": 0, "forbidden": forbidden, "horizon": horizon})
    for label, origin, samples in (("ordinary", 0, [3, 4]), ("zero", 2, [2, 2]), ("tiny", 0, [1e-200, 1e-200]), ("large-neighbour", 2**60, [2**60 + 1]), ("large-scale", 0, [1e100, -1e100]), ("maximum-deviation", -1e100, [1e100])):
        optional("engineering/rms/1", "measurement_series", f"rms-{label}", {"origin": origin, "samples": samples})
    for label in ("observed-violation", "violation-despite-gap", "no-observed-violation", "calibrated-non-detection", "missing-calibration", "non-detection-gap", "non-detection-violation", "zero-sensitivity", "detector-limit-too-high", "calibration-limit-too-high", "calibration-sensitivity-too-low", "inclusive-boundary", "strict-boundary", "duplicate-time"):
        payload = {"mode": "counterexample", "start": 0, "end": 2, "max_gap": 1, "property": {"operator": "lt", "value": 5}, "semantics": "sampled", "detector_contract": {"maximum_detection_limit": 1, "minimum_sensitivity": .9}, "events": [{"time": 0, "value": 1}, {"time": 1, "value": 9}, {"time": 2, "value": 3}]}
        if label not in ("observed-violation", "violation-despite-gap", "no-observed-violation", "strict-boundary", "duplicate-time"):
            payload["mode"] = "non_detection"
            payload["events"][1]["value"] = 2
            payload["calibration"] = {"detection_limit": 1, "sensitivity_lower_bound": .95}
        if label == "violation-despite-gap":
            payload["events"] = [{"time": 0, "value": 1}, {"time": 2, "value": 9}]
        elif label == "no-observed-violation":
            payload["events"][1]["value"] = 2
        elif label == "missing-calibration":
            del payload["calibration"]
        elif label == "non-detection-gap":
            del payload["events"][1]
        elif label == "non-detection-violation":
            payload["events"][1]["value"] = 9
        elif label == "zero-sensitivity":
            payload["detector_contract"]["minimum_sensitivity"] = 0
        elif label == "detector-limit-too-high":
            payload["detector_contract"]["maximum_detection_limit"] = 6
        elif label == "calibration-limit-too-high":
            payload["calibration"]["detection_limit"] = 2
        elif label == "calibration-sensitivity-too-low":
            payload["calibration"]["sensitivity_lower_bound"] = .5
        elif label in ("inclusive-boundary", "strict-boundary"):
            payload["events"][1]["value"] = 5
            payload["property"]["operator"] = "le" if label == "inclusive-boundary" else "lt"
        elif label == "duplicate-time":
            payload["events"][1]["time"] = 0
        optional("engineering/sampled-negative/1", "sampled_negative_trace", f"sampled-{label}", payload)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = args.reference.resolve()
    actual = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=reference, text=True).strip()
    if actual != REVISION or subprocess.check_output(["git", "status", "--porcelain", "--", "src/eal", "grammar"], cwd=reference, text=True).strip():
        parser.error("reference checkout must be unchanged at the pinned revision")
    sys.path.insert(0, str(reference / "src"))
    from eal.modes import assess_mode
    from eal.api_load_methods import assess_api_limits
    from eal.reachability import finite_reachability
    from eal.extensions import root_mean_square
    from eal.sampled_negative import sampled_negative
    callbacks = {"engineering/api-load-criteria/1": assess_api_limits, "engineering/reachability/1": finite_reachability, "engineering/rms/1": root_mean_square, "engineering/sampled-negative/1": sampled_negative}
    output = cases()
    for case in output:
        if case.get("optional"):
            try:
                case["expected_callback"] = {"details": callbacks[case["method"]](case["evidence"][0]["value"])}
            except ValueError as error:
                case["expected_callback"] = {"error": str(error)}
        else:
            case["expected"] = assess_mode(case["method"], case["evidence"], case["premises"])
    encoded = json.dumps({"schema": "eal-rust-method-reference/1", "source_revision": REVISION, "provenance": "Synthetic mathematical witnesses; not experimental measurements", "cases": output}, ensure_ascii=False, indent=2, allow_nan=False) + "\n"
    destination = ROOT / "fixtures" / "methods" / "reference.json"
    if args.check:
        if not destination.exists() or destination.read_text() != encoded:
            raise SystemExit("method reference differs")
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(encoded)


if __name__ == "__main__":
    main()
