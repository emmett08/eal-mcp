#!/usr/bin/env python3
"""Capture finite observation-bound compiler witnesses from the pinned source.

Development only. Uses the unmodified compiler and its bounded ASPIC worker;
native tests consume retained JSON without Python or a collector.
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

def literals(path):
    result = {}
    for node in ast.parse(path.read_text()).body:
        if isinstance(node, ast.Assign):
            try: value = ast.literal_eval(node.value)
            except (ValueError, TypeError): continue
            for target in node.targets:
                if isinstance(target, ast.Name): result[target.id] = value
    return result

def cases(reference):
    base = literals(reference / "tests/test_aspic_compiler.py")["BASE"]
    result = []
    def add(name, source=base, **options):
        result.append({"name":name,"source":source,"goal":"run_passes","now":NOW,"context":deepcopy(CONTEXT),**options})
    add("baseline")
    for kind,target in [("argument","primary_route"),("claim","run_passes")]:
        source=base+f'objection challenge = [evidence gap_data] -x> {kind} {target}\n'
        for mask in product((False,True),repeat=4):
            missing=[name for name,present in zip(("primary_data","probe_data","gap_data","calibration_data"),mask) if not present]
            add("snapshot-matrix-"+kind+"-"+"".join("1"if x else"0"for x in mask),source,missing=missing)
    for kind,target in [("reasoning","authored"),("assumption","calibration")]:
        add("target-"+kind,base+f'objection challenge = [evidence gap_data] -x> {kind} {target}\n')
    attack=base+'objection challenge = [evidence gap_data] -x> argument primary_route\n'
    add("defended-objection",attack+'objection defence = [evidence probe_data] -x> objection challenge\n')
    add("forward-objection",base+'objection defence = [evidence probe_data] -x> objection challenge\nobjection challenge = [evidence gap_data] -x> argument primary_route\n')
    add("circular-objection",base+'objection circular = [premises run_passes] -x> claim run_passes\n')
    add("ungrounded-objection",base+'objection challenge = [] -x> argument primary_route\n')
    add("out-of-scope",context={"site":"field"})
    add("stale-primary",record_changes={"primary_data":{"collected_at":"2026-09-23T11:58:59Z"}})
    add("stale-primary-other-run",record_changes={"primary_data":{"collected_at":"2026-09-23T11:58:59Z","run_id":"other-unusable-run"}})
    add("current-binding",binding_digests={"primary_data":"1"*64})
    add("literal-payload-keys",extra_value={"span":{"line":77},"method_contract":{"implementation_version":"literal-user-data"},"$serde_json::private::Number":"123"})
    add("strict-downstream",attack+'strict reporting_route reviewed "review/explicit-implication"\n')
    add("rank-targets",attack+'rank primary_data 610 reviewed "review/measurement"\nrank calibration 620 reviewed "review/assumption"\nrank probe_route 630 reviewed "review/inference"\n')
    generated=base+'pattern measured(c: claim,r: reasoning,e: evidence) = [evidence e] via r => c\napply measured_probe = measured(c=run_passes,r=authored,e=probe_data)\nstrict measured_probe reviewed "review/generated"\n'
    add("generated-route",generated)
    for target in ("argument reporting_route","claim reportable","reasoning authored"):
        add("strict-objection-"+target.split()[0],base+f'objection challenge = [evidence gap_data] -x> {target}\nstrict reporting_route reviewed "review/explicit-implication"\n')
    contrary=base+'''claim run_fails {
statement "The bounded condition fails."
environment lab
}
argument failure_route = [evidence gap_data] via authored => run_fails
'''
    one='contrary run_fails to run_passes reviewed "review/incompatible-outcomes"\n'
    two=one+'contrary run_passes to run_fails reviewed "review/incompatible-outcomes"\n'
    add("one-way-contrary",contrary+'rank failure_route 0 reviewed "review/weak"\nrank probe_route 1000 reviewed "review/strong"\n'+one)
    add("ranked-reciprocal-contrary",contrary+'rank probe_route 800 reviewed "review/strong"\nrank probe_data 700 reviewed "review/observation"\nrank failure_route 400 reviewed "review/weak"\n'+two)
    add("strict-inconsistent-conclusions",contrary+'strict probe_route reviewed "review/implication"\nstrict failure_route reviewed "review/failure"\n'+two)
    for semantics in ("grounded","preferred","stable"):
        for query in ("sceptical","credulous"):
            add("options-"+semantics+"-"+query,contrary+two,semantics=semantics,query_mode=query)
    add("last-link-rank",contrary+two,preference="last_link_rank")
    priority='prefer probe_route over failure_route reviewed "review/explicit-priority"\n'
    add("partial-rule-priority",contrary+two+priority)
    add("partial-rule-priority-filtered",contrary+two+priority,missing=["gap_data"])
    add("partial-premise-priority",contrary+two+'prefer probe_data over gap_data reviewed "review/observations"\n')
    add("partial-assumption-rule-priority",base+'prefer calibration over primary_route reviewed "review/mixed-rule-priority"\n')
    add("priority-option-mismatch",contrary+two+priority,preference="minimum_rank")
    add("unknown-goal",goal="absent")
    add("invalid-programme",base.replace('method "structured/1"','method "unknown/1"'))
    for name,limits in [("premise-budget",{"formal_premises":3}),("rule-budget",{"formal_rules":3}),("antecedent-budget",{"formal_antecedents":1}),("construction-budget",{"formal_arguments":1}),("local-incomplete",{"composed_edges":1})]:
        add(name,attack,limits=limits)
    add("extension-budget",contrary+two,semantics="preferred",limits={"extension_search":1})
    add("unknown-option",semantics="unknown")
    support='''language "EAL/3"
environment lab {
require site == "bench"
}
tool collector {
version "1"
}
evidence observation {
tool collector
kind test
environment lab
max_age 60
require ok
}
reasoning relation {
method "structured/1"
rationale "An explicit bounded relation."
}
claim result {
statement "The imported bounded condition holds."
environment lab
}
argument route = [evidence observation] via relation => result
strict route reviewed "review/imported-implication"
'''
    add("imported-source-provenance",'language "EAL/3"\nimport "support.eal" as support\n',goal="support.result",imports={"support.eal":support})
    add("empty-snapshot",missing=["primary_data","probe_data","gap_data","calibration_data"])
    return result

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("reference",type=Path);parser.add_argument("--check",action="store_true");args=parser.parse_args()
    reference=args.reference.resolve()
    if subprocess.check_output(["git","-C",str(reference),"rev-parse","HEAD"],text=True).strip()!=REVISION:raise SystemExit("Reference checkout must be pinned at "+REVISION)
    if subprocess.check_output(["git","-C",str(reference),"status","--porcelain","--","src/eal","grammar","tests"],text=True):raise SystemExit("Reference source, grammar and tests must be unchanged")
    sys.path.insert(0,str(reference/"src"))
    from eal.aspic_compiler import compile_eal_aspic
    from eal.evaluator import canonical_digest,environment_context,environment_fingerprint
    from eal.limits import ExecutionLimits
    from eal.parser import parse
    def record(program,name,value,context):
        evidence=program.evidence[name];tool=program.tools[evidence.tool];local=environment_context(evidence.environment,context)
        acquisition={"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":local}
        return {"evidence_id":name,"source_digest":program.source_digest,"tool":tool.name,"tool_version":tool.version,"tool_binding_digest":"0"*64,"evidence_kind":evidence.kind,"environment":evidence.environment,"environment_fingerprint":environment_fingerprint(evidence.environment,local),"input_digest":canonical_digest(evidence.input),"collected_at":NOW,"input":evidence.input,"context":local,"acquisition_request":acquisition,"acquisition_request_digest":canonical_digest(acquisition),"request_digest":canonical_digest({"evidence_id":name,"environment":evidence.environment,**acquisition}),"run_id":"synthetic-compiler-run","status":"ok","value":value,"data_digest":canonical_digest(value)}
    captured=[]
    for case in cases(reference):
        entry=deepcopy(case);entry["source_sha256"]=hashlib.sha256(case["source"].encode()).hexdigest();limits=ExecutionLimits(**case.get("limits",{}))
        resolver=(lambda path:case["imports"][path])if"imports"in case else None
        stage="compile"
        try:
            programme=parse(case["source"],limits=limits,resolver=resolver)
            records={name:record(programme,name,{"ok":True,**case.get("extra_value",{})},case["context"])for name in programme.evidence if name not in case.get("missing",())}
            for name,changes in case.get("record_changes",{}).items():records[name].update(changes)
            entry["records"]=records
            compiled=compile_eal_aspic(case["source"],records,goal=case["goal"],now=case["now"],context=case["context"],binding_digests=case.get("binding_digests"),limits=limits,resolver=resolver,semantics=case.get("semantics","grounded"),query_mode=case.get("query_mode","sceptical"),preference=case.get("preference"))
            entry["theory"]=deepcopy(compiled.theory);entry["source_map"]=deepcopy(compiled.source_map);entry["assessment"]=deepcopy(compiled.assessment)
            stage="solve";entry["projection"]=compiled.to_dict()
        except (ValueError,TypeError,OverflowError,RecursionError,KeyError)as error:
            entry["error"]={"stage":stage,"message":str(error),"kind":type(error).__name__}
        captured.append(entry)
    result={"schema":"eal-aspic-compiler-reference/1","source_revision":REVISION,"provenance":"Synthetic witnesses from unmodified pinned aspic_compiler.py and its bounded worker; no experimental observations","comparison":"Exact theory, mapping, assessment and formal projection; only declared native backend implementation metadata and digests depending on that metadata are projected out and independently checked natively","cases":captured}
    output=json.dumps(result,ensure_ascii=False,allow_nan=False,indent=2)+"\n";destination=ROOT/"fixtures/aspic/compiler-reference.json"
    if args.check:
        if destination.read_text()!=output:raise SystemExit("ASPIC compiler reference drift")
    else:destination.write_text(output)
    print(json.dumps({"cases":len(captured),"compiled":sum("theory"in c for c in captured),"solved":sum("projection"in c for c in captured),"errors":[(c["name"],c["error"])for c in captured if"error"in c]}))

if __name__=="__main__":main()
