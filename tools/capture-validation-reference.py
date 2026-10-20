#!/usr/bin/env python3
"""Capture static checks and typed binding witnesses from the pinned Python port reference."""
import argparse
from dataclasses import asdict
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]


def capture(reference):
    if subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip() != REVISION:
        raise SystemExit("The Python reference must be at the pinned revision")
    if subprocess.check_output(["git", "-C", str(reference), "status", "--porcelain", "--", "src/eal", "grammar"], text=True).strip():
        raise SystemExit("The Python reference must be unmodified")
    sys.path.insert(0, str(reference / "src"))
    from eal.parser import parse
    from eal.semantics import validate, parse_time
    from eal.propositions import prepare_binding, check_result
    baseline = (ROOT / "fixtures/parser/sources/baseline.eal").read_text()
    typed = (ROOT / "fixtures/parser/sources/typed-proposition.eal").read_text()
    cases = [("baseline", baseline), ("typed", typed)]
    changes = [
        ("unknown_tool", 'tool runner\n', 'tool missing\n'),
        ("unknown_environment", 'environment lab\n', 'environment missing\n'),
        ("unknown_validator", 'validate observation', 'validate missing'),
        ("negative_age", 'max_age 60', 'max_age -1'),
        ("nonfinite_age", 'max_age 60', 'max_age 1e999'),
        ("empty_version", 'version "1"', 'version ""'),
        ("empty_rationale", 'rationale "The bounded test result supports the stated test claim."', 'rationale ""'),
        ("empty_statement", 'statement "The smoke test passes."', 'statement ""'),
        ("unversioned_method", 'method "structured/1"', 'method "structured"'),
        ("unknown_method", 'method "structured/1"', 'method "unknown/1"'),
        ("naive_assumption_time", '2027-01-01T00:00:00Z', '2027-01-01T00:00:00'),
    ]
    cases.extend((name, baseline.replace(old, new)) for name, old, new in changes)
    typed_changes = [
        ("unknown_quantity", 'quantity "pressure"', 'quantity "flow"'),
        ("incompatible_unit", 'unit "kPa"', 'unit "L/s"'),
        ("missing_binding", ' binding trial', ''),
        ("unbound_source", 'binding trial', 'binding absent'),
        ("wrong_output_type", 'result "estimate" >= 5', 'result "estimate" == true'),
        ("unknown_output", 'result "estimate" >= 5', 'result "sample_size" >= 5'),
        ("missing_query", 'query {"assignment": "randomised"}', 'query {}'),
        ("extra_query", 'query {"assignment": "randomised"}', 'query {"assignment":"randomised","extra":1}'),
        ("naive_proposition_time", '2026-09-23T10:00:00Z', '2026-09-23T10:00:00'),
        ("wrong_evidence_kind", 'kind experiment', 'kind test'),
        ("typed_boolean_order", 'result "estimate" >= 5', 'result "estimate" >= true'),
        ("reasoning_missing_output", 'rationale "Difference of means in the declared randomised experiment."', 'rationale "Difference of means in the declared randomised experiment."\nrequire absent >= 0'),
    ]
    cases.extend((name, typed.replace(old, new)) for name, old, new in typed_changes)
    cases.extend([
        ("dependency_cycle", baseline + 'argument loop = [premises working] via measured => working\n'),
        ("unknown_objection", baseline + 'objection challenge = [evidence observation] -x> claim missing\n'),
        ("duplicate_symbol", baseline + 'tool runner {\nversion "2"\n}\n'),
        ("formal_wrong_kind", baseline + 'strict working reviewed "r"\n'),
        ("formal_missing_review", baseline + 'strict first reviewed ""\n'),
        ("formal_strict_rank", baseline + 'strict first reviewed "r"\nrank first 2 reviewed "r"\n'),
        ("formal_rank_range", baseline + 'rank first 1001 reviewed "r"\n'),
        ("formal_self_contrary", baseline + 'contrary working to working reviewed "r"\n'),
        ("formal_self_priority", baseline + 'prefer first over first reviewed "r"\n'),
        ("formal_priority_cycle", baseline + 'argument second = [evidence observation] via measured => working\nprefer first over second reviewed "r"\nprefer second over first reviewed "r"\n'),
        ("unused_local_metadata", baseline + 'pattern unused(x: claim) {\nreasoning bad {\nmethod "structured/1"\nrationale ""\n}\n}\n'),
    ])
    result = []
    for name, source in cases:
        diagnostics = []
        for diagnostic in validate(parse(source)):
            value = asdict(diagnostic)
            diagnostics.append(value)
        result.append({"name": name, "source": source, "diagnostics": diagnostics})
    timestamps = [
        "2026-10-20T12:34:56Z", "2026-10-20T12:34:56+01:00", "20261020T123456+0100",
        "2026-W43-2T12:34:56Z", "2026W432T123456Z", "2026-W43T12Z",
        "2026-10-20T12:34:56.123456789Z", "2026-10-20T12:34:56,123456789Z",
        "2026-10-20T12:34+01", "2026-10-20T12:34:56+01:02:03.4",
        "2026-10-20T12:34:56+00:00:00.5", "0001-01-01T00:00:00+01:00",
        "9999-12-31T23:59:59-01:00", "2026-10-20", "2026-10-20 12:34:56Z",
        "2026-10-20T12:34:56", "2026-01-1T12:34:56Z", "2026W€T12:34:56Z",
        "2026-W--43T12:34:56Z", "2026-10-20T24:00:00Z", "2026-10-20T12:34:60Z",
        "2026W4-3T12Z", "2026-W-43T12Z", "2026-10-20T12:00:00+01:60",
        "2026-10-20T12:00:00+01:02:60", "2026-10-20T12:00:00+23:59:60",
    ]
    times = []
    for value in timestamps:
        try:
            time = parse_time(value)
            times.append({"input": value, "unix_microseconds": (time.toordinal()-719163)*86400000000 + time.hour*3600000000 + time.minute*60000000 + time.second*1000000 + time.microsecond})
        except (ValueError, OverflowError):
            times.append({"input": value, "unix_microseconds": None})
    proposition = parse(typed).claims['raised'].proposition
    envelope = {'schema':'EAL/typed-input/1','method':'causal/1','subject':'pump-A','quantity':'pressure','unit':'Pa','scope':'experiment-v1','valid_from':'2026-09-23T10:00:00Z','valid_until':'2026-09-23T11:00:00Z','payload':{'assignment':'randomised','treatment':[10000,12000],'control':[3000,4000]}}
    bindings = []
    for name, changes in [("valid",{}),("wrong_subject",{'subject':'other'}),("wrong_unit",{'unit':'L/s'}),("short_interval",{'valid_until':'2026-09-23T10:59:59Z'}),("wrong_query",{'payload':{'assignment':'observational','treatment':[1,2],'control':[0,0]}})]:
        value = {**envelope, **changes}
        payload, trace = prepare_binding(proposition,'causal/1','trial',value)
        # Python implementation identities intentionally differ in Rust; retain
        # the correspondence result, query and digest rather than that identity.
        del trace['method_contract']
        bindings.append({'name':name,'input':value,'payload':payload,'trace':trace})
    scalar_results=[]
    for input_unit, value in [('Pa',7500),('kPa',7.5),('MPa',1e308),('Pa',5e-324)]:
        checked=check_result(proposition,'causal/1',{'estimate':value},{'input_unit':input_unit})
        scalar_results.append({'input_unit':input_unit,'estimate':value,'result':checked})
    return {'schema':'eal-validation-reference/1','source_revision':REVISION,'cases':result,'times':times,'binding_proposition':asdict(proposition),'bindings':bindings,'scalar_results':scalar_results}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reference', type=Path)
    parser.add_argument('--check', action='store_true')
    args=parser.parse_args()
    target=ROOT/'fixtures/validation/reference.json'
    text=json.dumps(capture(args.reference.resolve()),ensure_ascii=False,indent=2,allow_nan=False)+'\n'
    if args.check:
        if target.read_text()!=text: raise SystemExit('Validation fixtures differ')
    else:
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_text(text)
