#!/usr/bin/env python3
"""Synthetic records for executable method conformance; no real scientific study."""
import datetime
import json
import sys

request = json.load(sys.stdin)
scenario = request["input"]["scenario"]
if scenario == "error":
    raise RuntimeError("Synthetic measurement instrument failed")
reviewed_at = datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z")
value = {
    "sample": {"successes": 90, "trials": 100},
    "hypotheses": {"observed": "alarm", "candidates": [
        {"name": "fault", "prior": 0.2, "likelihood": 0.8},
        {"name": "normal", "prior": 0.8, "likelihood": 0.1}]},
    "experiment": {"treatment": [10, 12], "control": [2, 4]},
    "model": {"variables": {"x": {"intercept": 1, "coefficients": {}, "noise": 0},
                              "y": {"intercept": 3, "coefficients": {"x": 2}, "noise": 0}},
              "intervention": {"variable": "x", "value": 4}, "outcome": "y"},
    "analogy": {"relevant_features": ["protocol", "maximum_load"],
                "source": {"protocol": "v2", "maximum_load": 80},
                "target": {"protocol": "v2", "maximum_load": 80}},
    "review": {"reviewer": "synthetic-reviewer", "criterion": "choose feasible minimum-cost option",
               "target": "option-B", "scope": "synthetic-options-v1", "verdict": "supported",
               "reasons": ["A capacity 50 violates required 80; B capacity 100 satisfies it at cost 20."],
               "reviewed_at": reviewed_at,
               "provenance": {"options": [{"name": "A", "capacity": 50, "cost": 10},
                           {"name": "B", "capacity": 100, "cost": 20}]}},
    "applicability": {"scope": "synthetic-methods-v1", "verdict": "supported"},
    "challenge": {"active": False},
}
if scenario == "negative":
    value["sample"]["successes"] = 10
    value["experiment"]["treatment"] = [0, 2]
    value["analogy"]["target"]["maximum_load"] = 120
    value["review"]["verdict"] = "rejected"
elif scenario == "wrong-review-scope":
    value["review"]["scope"] = "different-options"
elif scenario == "inconclusive":
    value["review"]["verdict"] = "inconclusive"
elif scenario == "wrong-reviewer":
    value["review"]["reviewer"] = "other-reviewer"
elif scenario == "challenge":
    value["challenge"]["active"] = True
elif scenario == "old-nested-review":
    value["review"]["reviewed_at"] = "2000-01-01T00:00:00Z"
elif scenario == "malformed-review-reasons":
    value["review"]["reasons"] = [None]
elif scenario == "malformed-review-time":
    value["review"]["reviewed_at"] = "not-a-time"
elif scenario == "query-conflict":
    value["sample"]["confidence"] = 0.5
observed_at = "2000-01-01T00:00:00Z" if scenario == "stale" else reviewed_at
print(json.dumps({"value": value, "observed_at": observed_at}))
