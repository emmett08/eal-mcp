#!/usr/bin/env python3
"""Retain exact bounded ASPIC witnesses from the pinned, unchanged Python source.

This is a development oracle capture. Native tests consume JSON without Python,
optional solvers, observation collection or an external service.
"""
from __future__ import annotations

import argparse
import ast
from copy import deepcopy
import hashlib
from itertools import product
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]
NOW = "2026-09-23T12:00:00Z"
CONTEXT = {"site": "bench"}


def premise(atom, rank=500, *, axiom=False):
    return {"atom": atom, "kind": "axiom"} if axiom else {"atom": atom, "kind": "ordinary", "rank": rank}


def rule(identifier, antecedents, consequent, rank=None, name=None):
    value = {"id": identifier, "kind": "strict" if rank is None else "defeasible",
             "antecedents": antecedents, "consequent": consequent}
    if rank is not None:
        value.update(rank=rank, name=name or identifier)
    return value


def theory(premises, rules=(), edges=(), goal="p", **options):
    value = {"premises": premises, "rules": list(rules),
             "contraries": [{"attacker": a, "target": b} for a, b in edges], "goal": goal}
    if options:
        value["options"] = options
    return value


def base():
    # tests/test_aspic.py::theory: weak strict undercutter of a stronger rule.
    return theory([premise("report_checked", axiom=True), premise("latency_ok", 6), premise("trace_gap", 1)],
                  [rule("r_pass", ["report_checked", "latency_ok"], "run_passes", 7, "applicable_pass"),
                   rule("r_gap", ["trace_gap"], "~applicable_pass")],
                  [("~applicable_pass", "applicable_pass")], "run_passes")


def routes():
    # tests/test_aspic_extensions.py::two_routes: weakest and last link disagree.
    return theory([premise("a", 100), premise("b", 500)],
                  [rule("r_x", ["a"], "x", 800, "use_x"), rule("r_y", ["b"], "y", 700, "use_y")],
                  [("x", "y"), ("y", "x")], "x")


def cases():
    captured = []

    def add(name, value, *, limits=None, provenance="synthetic finite theory"):
        captured.append({"name": name, "payload": {"theory": deepcopy(value)},
                         "limits": limits or {}, "provenance": provenance})

    def payload(name, value):
        captured.append({"name": name, "payload": deepcopy(value), "limits": {},
                         "provenance": "synthetic adverse input"})

    add("weaker-undercutter", base(), provenance="tests/test_aspic.py::theory")
    alternative = base()
    alternative["premises"].append(premise("cold_run_ok", 8))
    alternative["rules"].append(rule("r_cold", ["cold_run_ok"], "run_passes", 8, "applicable_cold"))
    add("independent-route-survives-undercut", alternative)
    add("pure-axiom-strict-closure", theory([premise("p", axiom=True)], [rule("s", ["p"], "q")], goal="q"))
    add("ordinary-rank-zero", theory([premise("p", 0)]))
    add("ordinary-rank-one-thousand", theory([premise("p", 1000)]))
    add("unconstructed-goal", theory([premise("p")], goal="absent"))
    pair = theory([premise("p"), premise("q")], edges=[("p", "q"), ("q", "p")])
    for semantics, query_mode, preference in product(
            ("grounded", "preferred", "stable"), ("credulous", "sceptical"),
            ("minimum_rank", "last_link_rank", "last_link_partial")):
        value = routes()
        value["options"] = {"semantics": semantics, "query_mode": query_mode, "preference": preference}
        add(f"routes-{semantics}-{query_mode}-{preference}", value)
    for semantics, query_mode in product(("grounded", "preferred", "stable"), ("credulous", "sceptical")):
        for label, edges in [("mutual", [("p", "q"), ("q", "p")]),
                             ("odd-cycle", [("p", "q"), ("q", "r"), ("r", "p")]),
                             ("defended-chain", [("p", "q"), ("q", "r")])]:
            add(f"small-{label}-{semantics}-{query_mode}",
                theory([premise("p"), premise("q"), premise("r")], edges=edges,
                       semantics=semantics, query_mode=query_mode))
    absent_cycle = theory([premise("p"), premise("q"), premise("r")],
                          edges=[("p", "q"), ("q", "r"), ("r", "p")], goal="absent", semantics="stable")
    add("no-extension-precedes-unconstructed-goal", absent_cycle)
    for semantics in ("preferred", "stable"):
        add("different-arguments-sceptically-support-one-conclusion-" + semantics,
            theory([premise("p"), premise("q")],
                   [rule("left", ["p"], "goal", 500), rule("right", ["q"], "goal", 500)],
                   [("p", "q"), ("q", "p")], "goal", semantics=semantics, query_mode="sceptical"))
    for attack, reciprocal, attacker_rank in product(("undermine", "rebut", "undercut"), (False, True), (1, 9)):
        if attack == "undermine":
            value = theory([premise("p", 8), premise("q", attacker_rank)],
                           [rule("downstream", ["p"], "goal")], [("q", "p")], "goal")
            reverse = ("p", "q")
        else:
            target = "use_p" if attack == "undercut" else "p"
            value = theory([premise("seed", axiom=True), premise("q", attacker_rank)],
                           [rule("p_rule", ["seed"], "p", 8, "use_p"), rule("downstream", ["p"], "goal")],
                           [("q", target)], "goal")
            reverse = (target, "q")
        if reciprocal:
            value["contraries"].append({"attacker": reverse[0], "target": reverse[1]})
        add(f"{attack}-{'reciprocal' if reciprocal else 'one-way'}-rank-{attacker_rank}", value)
    # A defeat edge may have several distinct witnesses through shared descendants.
    add("multiple-defeat-witnesses", theory([premise("p"), premise("q"), premise("attack", 1)],
        [rule("joint", ["p", "q"], "goal", 500)], [("attack", "p"), ("attack", "q")], "goal"))
    add("undercut-precedes-rebut-for-one-subargument", theory([premise("seed", axiom=True), premise("q")],
        [rule("d", ["seed"], "p", 500, "p")], [("q", "p")]))
    add("axiom-is-not-undermined", theory([premise("p", axiom=True), premise("q")],
        [rule("s", ["p"], "goal")], [("q", "p"), ("p", "q")], "goal"))
    partial = routes()
    partial["options"] = {"preference": "last_link_partial"}
    partial["priorities"] = [{"kind": "rule", "higher": "r_x", "lower": "r_y"}]
    add("partial-rule-priority", partial)
    partial_premises = theory([premise("p", 1), premise("q", 1000), premise("r", 500)],
        edges=[("p", "q"), ("q", "p")], preference="last_link_partial")
    partial_premises["priorities"] = [{"kind": "premise", "higher": "p", "lower": "r"},
                                      {"kind": "premise", "higher": "r", "lower": "q"}]
    add("partial-premise-priority-transitive-closure", partial_premises)
    strict_last = theory([premise("p", 10), premise("q", 20), premise("r", 30)],
        [rule("d1", ["p"], "x", 100), rule("d2", ["q"], "y", 200),
         rule("join", ["x", "y"], "goal"), rule("other", ["r"], "opposite", 150)],
        [("goal", "opposite"), ("opposite", "goal")], "goal", preference="last_link_rank")
    add("strict-last-link-unions-uncertain-frontier", strict_last)
    cyclic = theory([premise("p")], [rule("r_q", ["p"], "q"), rule("r_p", ["q"], "p"),
        rule("r_y", ["x"], "y"), rule("r_x", ["y"], "x")], goal="x")
    add("finite-founded-cycle-and-unfounded-cycle", cyclic)
    add("finite-cycle-constructed-goal", {**cyclic, "goal": "q"})
    add("self-loop-rule-never-repeats-conclusion", theory([premise("p")], [rule("self", ["p"], "p")]))
    diamond = theory([premise(atom, axiom=True) for atom in ("p1", "p2", "q1", "q2")],
        [rule("early_goal", ["p", "q"], "goal", 5, "apply_goal"),
         *[rule("r_" + atom, [atom], atom[0]) for atom in ("p1", "p2", "q1", "q2")]], goal="goal")
    add("cartesian-alternative-antecedents", diamond)
    reversed_rules = deepcopy(diamond)
    reversed_rules["rules"].reverse()
    add("cartesian-reordered-source-rules", reversed_rules)
    for middle_strict, goal_strict in product((False, True), repeat=2):
        add(f"shared-diamond-direct-children-{middle_strict}-{goal_strict}",
            theory([premise("p"), premise("~d1", 1)],
                   [rule("d1", ["p"], "x", 500), rule("d2", ["p"], "x", 500),
                    rule("middle", ["x"], "y", None if middle_strict else 500),
                    rule("joint", ["x", "y"], "goal", None if goal_strict else 500)],
                   [("~d1", "d1")], "goal"))
    cyclic_diamond = deepcopy(diamond)
    cyclic_diamond["rules"].append(rule("cycle_back", ["goal"], "p"))
    add("cyclic-cartesian-finite-branch-routes", cyclic_diamond)
    add("numeric-argument-identifier-order", theory([premise("p" + str(n)) for n in range(12)]))
    many = theory([premise("p" + str(n)) for n in range(129)], goal="p128")
    add("raised-host-argument-budget", many,
        limits={"formal_premises": 200, "formal_atoms": 200, "formal_arguments": 200})
    for semantics in ("preferred", "stable"):
        high = deepcopy(many)
        high["contraries"] = [{"attacker": "p127", "target": "p128"}, {"attacker": "p128", "target": "p127"}]
        high["options"] = {"semantics": semantics, "query_mode": "sceptical"}
        add("extension-masks-cross-128-bits-" + semantics, high,
            limits={"formal_premises": 200, "formal_atoms": 200, "formal_arguments": 200})
    # The explicit profile rejects undefeated strict/axiom conflicts.
    add("contrary-axiom-closure", theory([premise("p", axiom=True), premise("q", axiom=True)], edges=[("p", "q")]))
    for reciprocal in (False, True):
        value = theory([premise("p"), premise("r")], [rule("s_yes", ["p"], "yes"), rule("s_no", ["r"], "no")],
                       [("yes", "no"), *([("no", "yes")] if reciprocal else [])], "yes")
        add(f"unsupported-strict-contraries-{reciprocal}", value)
        value["premises"].append(premise("not_p", axiom=True))
        value["contraries"].append({"attacker": "not_p", "target": "p"})
        add(f"strict-contraries-resolved-through-premise-{reciprocal}", value)
    extension_conflict = theory([premise("p"), premise("q")],
        [rule("yes", ["p"], "yes"), rule("no", ["p"], "no")],
        [("p", "q"), ("q", "p"), ("yes", "no")], "yes", semantics="preferred")
    add("unsupported-contrary-conclusions-only-in-extension", extension_conflict)
    # Root/schema/lexical/identity failures remain exact returned errors.
    for name, value in [("payload-null", None), ("payload-array", []), ("payload-empty", {}),
                        ("payload-extra-key", {"theory": pair, "extra": True}),
                        ("theory-null", {"theory": None}), ("theory-array", {"theory": []})]:
        payload(name, value)
    malformed = []
    def mutate(name, path, value):
        copied = deepcopy(base())
        target = copied
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        malformed.append((name, copied))
    for name, path, value in [
        ("extra-theory-property", ["extra"], 1), ("no-premises", ["premises"], []),
        ("premise-kind", ["premises", 0, "kind"], "strict"),
        ("axiom-unexpected-rank", ["premises", 0, "rank"], 500),
        ("ordinary-rank-boolean", ["premises", 1, "rank"], True),
        ("ordinary-rank-float", ["premises", 1, "rank"], 6.0),
        ("ordinary-rank-negative", ["premises", 1, "rank"], -1),
        ("ordinary-rank-too-high", ["premises", 1, "rank"], 1001),
        ("premise-atom-unicode", ["premises", 0, "atom"], "α"),
        ("premise-atom-space", ["premises", 0, "atom"], "p q"),
        ("premise-atom-too-long", ["premises", 0, "atom"], "p" * 66),
        ("rule-id-negative-literal", ["rules", 0, "id"], "~invalid"),
        ("rule-id-extra-property", ["rules", 0, "extra"], True),
        ("empty-rule-antecedents", ["rules", 0, "antecedents"], []),
        ("repeated-rule-antecedent", ["rules", 0, "antecedents"], ["latency_ok", "latency_ok"]),
        ("invalid-rule-consequent", ["rules", 0, "consequent"], "1bad"),
        ("defeasible-name-negative", ["rules", 0, "name"], "~invalid"),
        ("goal-invalid", ["goal"], "p q"),
        ("same-atom-contrary", ["contraries"], [{"attacker": "p", "target": "p"}]),
        ("invalid-contrary-atom", ["contraries", 0, "attacker"], "!bad"),
        ("unknown-option", ["options"], {"unsupported": True}),
        ("invalid-semantics", ["options"], {"semantics": "complete"}),
        ("invalid-query-mode", ["options"], {"query_mode": "skeptical"}),
        ("invalid-preference", ["options"], {"preference": "weakest_link"}),
        ("priority-without-partial-option", ["priorities"], [{"kind": "rule", "higher": "r_pass", "lower": "r_gap"}]),
    ]:
        mutate(name, path, value)
    for name, value in malformed:
        add(name, value, provenance="synthetic adverse theory")
    for name, table, entry in [("duplicate-premise", "premises", base()["premises"][0]),
                               ("duplicate-rule-id", "rules", base()["rules"][0]),
                               ("duplicate-contrary", "contraries", base()["contraries"][0])]:
        value = base()
        value[table].append(entry)
        add(name, value)
    value = base()
    value["rules"].append(rule("different_id", ["trace_gap"], "other", 5, "applicable_pass"))
    add("duplicate-defeasible-name", value)
    for property_name in ("premises", "rules", "contraries", "goal"):
        value = base()
        del value[property_name]
        add("missing-" + property_name, value)
    for name, priority in [("priority-unknown-element", {"kind": "rule", "higher": "r_x", "lower": "missing"}),
                           ("priority-self", {"kind": "rule", "higher": "r_x", "lower": "r_x"}),
                           ("priority-wrong-kind", {"kind": "premise", "higher": "r_x", "lower": "r_y"})]:
        value = deepcopy(partial)
        value["priorities"] = [priority]
        add(name, value)
    value = deepcopy(partial)
    value["priorities"].append(deepcopy(value["priorities"][0]))
    add("priority-duplicate", value)
    value = deepcopy(partial)
    value["priorities"].append({"kind": "rule", "higher": "r_y", "lower": "r_x"})
    add("priority-cycle", value)
    add("maximum-positive-atom-length", theory([premise("p" * 64)], goal="p" * 64))
    add("maximum-negative-atom-length", theory([premise("~" + "p" * 64)], goal="~" + "p" * 64))
    # Each independent budget has an exact, non-conclusive failure witness.
    for name, value, limits in [
        ("budget-premises", pair, {"formal_premises": 1}),
        ("budget-rules", base(), {"formal_rules": 1}),
        ("budget-contraries", pair, {"formal_contraries": 1}),
        ("budget-antecedents", base(), {"formal_antecedents": 1}),
        ("budget-atoms", pair, {"formal_atoms": 1}),
        ("budget-arguments", base(), {"formal_arguments": 3}),
        ("budget-construction", diamond, {"formal_construction": 1}),
        ("budget-cyclic-construction-revisits", cyclic, {"formal_construction": 1}),
        ("budget-defeat-witnesses", pair, {"formal_defeats": 1}),
        ("budget-duplicate-edge-witnesses", captured[next(n for n,c in enumerate(captured) if c["name"] == "multiple-defeat-witnesses")]["payload"]["theory"], {"formal_defeats": 2}),
        ("budget-extension-enumeration", {**pair, "options": {"semantics": "preferred"}}, {"extension_search": 1}),
        ("budget-maximal-extension-filter", {**pair, "options": {"semantics": "preferred"}}, {"extension_search": 7}),
        ("budget-priority-closure", partial_premises, {"formal_defeats": 2}),
    ]:
        add(name, value, limits=limits)
    return captured


def literal(reference, filename, variable):
    for node in ast.parse((reference / "tests" / filename).read_text()).body:
        if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == variable for target in node.targets):
            return ast.literal_eval(node.value)
    raise ValueError(f"No literal {filename}:{variable}")


def source_cases(reference):
    from eal.aspic_compiler import compile_eal_aspic
    from eal.evaluator import canonical_digest, environment_context, environment_fingerprint
    from eal.parser import parse

    base_source = literal(reference, "test_aspic_compiler.py", "BASE")
    variants = [("source-alternative-routes", base_source)]
    for kind, target in [("claim", "run_passes"), ("reasoning", "authored"),
                         ("assumption", "calibration"), ("argument", "primary_route")]:
        variants.append(("source-attack-" + kind, base_source + f'\nobjection gap = [evidence gap_data] -x> {kind} {target}\n'))
    variants.append(("source-defended-undercut", base_source + '''
objection gap = [evidence gap_data] -x> argument primary_route
objection defence = [evidence probe_data] -x> objection gap
'''))
    variants.append(("source-reviewed-strict-and-ranks", base_source + '''
strict reporting_route reviewed "synthetic-review/strict-report"
rank primary_data 600 reviewed "synthetic-review/premise"
rank probe_route 700 reviewed "synthetic-review/rule"
prefer probe_route over primary_route reviewed "synthetic-review/priority"
'''))
    result = []
    for name, source in variants:
        program = parse(source)
        records = {}
        for evidence_name, evidence in program.evidence.items():
            tool = program.tools[evidence.tool]
            local = environment_context(evidence.environment, CONTEXT)
            acquisition = {"tool": tool.name, "tool_version": tool.version, "input": evidence.input, "context": local}
            value = {"ok": True}
            records[evidence_name] = {"evidence_id": evidence_name, "source_digest": program.source_digest,
                "tool": tool.name, "tool_version": tool.version, "tool_binding_digest": "0" * 64,
                "evidence_kind": evidence.kind, "environment": evidence.environment,
                "environment_fingerprint": environment_fingerprint(evidence.environment, local),
                "input_digest": canonical_digest(evidence.input), "collected_at": NOW,
                "input": evidence.input, "context": local, "acquisition_request": acquisition,
                "acquisition_request_digest": canonical_digest(acquisition),
                "request_digest": canonical_digest({"evidence_id": evidence_name, "environment": evidence.environment, **acquisition}),
                "run_id": "synthetic-aspic-reference", "status": "ok", "value": value, "data_digest": canonical_digest(value)}
        compiled = compile_eal_aspic(source, records, goal="run_passes", now=NOW, context=CONTEXT)
        result.append({"name": name, "payload": {"theory": compiled.theory}, "limits": {},
                       "provenance": "production compile_eal_aspic of synthetic tests/test_aspic_compiler.py BASE variant",
                       "source": source, "source_sha256": hashlib.sha256(source.encode()).hexdigest()})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = args.reference.resolve()
    revision = subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip()
    if revision != REVISION:
        raise SystemExit("Reference checkout must be pinned at " + REVISION)
    if subprocess.check_output(["git", "-C", str(reference), "status", "--porcelain", "--", "src/eal", "grammar", "tests"], text=True):
        raise SystemExit("Reference production source, grammar and tests must remain unchanged")
    sys.path.insert(0, str(reference / "src"))
    from eal.aspic import _validate, solve_aspic
    from eal.limits import BudgetExceeded, ExecutionLimits, using_limits

    def observation(operation):
        try:
            return {"ok": True, "value": operation()}
        except (ValueError, TypeError, KeyError, OverflowError, RecursionError) as error:
            return {"ok": False, "error": str(error), "error_type": type(error).__name__,
                    "budget_exceeded": isinstance(error, BudgetExceeded)}

    captured = cases() + source_cases(reference)
    assert len({case["name"] for case in captured}) == len(captured)
    for case in captured:
        payload = case["payload"]
        target = payload.get("theory") if type(payload) is dict else None
        case["validation_target"] = deepcopy(target)
        with using_limits(ExecutionLimits(**case["limits"])):
            case["validation"] = observation(lambda: (_validate(target), None)[1])
            case["solve"] = observation(lambda: solve_aspic(payload))
    result = {"schema": "eal-aspic-reference/1", "source_revision": REVISION,
              "provenance": "Synthetic finite theories and checked source-derived theories; no experimental measurements",
              "comparison": "Every solver output field and ordered array, theory digest, validation and solve error string is exact; no metadata projection or numerical tolerance",
              "independent_extensions": "Native exhaustive subset definitions check every irreflexive directed graph with one through three nodes; optional PyArg is not required or silently skipped",
              "pyarg_capture": "not used; this retained oracle is the pinned Python implementation, independently checked by native subset definitions",
              "cases": captured}
    output = json.dumps(result, ensure_ascii=False, allow_nan=False, indent=2) + "\n"
    destination = ROOT / "fixtures/aspic/reference.json"
    destination.parent.mkdir(parents=True, exist_ok=True)
    if args.check:
        if destination.read_text() != output:
            raise SystemExit("ASPIC reference drift")
    else:
        destination.write_text(output)
    print(json.dumps({"cases": len(captured), "valid_theories": sum(case["validation"]["ok"] for case in captured),
                      "solved": sum(case["solve"]["ok"] for case in captured),
                      "budget_errors": sum(case["solve"].get("budget_exceeded", False) for case in captured)}))


if __name__ == "__main__":
    main()
