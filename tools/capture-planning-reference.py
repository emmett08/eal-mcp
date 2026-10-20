#!/usr/bin/env python3
"""Retain source-order claim collection plans from the pinned Python implementation."""
from __future__ import annotations
import argparse
import ast
from dataclasses import asdict
import json
from pathlib import Path
import subprocess
import sys

REVISION='e2d6f346a37f92483b9f9a13923b98feb053447b'
ROOT=Path(__file__).resolve().parents[1]

def capture(reference):
    if subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()!=REVISION:
        raise SystemExit('Reference checkout does not match the frozen revision')
    if subprocess.check_output(['git','status','--porcelain','--','src/eal','grammar'],cwd=reference,text=True):
        raise SystemExit('Reference source must be unchanged')
    sys.path.insert(0,str(reference/'src'))
    from eal.parser import parse
    from eal.planning import EvidencePlanner
    from eal.semantics import validate
    module=ast.parse((reference/'tests/test_planning.py').read_text())
    source=next(ast.literal_eval(node.value) for node in module.body if isinstance(node,ast.Assign) and any(isinstance(target,ast.Name) and target.id=='SOURCE' for target in node.targets))
    variants={
        'base':source,
        'circular-objection':source.replace('objection grounds = [premises attacker] -x> claim root','objection grounds = [premises root] -x> claim root'),
        'pattern-identity':source.replace('argument alternative_route = [evidence alternative] via inferred => root','pattern support(c: claim, r: reasoning, e: evidence) = [evidence e] via r => c\n\napply alternative_route=support(c=root, r=inferred, e=alternative)'),
        'cyclic-defences':source+'\nobjection mutual_a = [evidence challenge] -x> objection mutual_b\n\nobjection mutual_b = [evidence defence] -x> objection mutual_a\n\nobjection link = [evidence reply] -x> objection mutual_a\n',
    }
    cases=[]
    for label,source in variants.items():
        program=parse(source)
        if validate(program):
            raise SystemExit(f'Invalid planning reference source: {label}')
        planner=EvidencePlanner(program)
        for claim in ['root','leaf','attacker','other','foreign_claim','missing']:
            case={'name':f'{label}-{claim}','source':source,'claim':claim}
            try:
                plan=planner.plan(claim)
                case['result']={'claim':plan.claim,'closure':{key:sorted(value) for key,value in asdict(plan.closure).items()},'calls':[asdict(call) for call in plan.calls]}
                case['evidence_ids']=plan.evidence_ids
                case['estimated_calls']=plan.estimated_calls
            except ValueError as exc:
                case['error']=str(exc)
            cases.append(case)
    return {'schema':'eal-rust-planning-oracle/1','source_revision':REVISION,'cases':cases}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reference',type=Path)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    target=ROOT/'fixtures/planning/reference.json'
    content=json.dumps(capture(args.reference.resolve()),ensure_ascii=False,indent=2,allow_nan=False)+'\n'
    if args.check:
        if target.read_text()!=content:raise SystemExit('Planning reference fixture drifted')
    else:
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_text(content)
if __name__=='__main__':main()
