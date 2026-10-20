#!/usr/bin/env python3
"""Capture typed EAL/ASPIC witnesses from the unchanged pinned Python reference.

Development only: normal Rust tests read the retained JSON. Test functions are
extracted through Python's AST without importing test_aspic's MCP dependencies.
"""
from __future__ import annotations

import argparse
import ast
from copy import deepcopy
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]


def retained_functions(path, names, namespace):
    nodes = [node for node in ast.parse(path.read_text()).body
             if isinstance(node, ast.FunctionDef) and node.name in names]
    if {node.name for node in nodes} != set(names):
        raise ValueError("Pinned test helpers are missing")
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(path), "exec"), namespace)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = args.reference.resolve()
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=reference, text=True).strip()
    changed = subprocess.check_output(["git", "status", "--porcelain", "--", "src/eal", "grammar", "tests"], cwd=reference, text=True)
    if revision != REVISION or changed:
        parser.error("reference source and tests must be unchanged at the pinned revision")
    sys.path.insert(0, str(reference / "src"))
    from eal.aspic import METHOD, aspic_registry
    from eal.evaluator import canonical_digest, environment_fingerprint, evaluate
    from eal.limits import ExecutionLimits
    from eal.parser import parse
    from eal.semantics import validate

    now = "2026-09-23T12:00:00Z"
    context = {"site": "bench"}
    helpers = {"json": json, "METHOD": METHOD, "NOW": now, "CONTEXT": context,
               "canonical_digest": canonical_digest, "environment_fingerprint": environment_fingerprint}
    retained_functions(reference / "tests/test_aspic.py", {"theory", "source"}, helpers)
    retained_functions(reference / "tests/test_evaluator.py", {"record"}, helpers)
    data = helpers["theory"]()
    del data["premises"][-1]
    del data["rules"][-1]
    source = helpers["source"](data)
    cases = []
    for name in ("typed-theory-accepted", "typed-query-mismatch", "typed-scope-mismatch", "typed-stale-observation", "typed-construction-incomplete"):
        limits = {"formal_arguments": 1} if name == "typed-construction-incomplete" else {}
        program = parse(source, limits=ExecutionLimits(**limits))
        envelope = {"schema": "EAL/typed-input/1", "method": METHOD, "subject": "orders-api",
                    "quantity": "proposition", "unit": "1", "scope": "run-001",
                    "valid_from": "2026-09-23T10:00:00Z", "valid_until": "2026-09-23T11:00:00Z",
                    "payload": {"theory": deepcopy(data)}}
        if name == "typed-query-mismatch":
            envelope["payload"]["theory"]["goal"] = "other_run"
        if name == "typed-scope-mismatch":
            envelope["scope"] = "run-002"
        collected_at = "2026-09-23T11:58:59Z" if name == "typed-stale-observation" else now
        records = {"formal": helpers["record"](program, "formal", envelope, collected_at),
                   "report_record": helpers["record"](program, "report_record", {"checked": True}),
                   "latency_record": helpers["record"](program, "latency_record", {"within_limit": True}),
                   "gap_record": helpers["record"](program, "gap_record", {"gap_found": True})}
        diagnostics = validate(program, registry=aspic_registry())
        if diagnostics:
            raise ValueError(f"Pinned typed ASPIC source failed validation: {diagnostics}")
        outcome = evaluate(program, records, now=now, context=context, registry=aspic_registry())
        cases.append({"name": name, "source": source, "source_sha256": hashlib.sha256(source.encode()).hexdigest(),
                      "limits": limits, "context": context, "now": now, "records": records,
                      "diagnostics": [asdict(item) for item in diagnostics], "evaluation": outcome})
    result = {"schema": "eal-aspic-typed-method-reference/1", "source_revision": REVISION,
              "provenance": "Synthetic typed-method witnesses derived from pinned tests/test_aspic.py and record helper from tests/test_evaluator.py; not experimental measurements",
              "comparison": {"ordinary": "All evaluation fields; only the top-level registry fingerprint and known reasoning_result[/binding]/method_contract backend identity and execution fields are normalized",
                             "budget": "Structured completeness, method status, empty details, binding/provenance and the exact formal budget message; process-specific diagnostic prefixes remain retained in this fixture"},
              "cases": cases}
    output = json.dumps(result, ensure_ascii=False, allow_nan=False, indent=2) + "\n"
    destination = ROOT / "fixtures/aspic/method-reference.json"
    if args.check:
        if not destination.exists() or destination.read_text() != output:
            raise SystemExit("Typed ASPIC method reference differs")
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(output)
    print(json.dumps({"cases": len(cases), "statuses": {case["name"]: case["evaluation"]["claims"]["run_passes"]["status"] for case in cases}}))


if __name__ == "__main__":
    main()
