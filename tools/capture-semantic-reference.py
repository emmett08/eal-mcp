#!/usr/bin/env python3
"""Capture cross-component semantic witnesses from the pinned Python source.

Development only. The Rust comparison suite reads the retained JSON without
Python, network, filesystem imports or executable collectors.
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
NOW = "2026-09-23T12:00:00Z"
CONTEXT = {"site": "bench"}


def literals(path):
    values = {}
    for node in ast.parse(path.read_text()).body:
        if isinstance(node, ast.Assign):
            try:
                value = ast.literal_eval(node.value)
            except (ValueError, TypeError):
                continue
            for target in node.targets:
                if isinstance(target, ast.Name):
                    values[target.id] = value
    return values


def cases(reference):
    tests = reference / "tests"
    base = literals(tests / "test_evaluator.py")["SOURCE"]
    scoped = literals(tests / "test_scoped_composition.py")
    typed = literals(tests / "test_typed_propositions.py")
    composed = literals(tests / "test_composed_language.py")["BASE"]
    result = []

    def add(name, source=base, *, values=None, records=None, **options):
        result.append({"name": name, "source": source, "context": deepcopy(CONTEXT),
                       "now": NOW, "values": deepcopy(values if values is not None else
                       {"observed": {"passed": True}}), "record_changes": records or {},
                       **options})

    add("nested-support")
    first = base.index("claim working {")
    second = base.index("claim downstream {")
    end = base.index("argument base")
    add("premise-depth-one-edge-forward-order",
        base[:first] + base[second:end] + base[first:second] + base[end:], limits={"premise_depth": 1})
    add("missing-observations", values={})
    add("false-predicate", values={"observed": {"passed": False}})
    add("missing-field", values={"observed": {}})
    add("boolean-number-mismatch", values={"observed": {"passed": 1}})
    add("literal-user-span-and-contract-keys", values={"observed": {
        "passed": True, "span": {"start": {"line": 90}, "end": {"column": 91}},
        "method_contract": {"identity_basis": "literal payload", "implementation_version": "user value"},
        "$serde_json::private::Number": "1", "$serde_json::private::RawValue": "null"}})
    add("reviewed-argumentation-directives", base + '''
strict base reviewed "synthetic-review/explicit-implication"
rank observed 700 reviewed "synthetic-review/observation"
prefer observed over counterexample reviewed "synthetic-review/explicit-priority"
''')
    add("environment-mismatch", context={"site": "field"})
    add("age-at-boundary", records={"observed": {"collected_at": "2026-09-23T11:59:00Z"}})
    add("age-after-boundary", records={"observed": {"collected_at": "2026-09-23T11:58:59Z"}})
    add("future-observation", records={"observed": {"collected_at": "2026-09-23T12:00:01Z"}})
    add("naive-observation-time", records={"observed": {"collected_at": "2026-09-23T12:00:00"}})
    add("epoch-observation", records={"observed": {"collected_at": "1970-01-01T00:00:00Z"}})
    add("equivalent-timezone", records={"observed": {"collected_at": "2026-09-23T13:00:00+01:00"}})
    add("half-open-assumption-end", now="2026-09-23T13:00:00Z",
        records={"observed": {"collected_at": "2026-09-23T13:00:00Z"}})
    add("assumption-before-start", now="2026-09-23T10:59:59Z",
        records={"observed": {"collected_at": "2026-09-23T10:59:59Z"}})
    add("current-binding-match", binding_digests={"observed": "0" * 64})
    add("current-binding-mismatch", binding_digests={"observed": "1" * 64})
    for key, value in [
        ("source_digest", "other"), ("tool", "other"), ("tool_version", "2"),
        ("tool_binding_digest", "invalid"), ("evidence_kind", "sample"),
        ("environment", "elsewhere"), ("environment_fingerprint", "other"),
        ("input_digest", "other"), ("input", {"suite": "other"}),
        ("context", {"site": "elsewhere"}), ("acquisition_request", {}),
        ("acquisition_request_digest", "other"), ("request_digest", "other"),
        ("data_digest", "other"), ("run_id", ""), ("status", "error"),
    ]:
        add("record-mismatch-" + key, records={"observed": {key: value}})
    for kind, target in [("claim", "working"), ("reasoning", "method"),
                         ("assumption", "stable"), ("argument", "base")]:
        attack = f"\nobjection attack = [evidence counterexample] -x> {kind} {target}\n"
        add("attack-" + kind, base + attack,
            values={"observed": {"passed": True}, "counterexample": {"found": True}})
    add("independent-argument-survives", base + '''
objection attack = [evidence counterexample] -x> reasoning method
argument alternative = [evidence observed] via alternative_method => working
''', values={"observed": {"passed": True}, "counterexample": {"found": True}})
    add("source-free-attack-cycle", base + '''
objection first = [premises working] -x> objection second
objection second = [premises downstream] -x> objection first
''')
    add("circular-claim-objection", composed +
        "\nobjection circular = [premises outcome] -x> claim outcome\n",
        values={name: {"holds": True} for name in ("positive", "negative", "defence_data")})
    defended = composed + '''
objection challenged = [premises critique] -x> assumption applicable
objection defended = [evidence defence_data] -x> objection challenged
'''
    add("defended-assumption", defended,
        values={name: {"holds": True} for name in ("positive", "negative", "defence_data")})
    add("rebutted-defence", defended +
        "\nobjection reply = [evidence negative] -x> objection defended\n",
        values={name: {"holds": True} for name in ("positive", "negative", "defence_data")})
    add("recursive-list-pattern", scoped["BASE"] + scoped["RECURSIVE"],
        values={"first": {"ok": True}, "second": {"ok": True}})
    add("empty-recursive-list", (scoped["BASE"] + scoped["RECURSIVE"]).replace("[first, second]", "[]"), values={})
    add("unguarded-pattern-recursion", (scoped["BASE"] + scoped["RECURSIVE"]).replace("when es {", "context environment lab {"), values={})
    add("non-decreasing-pattern-recursion", (scoped["BASE"] + scoped["RECURSIVE"]).replace("es=tail(es)", "es=es"), values={})
    add("pattern-application-budget", scoped["BASE"] + scoped["RECURSIVE"],
        limits={"applications": 1}, values={})
    add("module-argument-forward-binding", scoped["BASE"] + '''
module release {
  claim ready { statement "Scoped release claim."
    environment lab
  }
  argument assess {
    argument child = [evidence first] via measured => intermediate
    claim intermediate { statement "Local claim."
      environment lab
    }
    [premises intermediate] via measured => ready
  }
}
''', values={"first": {"ok": True}, "second": {"ok": True}})
    imported = '''language "EAL/3"
import "support.eal" as support
argument ready = [evidence support.first] via support.measured => support.passed
'''
    add("trusted-relative-import", imported, imports={"support.eal": scoped["BASE"]},
        origin="main.eal", values={"support.first": {"ok": True}})
    add("definition-relative-template-import", '''language "EAL/3"
import "library/templates.eal" as templates
apply route = templates.attach()
argument result = [evidence route.data.first] via route.data.measured => route.data.passed
''', imports={"library/templates.eal": '''language "EAL/3"
pattern attach() {
  import "data.eal" as data
}
''', "library/data.eal": scoped["BASE"]}, origin="main.eal",
        values={"route.data.first": {"ok": True}})
    add("missing-import-inside-unused-pattern", '''language "EAL/3"
pattern unused() {
  import "absent.eal" as missing
}
''', imports={}, origin="main.eal", values={})
    add("import-cycle", 'language "EAL/3"\nimport "cycle.eal" as cycle\n',
        imports={"cycle.eal": 'language "EAL/3"\nimport "cycle.eal" as again\n'}, values={})
    add("import-root-escape", 'language "EAL/3"\nimport "../outside.eal" as outside\n', imports={}, values={})
    add("import-aggregate-byte-budget", imported, imports={"support.eal": scoped["BASE"]},
        origin="main.eal", limits={"source_bytes": len(scoped["BASE"].encode()) + 10}, values={})
    for name, old, new in [
        ("unknown-evidence", "evidence observed,", "evidence absent,"),
        ("wrong-reference-kind", "evidence observed,", "evidence working,"),
        ("unknown-method", 'method "structured/1"', 'method "missing/1"'),
        ("negative-max-age", "max_age 60", "max_age -1"),
        ("empty-argument", "[evidence observed, assumptions stable]", "[]"),
        ("premise-cycle", "[evidence observed, assumptions stable]", "[premises downstream]"),
    ]:
        add(name, base.replace(old, new), values={})
    add("duplicate-symbol", base + '\ntool runner { version "2"\n}\n', values={})
    for name, expression in [
        ("numeric-truthiness-static", "1 and true"),
        ("bad-function-static", 'shell("echo x") == 0'),
        ("dimension-mismatch-static", 'quantity(distance,"m") < quantity(duration,"s")'),
        ("division-error-not-short-circuited", "count(samples) == 0 or sum(samples) / count(samples) > 0"),
        ("unknown-or-true", "missing or ready"),
        ("python-negative-modulo", "reading % 3 == 1"),
        ("arbitrary-integer-comparison", "reading > 9007199254740992.0"),
    ]:
        values = {"samples": [], "ready": True, "reading": -5}
        if name == "arbitrary-integer-comparison": values["reading"] = 9007199254740993
        add(name, scoped["BASE"].replace("require ok", "require " + expression) +
            "\nargument result = [evidence first] via measured => passed\n",
            values={"first": values})
    for name, value in [
        ("typed-pressure-conversion", typed["VALUE"]),
        ("typed-negative-result", {**typed["VALUE"], "payload": {"assignment": "randomised", "treatment": [6000, 7000], "control": [3000, 4000]}}),
        ("typed-subject-mismatch", {**typed["VALUE"], "subject": "pump-B"}),
        ("typed-query-mismatch", {**typed["VALUE"], "payload": {**typed["VALUE"]["payload"], "assignment": "observational"}}),
        ("typed-uncovered-time", {**typed["VALUE"], "valid_from": "2026-09-23T10:00:01Z"}),
        ("typed-quantity-mismatch", {**typed["VALUE"], "quantity": "volumetric_flow"}),
    ]:
        add(name, typed["SOURCE"], values={"trial": value})
    for name, old, new in [
        ("typed-missing-binding", "binding trial", ""),
        ("typed-wrong-result-type", 'result "estimate" >= 5', 'result "estimate" == true'),
        ("typed-unknown-result-field", 'result "estimate" >= 5', 'result "sample_size" >= 5'),
        ("typed-missing-query", 'query {"assignment": "randomised"}', 'query {}'),
        ("typed-incompatible-unit", 'unit "kPa"', 'unit "L/s"'),
    ]:
        add(name, typed["SOURCE"].replace(old, new), values={})
    generic = '''language "EAL/3"
environment lab { require site == "bench"
}
tool collector { version "1"
}
evidence measured {
  tool collector
  environment lab
  max_age 60
  kind KIND
  require true
}
reasoning compute { method "METHOD/1"
  rationale "Synthetic cross-component calculation under the declared method."
  require PREDICATE
}
claim outcome { statement "The declared bounded computation supplies a result."
  environment lab
}
argument assessed = [evidence measured] via compute => outcome
'''
    counterfactual = {"variables": {"x": {"intercept": 2, "noise": 1, "coefficients": {}},
                                    "y": {"intercept": 1, "noise": 0, "coefficients": {"x": 4}}},
                      "intervention": {"variable": "x", "value": 7}, "outcome": "y"}
    temporal = {"start": 0, "end": 2, "max_gap": 1,
                "events": [{"time": 0, "value": 3}, {"time": 1, "value": 4}, {"time": 2, "value": 3}],
                "property": {"operator": "le", "value": 5}, "semantics": "sampled"}
    for name, method, kind, payload in [
        ("method-deductive-entailment", "deductive", "logical_case", {"premises": ["p", {"implies": ["p", "q"]}], "conclusion": "q"}),
        ("method-deductive-countermodel", "deductive", "logical_case", {"premises": ["q", {"implies": ["p", "q"]}], "conclusion": "p"}),
        ("method-deductive-inconsistency", "deductive", "logical_case", {"premises": ["p", {"not": "p"}], "conclusion": "q"}),
        ("method-inductive-wilson", "inductive", "sample", {"successes": 20, "trials": 25, "confidence": 0.95}),
        ("method-inductive-count-type", "inductive", "sample", {"successes": True, "trials": 25, "confidence": 0.95}),
        ("method-abductive-posterior", "abductive", "hypotheses", {"observed": "alarm", "candidates": [{"name": "fault", "prior": 0.25, "likelihood": 0.8}, {"name": "noise", "prior": 0.75, "likelihood": 0.1}]}),
        ("method-causal-means", "causal", "experiment", {"assignment": "randomised", "treatment": [10, 12], "control": [3, 4]}),
        ("method-counterfactual-affine", "counterfactual", "causal_model", counterfactual),
        ("method-analogical-mismatch", "analogical", "analogy", {"relevant_features": ["capacity", "safe"], "source": {"capacity": 2, "safe": True}, "target": {"capacity": 2.0, "safe": 1}}),
        ("method-temporal-coverage", "temporal", "trace", temporal),
        ("method-temporal-negative-finding", "temporal", "trace", {**temporal, "property": {"operator": "lt", "value": 4}}),
        ("method-temporal-incomplete-coverage", "temporal", "trace", {**temporal, "events": temporal["events"][:2]}),
    ]:
        predicate = {"deductive": "entailed == true", "inductive": "lower > 0.5",
                     "abductive": "best_posterior > 0.5", "causal": "estimate > 0",
                     "counterfactual": "difference > 0", "analogical": "match_fraction == 1",
                     "temporal": "holds == true"}[method]
        add(name, generic.replace("KIND", kind).replace("METHOD", method).replace("PREDICATE", predicate),
            values={"measured": payload})
    return result


def compilation_view(program):
    # Expressions retain the source reference's portable kind/value/arguments
    # tree. Rust adapters translate enum spellings before comparison.
    tables = ("environments", "tools", "evidence", "assumptions", "reasoning",
              "claims", "arguments", "objections")
    result = {name: {key: asdict(value) for key, value in getattr(program, name).items()}
              for name in tables}
    result.update(language=program.language, source_digest=program.source_digest,
                  pattern_names=list(program.patterns), application_names=list(program.applications),
                  duplicates=list(program.duplicates), declaration_count=program.declaration_count,
                  generated=list(program.generated), imports=program.imports,
                  source_files=program.source_files,
                  locations={name: asdict(span) for name, span in program.locations.items()},
                  argumentation_directives=[asdict(item) for item in program.argumentation_directives],
                  lowering_diagnostic_codes=[item.code for item in program.lowering_diagnostics])
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
        raise SystemExit("Reference source, grammar and tests must be unchanged")
    sys.path.insert(0, str(reference / "src"))
    from eal.evaluator import canonical_digest, environment_context, environment_fingerprint, evaluate
    from eal.limits import ExecutionLimits
    from eal.parser import parse
    from eal.semantics import validate

    def record(program, name, value, context):
        evidence = program.evidence[name]
        tool = program.tools[evidence.tool]
        local = environment_context(evidence.environment, context)
        acquisition = {"tool": tool.name, "tool_version": tool.version,
                       "input": evidence.input, "context": local}
        return {"evidence_id": name, "source_digest": program.source_digest,
                "tool": tool.name, "tool_version": tool.version,
                "tool_binding_digest": "0" * 64, "evidence_kind": evidence.kind,
                "environment": evidence.environment,
                "environment_fingerprint": environment_fingerprint(evidence.environment, local),
                "input_digest": canonical_digest(evidence.input), "collected_at": NOW,
                "input": evidence.input, "context": local, "acquisition_request": acquisition,
                "acquisition_request_digest": canonical_digest(acquisition),
                "request_digest": canonical_digest({"evidence_id": name, "environment": evidence.environment, **acquisition}),
                "run_id": "synthetic-reference-run", "status": "ok", "value": value,
                "data_digest": canonical_digest(value)}

    captured = []
    for case in cases(reference):
        case["source_sha256"] = hashlib.sha256(case["source"].encode()).hexdigest()
        entry = deepcopy(case)
        try:
            program = parse(case["source"], resolver=(lambda path: case["imports"][path]) if "imports" in case else None,
                            origin=case.get("origin", ""), limits=ExecutionLimits(**case.get("limits", {})))
            entry["compiled"] = compilation_view(program)
            diagnostics = validate(program)
            entry["diagnostics"] = [asdict(item) for item in diagnostics]
            entry["diagnostic_codes"] = [item.code for item in diagnostics]
            records = {name: record(program, name, value, case["context"])
                       for name, value in case["values"].items() if name in program.evidence}
            for name, changes in case["record_changes"].items(): records[name].update(changes)
            entry["records"] = records
            outcome = evaluate(program, records, now=case["now"], context=case["context"],
                               binding_digests=case.get("binding_digests"))
            outcome.pop("method_registry_fingerprint", None)
            entry["evaluation"] = outcome
        except (ValueError, TypeError, OverflowError, RecursionError, KeyError) as error:
            entry["error"] = str(error)
        captured.append(entry)
    result = {"schema": "eal-semantic-reference/1", "source_revision": REVISION,
              "provenance": "Synthetic cases derived from current source regression tests; no experimental measurements",
              "comparison": {"compiled": "Exact table/scalar/reference projection after translating expression enum spellings",
                             "diagnostics": "Exact ordered codes, messages, declaration identity, flat spans, expected and actual fields",
                             "evaluation": "All fields, excluding top-level method_registry_fingerprint and the implementation_version/source_digest/code_digest/artifact_digest/identity_basis members of argument reasoning_result.binding.method_contract; builtin execution labels are mapped to builtin",
                             "numbers": "Exact integers; floating-point tolerance only for independently documented numerical algorithms"},
              "cases": captured}
    output = json.dumps(result, ensure_ascii=False, allow_nan=False, indent=2) + "\n"
    destination = ROOT / "fixtures/semantics/reference.json"
    if args.check:
        if destination.read_text() != output: raise SystemExit("Semantic reference drift")
    else:
        destination.write_text(output)
    print(json.dumps({"cases": len(captured), "compiled": sum("compiled" in item for item in captured),
                      "valid": sum(not item.get("diagnostic_codes", ["error"]) for item in captured),
                      "errors": [item["name"] for item in captured if "error" in item]}))


if __name__ == "__main__":
    main()
