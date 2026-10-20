#!/usr/bin/env python3
"""Retain checked ASPIC export witnesses from the pinned, unmodified Python source.

Development capture only; native tests read JSON and never start Python workers.
Requires CPython 3.12.14, antlr4-python3-runtime 4.13.2 and jsonschema 4.23.0.
"""
from __future__ import annotations

import argparse
import ast
from copy import deepcopy
import hashlib
import importlib.metadata
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]
NOW = "2026-09-23T12:00:00Z"
CONTEXT = {"site": "bench"}


def literal(path, name):
    for node in ast.parse(path.read_text()).body:
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == name for t in node.targets):
            return ast.literal_eval(node.value)
    raise ValueError(name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = args.reference.resolve()
    if sys.implementation.name != "cpython" or sys.version_info[:3] != (3, 12, 14):
        raise SystemExit("Reference capture requires CPython 3.12.14")
    for package, version in (("antlr4-python3-runtime", "4.13.2"), ("jsonschema", "4.23.0")):
        if importlib.metadata.version(package) != version:
            raise SystemExit(f"Reference capture requires {package} {version}")
    if subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip() != REVISION:
        raise SystemExit("Reference checkout must be pinned at " + REVISION)
    if subprocess.check_output(["git", "-C", str(reference), "status", "--porcelain", "--", "src/eal", "grammar", "tests", "docs/aspic-view.schema.json"], text=True):
        raise SystemExit("Reference source and export schema must be unchanged")
    sys.path.insert(0, str(reference / "src"))
    from eal.aspic import solve_aspic
    from eal.aspic_compiler import compile_eal_aspic
    from eal.aspic_export import export_aspic_view, VIEW_SCHEMA
    from eal.evaluator import canonical_digest, environment_fingerprint
    from eal.limits import ExecutionLimits
    from eal.parser import parse

    retained_schema = ROOT / "contracts/schemas/aspic-view.schema.json"
    assert json.loads(retained_schema.read_text()) == VIEW_SCHEMA
    assert retained_schema.read_bytes() == (reference / "docs/aspic-view.schema.json").read_bytes()
    base = literal(reference / "tests/test_aspic_compiler.py", "BASE")

    def compiled(source=base, *, missing=(), stale=()):
        program = parse(source)
        records = {}
        for name, evidence in program.evidence.items():
            if name in missing:
                continue
            tool = program.tools[evidence.tool]
            acquisition = {"tool": tool.name, "tool_version": tool.version,
                           "input": evidence.input, "context": CONTEXT}
            value = {"ok": True}
            records[name] = {
                "evidence_id": name, "source_digest": program.source_digest,
                "tool": tool.name, "tool_version": tool.version,
                "tool_binding_digest": "0" * 64, "evidence_kind": evidence.kind,
                "environment": evidence.environment,
                "environment_fingerprint": environment_fingerprint(evidence.environment, CONTEXT),
                "input_digest": canonical_digest(evidence.input),
                "collected_at": "2026-09-23T11:58:59Z" if name in stale else NOW,
                "input": evidence.input, "context": CONTEXT,
                "acquisition_request": acquisition,
                "acquisition_request_digest": canonical_digest(acquisition),
                "request_digest": canonical_digest({"evidence_id": name,
                    "environment": evidence.environment, **acquisition}),
                "run_id": "synthetic-export-reference", "status": "ok", "value": value,
                "data_digest": canonical_digest(value),
            }
        return compile_eal_aspic(source, records, goal="run_passes", now=NOW, context=CONTEXT).to_dict()

    cases = []

    def add(name, result, error_match=None, host_limits=None):
        entry = {"name": name, "input": deepcopy(result)}
        if host_limits:
            entry["host_limits"] = host_limits
        try:
            entry["view"] = export_aspic_view(result, limits=ExecutionLimits(**(host_limits or {})))
        except (ValueError, TypeError, KeyError, AttributeError) as error:
            entry["error"] = str(error)
            entry["error_match"] = error_match or str(error).split(":", 1)[0]
        if error_match is not None and "error" not in entry:
            raise AssertionError((name, "expected an export rejection"))
        cases.append(entry)

    supplied = compiled(base + "objection challenge = [evidence gap_data] -x> argument primary_route")
    add("compiled-undercut", supplied)
    missing = compiled(base + "objection challenge = [evidence gap_data] -x> argument primary_route", missing=("primary_data",))
    add("missing-observation", missing)
    add("stale-observation", compiled(stale=("primary_data",)))
    add("structural-basis", compiled(missing=("primary_data", "probe_data", "gap_data", "calibration_data")))
    add("reviewed-directives", compiled(base + '''
rank primary_route 600 reviewed "synthetic-review/rank"
prefer primary_route over probe_route reviewed "synthetic-review/preference"
'''))
    add("reviewed-strict-rule", compiled(base + '\nstrict probe_route reviewed "synthetic-review/strict"\n'))
    add("reviewed-contrary-origins", compiled(base + '''
claim run_fails { statement "The synthetic run failed."
  environment lab
}
argument failure_route = [evidence gap_data] via authored => run_fails
contrary run_fails to run_passes reviewed "synthetic-review/incompatible-outcomes"
contrary run_passes to run_fails reviewed "synthetic-review/incompatible-outcomes"
'''))
    for name, options in [
        ("formal-default", {}),
        ("formal-preferred-credulous", {"semantics": "preferred", "query_mode": "credulous"}),
        ("formal-stable-sceptical", {"semantics": "stable", "query_mode": "sceptical"}),
        ("formal-last-link", {"preference": "last_link_rank"}),
    ]:
        theory = {"premises": [{"atom": "p", "kind": "ordinary", "rank": 5}],
                  "rules": [{"id": "derive", "kind": "defeasible", "antecedents": ["p"],
                             "consequent": "q", "name": "applicable", "rank": 5}],
                  "contraries": [], "goal": "q"}
        if options:
            theory["options"] = options
        add(name, {"theory": theory, "formal": solve_aspic({"theory": theory})})
    theory = {"premises": [{"atom": atom, "kind": "ordinary", "rank": 5} for atom in ("p", "q", "r")],
              "rules": [], "contraries": [{"attacker": a, "target": b} for a,b in (("p","q"),("q","r"),("r","p"))],
              "goal": "p", "options": {"semantics": "stable", "query_mode": "sceptical"}}
    add("formal-no-stable-extension", {"theory": theory, "formal": solve_aspic({"theory": theory})})

    def mutate(name, function, original=supplied, error_match=None):
        changed = deepcopy(original)
        function(changed)
        add(name, changed, error_match)

    mutate("supplied-prose-and-open-source-metadata", lambda r: (
        r["source_map"]["claims"]["run_passes"].update(statement='</script><img src=x onerror="bad"> é🧪'),
        r["source_map"].update(uninterpreted={"span": {"not": "a location"}}),
        r["source_map"]["arguments"]["primary_route"].update(
            extra={"user": "payload"}, source_file="synthetic/main.eal", scope_transfer={
                "source":"lab", "target":"field", "assumption":"reviewed", "review":"synthetic-review/transfer"})))
    mutate("integral-float-source-spans", lambda r: r["source_map"]["evidence"]["primary_data"]["span"].update(line=1.0))
    for issue in ("predicate_not_met", "missing_observation", "stale_observation", "tool_error", "invalid_observation", "out_of_scope"):
        mutate("supplied-typed-availability-" + issue, lambda r,v=issue: r["source_map"]["evidence"]["primary_data"].update(availability_issues=[v]), missing)
    mutate("theory-digest-mismatch", lambda r: r["theory"]["rules"].pop(), error_match="theory digest")
    for field, value in (("label","out"),("strength",499),("rank",499),("direct_subarguments",["unknown"])):
        mutate("forged-argument-" + field, lambda r,f=field,v=value: r["formal"]["arguments"][0].update({f:v}), error_match="recomputation")
    for field, value in (("argument_count",1),("grounded_accepted",False)):
        mutate("forged-" + field, lambda r,f=field,v=value: r["formal"].update({f:v}), error_match="recomputation")
    mutate("omitted-defeats", lambda r: r["formal"].update(defeats=[],defeat_count=0), error_match="recomputation")
    mutate("omitted-argument", lambda r: r["formal"]["arguments"].pop(), error_match="recomputation")
    mutate("reordered-arguments", lambda r: r["formal"]["arguments"].reverse(), error_match="recomputation")
    mutate("unknown-defeat-subargument", lambda r: r["formal"]["defeats"][0].update(subargument="unknown"), error_match="recomputation")
    mutate("extra-formal-property", lambda r: r["formal"].update(forged=True), error_match="Invalid ASPIC+ result")
    for field in ("theory_digest","snapshot_digest","source_digest"):
        mutate("source-" + field, lambda r,f=field: r["source_map"].update({f:"0"*64}), error_match="Source map" if field != "snapshot_digest" else "Snapshot digest")
    mutate("wrong-goal-claim", lambda r: (r.update(claim="reportable"),r["source_map"]["goal"].update(claim="reportable")), error_match="goal claim")
    mutate("duplicate-claim-atom", lambda r: r["source_map"]["claims"]["reportable"].update(atom=r["source_map"]["claims"]["run_passes"]["atom"]), error_match="claim identity")
    for field, value in (("atom",[]),("identity","not an observation"),("rank",0)):
        mutate("invalid-evidence-" + field, lambda r,f=field,v=value: r["source_map"]["evidence"]["primary_data"].update({f:v}), error_match="source map" if field != "rank" else "Source map evidence")
    for field, value in (("rank",0),("conclusion_atom","unrelated"),("applicability_atom","absent")):
        mutate("invalid-rule-" + field, lambda r,f=field,v=value: r["source_map"]["arguments"]["primary_route"].update({f:v}), error_match="Source map rule")
    mutate("absent-contrary", lambda r: r["source_map"].update(contraries=[{"attacker":"absent","target":"absent"}]), error_match="Source map contrary")
    mutate("structural-basis-ordinary-premise", lambda r: r["source_map"].update(structural_basis={"atom":r["source_map"]["evidence"]["primary_data"]["atom"],"meaning":"No observation"}), error_match="unmapped axiom")
    mutate("structural-basis-absent", lambda r: r["source_map"].update(structural_basis={"atom":"absent","meaning":"No observation"}), error_match="unmapped axiom")
    for issues in (None, [], ["probably_absent"], ["missing_observation","missing_observation"]):
        label = "missing" if issues is None else "empty" if not issues else "unknown" if issues[0] == "probably_absent" else "duplicates"
        def change(r, issues=issues):
            item = r["source_map"]["evidence"]["primary_data"]
            if issues is None: item.pop("availability_issues")
            else: item["availability_issues"] = issues
        mutate("availability-issues-" + label, change, missing, "availability issues" if issues is None or not issues else "source map structure")
    for name,value in (("negative",-1),("float",1.0),("boolean",True),("zero",0),("raised",129)):
        mutate("snapshot-limit-" + name, lambda r,v=value: r["source_map"]["execution_limits"].update(formal_arguments=v), error_match="Snapshot budgets" if name == "raised" else "snapshot execution budgets")
    mutate("unknown-snapshot-budget", lambda r: r["source_map"]["execution_limits"].update(unknown=1), error_match="snapshot execution budgets")
    mutate("incomplete-recomputation", lambda r: r["source_map"]["execution_limits"].update(formal_arguments=1), error_match="Bounded ASPIC recomputation failed")
    mutate("empty-profile", lambda r: r.update(profile=""), error_match="view metadata")
    mutate("invalid-view-digest", lambda r: (r.update(source_digest="G"*64),r["source_map"].update(source_digest="G"*64)), error_match="view metadata")
    mutate("malformed-unused-source-entry", lambda r: r["source_map"]["arguments"].update(unused={"emitted":False,"span":{"line":0}}), error_match="source map structure")
    mutate("malformed-source-container", lambda r: r["source_map"].update(arguments=[]), error_match="source map structure")
    mutate("closed-review-annotation", lambda r: r["source_map"].update(formal_directives=[{
        "kind":"rank", "name":"primary_route", "other":None, "rank":500,
        "review":"synthetic-review/rank", "span":None, "target_kind":"argument", "source_file":None,
        "extra":"not in the reviewed-annotation contract"}]), error_match="source map structure")

    result = {"schema":"eal-aspic-export-reference/1", "source_revision":REVISION,
              "view_schema_sha256":hashlib.sha256(retained_schema.read_bytes()).hexdigest(),
              "provenance":"Synthetic production regression witnesses; source-map assertions remain supplied, not authenticated",
              "comparison":"Exact complete view for valid cases; checked rejection category for invalid cases. Formal labels, ordering, witnesses, ranks and digests are never projected.",
              "cases":cases}
    output = json.dumps(result, ensure_ascii=False, allow_nan=False, indent=2) + "\n"
    destination = ROOT / "fixtures/aspic/export-reference.json"
    if args.check:
        if destination.read_text() != output:
            raise SystemExit("ASPIC export reference drift")
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(output)
    print(json.dumps({"cases":len(cases),"views":sum("view" in c for c in cases)}))


if __name__ == "__main__":
    main()
