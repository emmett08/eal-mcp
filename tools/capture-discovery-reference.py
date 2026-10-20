#!/usr/bin/env python3
"""Freeze the pinned language guide, independently of native backend identities."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

REVISION="e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT=Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reference',type=Path);parser.add_argument('--check',action='store_true')
    args=parser.parse_args();reference=args.reference.resolve()
    if subprocess.check_output(['git','-C',str(reference),'rev-parse','HEAD'],text=True).strip()!=REVISION:raise SystemExit('Reference revision differs')
    if subprocess.check_output(['git','-C',str(reference),'status','--porcelain','--','src/eal','tests'],text=True):raise SystemExit('Reference source must remain unchanged')
    sys.path.insert(0,str(reference/'src'))
    from eal.discovery import describe_language
    value=describe_language()
    # These three members identify the selected implementation/installed registry.
    for name in ('implementation_version','method_registry_fingerprint','typed_bindings'):value.pop(name)
    result={'schema':'eal-discovery-reference/1','source_revision':REVISION,
            'source_sha256':hashlib.sha256((reference/'src/eal/discovery.py').read_bytes()).hexdigest(),
            'comparison':'Exact pinned language reference; native version, installed registry identity/bindings and capability admission are supplied by the Rust host',
            'language':value}
    output=json.dumps(result,ensure_ascii=False,allow_nan=False,indent=2)+'\n'
    path=ROOT/'fixtures/applications/discovery-reference.json'
    if args.check:
        if path.read_text()!=output:raise SystemExit('Discovery reference drift')
    else:path.parent.mkdir(parents=True,exist_ok=True);path.write_text(output)
    print(json.dumps({'reference_fields':len(value),'source_revision':REVISION}))

if __name__=='__main__':main()
