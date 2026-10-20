#!/usr/bin/env python3
"""Capture bounded graph-solver oracles from the unmodified pinned EAL source.

Development only. Normal Rust tests consume retained JSON without Python.
"""
from __future__ import annotations
import argparse
from contextlib import nullcontext
from dataclasses import replace
import importlib
import json
from pathlib import Path
import random
import subprocess
import sys
import unicodedata

REVISION = 'e2d6f346a37f92483b9f9a13923b98feb053447b'
ROOT = Path(__file__).resolve().parents[1]

def capture(reference):
    if subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=reference, text=True).strip() != REVISION:
        raise SystemExit('Reference checkout does not match the frozen revision')
    if subprocess.check_output(['git', 'status', '--porcelain', '--', 'src/eal', 'grammar'], cwd=reference, text=True):
        raise SystemExit('Reference production source must be unchanged')
    sys.path.insert(0, str(reference / 'src'))
    from eal.dialectic import solve_grounded, solve_composed
    from eal.limits import ExecutionLimits, using_limits
    cases = []
    def add(name, operation, request, limits=None):
        with using_limits(replace(ExecutionLimits(), **limits)) if limits else nullcontext():
            try:
                output = solve_grounded(request['arguments'], request['attacks']) if operation == 'grounded' else solve_composed(request['nodes'], request['claims'], request['attacks'])
                case = {'name': name, 'operation': operation, 'request': request, 'result': output}
            except ValueError as exc:
                case = {'name': name, 'operation': operation, 'request': request, 'error': str(exc)}
        if limits:
            case['limits'] = limits
        cases.append(case)
    add('empty', 'grounded', {'arguments': [], 'attacks': []})
    add('reinstatement', 'grounded', {'arguments': ['c','b','a'], 'attacks': [['b','a'],['a','b'],['c','b']]})
    add('all-defenders-required', 'grounded', {'arguments': list('abcde'), 'attacks': [['a','b'],['b','e'],['c','d'],['d','c'],['d','e']]})
    rng = random.Random(781241)
    for size in range(1, 8):
        nodes = list('abcdefg'[:size])
        for repeat in range(6):
            add(f'random-{size}-{repeat}', 'grounded', {'arguments': nodes, 'attacks': [[a,b] for a in nodes for b in nodes if rng.random() < .3]})
    for index, (arguments, attacks) in enumerate([
        (None,[]), ([],{}), (['a','a'],[]), ([1],[]), ([True],[]), ([''],[]), ([' a'],[]), (['a\nb'],[]), (['\u200d'],[]), (['\ue000'],[]), (['\u0378'],[]), (['a\u00a0b'],[]), (['a'*257],[]), (['sensor reading','α','a'*256],[]), (['a'],[['a']]), (['a'],[['a','b']]), (['a'],[['a','a'],['a','a']]), (['a'],[['a',None]]), (['a'],['aa']),
    ]):
        add(f'invalid-grounded-{index}', 'grounded', {'arguments': arguments, 'attacks': attacks})
    add('argument-budget','grounded',{'arguments':['a','b'],'attacks':[]},{'declarations':1})
    add('attack-budget','grounded',{'arguments':['a'],'attacks':[['a','a'],['a','a']]},{'abstract_attacks':1})
    add('and-or-alternatives','composed',{'nodes':{'a':{'usable':True,'premises':[]},'b':{'usable':True,'premises':[]},'dependent':{'usable':True,'premises':['c']},'attack':{'usable':True,'premises':[]}},'claims':{'c':['a','b'],'d':['dependent']},'attacks':[['attack','a']]})
    add('unsupported-cycle','composed',{'nodes':{'a':{'usable':True,'premises':['b']},'b':{'usable':True,'premises':['a']}},'claims':{'a':['a'],'b':['b']},'attacks':[]})
    add('support-attack-cycle','composed',{'nodes':{'a':{'usable':True,'premises':[]},'objection':{'usable':True,'premises':['c']}},'claims':{'c':['a']},'attacks':[['objection','a']]})
    add('missing-derivation','composed',{'nodes':{'a':{'usable':True,'premises':['missing']}},'claims':{'missing':[]},'attacks':[]})
    add('defeat-defence','composed',{'nodes':{'a':{'usable':True,'premises':[]},'objection':{'usable':True,'premises':['c']},'defence':{'usable':True,'premises':[]}},'claims':{'c':['a']},'attacks':[['objection','a'],['defence','objection']]})
    invalid = [
        {'nodes':[],'claims':{},'attacks':[]},
        {'nodes':{'n':{'usable':1,'premises':[]}},'claims':{},'attacks':[]},
        {'nodes':{'n':{'usable':True,'premises':[],'extra':0}},'claims':{},'attacks':[]},
        {'nodes':{'n':{'usable':True,'premises':['missing']}},'claims':{},'attacks':[]},
        {'nodes':{'n':{'usable':True,'premises':[]}},'claims':{'c':['n','n']},'attacks':[]},
        {'nodes':{'n':{'usable':True,'premises':['c','c']}},'claims':{'c':['n']},'attacks':[]},
        {'nodes':{'n':{'usable':True,'premises':[]}},'claims':{},'attacks':[['n','missing']]},
        {'nodes':{'n':{'usable':True,'premises':[]}},'claims':{},'attacks':[['n','n'],['n','n']]},
    ]
    for index, request in enumerate(invalid):
        add(f'invalid-composed-{index}','composed',request)
    add('composed-edge-budget','composed',{'nodes':{'n':{'usable':True,'premises':['c']}},'claims':{'c':['n']},'attacks':[]},{'composed_edges':1})
    return {'schema':'eal-rust-graph-oracle/1','source_revision':REVISION,'python_unicode_version':unicodedata.unidata_version,'cases':cases}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reference',type=Path)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    target=ROOT/'fixtures/evaluation/dialectic.json'
    content=json.dumps(capture(args.reference.resolve()),ensure_ascii=False,indent=2,allow_nan=False)+'\n'
    if args.check:
        if target.read_text()!=content:
            raise SystemExit('Graph reference fixture drifted')
    else:
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_text(content)
if __name__=='__main__':
    main()
