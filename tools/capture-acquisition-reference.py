#!/usr/bin/env python3
"""Capture synthetic acquisition contracts from the pinned read-only Python source.

Development only. Native tests consume the retained JSON without Python.
"""
from __future__ import annotations
import argparse
from copy import deepcopy
from dataclasses import asdict
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
SECRET = b"x" * 32


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = args.reference.resolve()
    if subprocess.check_output(["git","-C",str(reference),"rev-parse","HEAD"],text=True).strip() != REVISION:
        raise SystemExit("Reference must be pinned at " + REVISION)
    if subprocess.check_output(["git","-C",str(reference),"status","--porcelain","--","src/eal","tests"],text=True):
        raise SystemExit("Reference source and tests must remain unchanged")
    sys.path.insert(0,str(reference / "src"))
    from eal.tool_acquisition import ToolRegistry, keyed_digest, strict_json, validate_envelope
    from eal.evaluator import canonical_digest

    def observed(function):
        try: return {"value":function()}
        except (ValueError,TypeError,UnicodeError) as error:
            return {"error":{"type":type(error).__name__,"message":str(error)}}

    registry=[]
    command='[tools.reader]\nkind="command"\nversion="1"\nargv=["collector","--synthetic"]\n'
    file='[tools.reader]\nkind="json_file"\nversion="1"\npath="observation.json"\n'
    with tempfile.TemporaryDirectory(prefix="eal-acquisition-reference-") as directory:
        workspace=Path(directory)
        pinned=b"synthetic collector artefact\n"
        (workspace / "collector.bin").write_bytes(pinned)
        pin=hashlib.sha256(pinned).hexdigest()
        def capture(name,text):
            path=workspace / "tools.toml"
            path.write_text(text)
            entry={"name":name,"toml":text}
            def load():
                bindings=ToolRegistry.load(path).bindings
                return {name:{"binding":asdict(binding),"digest":binding.binding_digest(SECRET,workspace=workspace),
                              "environment_digest":binding.process_environment_digest(SECRET,effective_env={"TOKEN":"synthetic-secret","LANG":"C"})}
                        for name,binding in bindings.items()}
            entry.update(observed(load));registry.append(entry)
        capture("empty","")
        capture("command-default",command)
        capture("file-default",file)
        capture("parallel-scheduling-only",command+'parallel_safe=true\n')
        capture("empty-environment",command+'inherit_env=[]\n')
        capture("allowlist-forward",command+'inherit_env=["PATH","HOME"]\n')
        capture("allowlist-reverse",command+'inherit_env=["HOME","PATH"]\n')
        capture("environment-overlay",command+'[tools.reader.env]\nTOKEN="synthetic-secret"\n')
        capture("pinned-file",command+f'pinned_files=[{{path="collector.bin",sha256="{pin}"}}]\n')
        capture("timing-and-output",command+'timeout_seconds=0.125\nmax_output_bytes=17\n')
        capture("integer-timeout",command+'timeout_seconds=1\n')
        capture("maximum-bounds",command+'timeout_seconds=3600\nmax_output_bytes=16777216\n')
        capture("file-environment-overlay",file+'[tools.reader.env]\nTOKEN="synthetic-unused-file-setting"\n')
        for name,text in [
            ("unknown-root",'[other]\nx=1\n'),("invalid-tools",'tools=[]\n'),("invalid-binding",'[tools]\nreader=1\n'),
            ("unknown-setting",command+'obsolete=true\n'),("missing-kind",command.replace('kind="command"\n','')),
            ("unknown-kind",command.replace('kind="command"','kind="shell"')),
            ("missing-version",command.replace('version="1"\n','')),("empty-version",command.replace('version="1"','version=""')),
            ("integer-version",command.replace('version="1"','version=1')),
            ("argv-string",command.replace('["collector","--synthetic"]','"collector"')),
            ("argv-empty-item",command.replace('["collector","--synthetic"]','[""]')),
            ("argv-boolean",command.replace('["collector","--synthetic"]','[true]')),
            ("command-empty-argv",command.replace('["collector","--synthetic"]','[]')),
            ("command-path",command+'path="forbidden"\n'),("file-missing-path",file.replace('path="observation.json"\n','')),
            ("file-argv",file+'argv=["collector"]\n'),("file-path-number",file.replace('path="observation.json"','path=1')),
        ]:capture(name,text)
        for field,values in [
            ("timeout_seconds",["true","0","-1","3601","nan","inf",'"30"']),
            ("max_output_bytes",["true","0","16777217","1.0",'"1"']),
            ("parallel_safe",["1",'"true"']),
            ("env",['[]','{TOKEN=1}']),
            ("inherit_env",['"PATH"','["PATH","PATH"]','["INVALID-NAME"]','[1]']),
            ("pinned_files",['{}','[{path="collector.bin"}]','[{path="",sha256="'+pin+'"}]',
                              '[{path="collector.bin",sha256="'+pin.upper()+'"}]',
                              '[{path="collector.bin",sha256="'+pin+'",extra=true}]']),
        ]:
            for i,value in enumerate(values):capture(f"invalid-{field}-{i}",command+f'{field}={value}\n')
        capture("file-inherit-env",file+'inherit_env=[]\n')
        capture("file-pin",file+f'pinned_files=[{{path="collector.bin",sha256="{pin}"}}]\n')
        capture("too-many-inherited-variables",command+'inherit_env='+json.dumps([f'VAR{i}' for i in range(65)])+'\n')
        capture("duplicate-pins",command+'pinned_files=['+','.join([f'{{path="collector.bin",sha256="{pin}"}}']*2)+']\n')
        capture("too-many-pins",command+'pinned_files=['+','.join(f'{{path="collector{i}.bin",sha256="{pin}"}}' for i in range(33))+']\n')

    digests=[]
    for i,value in enumerate([None,True,1,1.0,-0.0,{"z":1,"a":"é🧪"},[2**80,0.1],{"TOKEN":"synthetic-secret"}]):
        for domain in (b"tool-binding",b"process-environment"):
            digests.append({"name":f"value-{i}-{domain.decode()}","value":value,"domain":domain.decode(),
                            "secret_hex":SECRET.hex(),"digest":keyed_digest(SECRET,domain,value)})
    strict=[]
    texts=['null','true','123456789012345678901234567890','1.0','-0','-0.0','1e-9999','"é🧪"',
           '{"$serde_json::private::Number":"literal","$serde_json::private::RawValue":"literal"}',
           '{"a":1,"\\u0061":2}','NaN','Infinity','-Infinity','1e309','{"n":-1e309}',
           '"\\ud800"','"\\udc00"','"\\ud83e\\uddea"','[1,]','{} trailing','{"a":1, "a":2}',
           '"NaN Infinity -Infinity"']
    for depth in (127,128,129):texts.append('['*depth+'0'+']'*depth)
    for i,text in enumerate(texts):
        entry={"name":f"strict-{i}","text":text,**observed(lambda t=text:strict_json(t))}
        if text.startswith('['*100) and 'value' in entry:
            entry['encoded_value']=json.dumps(entry.pop('value'),ensure_ascii=False,separators=(',',':'))
        strict.append(entry)

    context={"site":"bench"}
    acquisition={"tool":"reader","tool_version":"1","input":{"sample":1},"context":context}
    envelopes=[]
    def envelope(name,value,file_import=False):
        text=json.dumps(value,ensure_ascii=False,allow_nan=False)
        envelopes.append({"name":name,"text":text,"file_import":file_import,"context":context,"acquisition":acquisition,
                          **observed(lambda:validate_envelope(text.encode(),file_import=file_import,context=context,acquisition=acquisition))})
    envelope("command-value",{"value":{"passed":True}})
    envelope("command-details",{"value":False,"details":{"literal":"span"},"observed_at":"2026-09-23T12:00:00Z"})
    complete={"value":{"passed":True},"observed_at":"2026-09-23T12:00:00Z","context":context,"request":acquisition}
    envelope("file-complete",complete,True)
    envelope("command-correspondence",complete)
    for field in ("observed_at","context","request"):
        changed=deepcopy(complete);changed.pop(field);envelope("file-missing-"+field,changed,True)
    for name,value in [("not-object",[]),("missing-value",{}),("extra-property",{"value":True,"status":"supported"}),
                       ("wrong-request",{"value":True,"request":{}}),("request-not-object",{"value":True,"request":[]}),
                       ("wrong-context",{"value":True,"context":{"site":"field"}}),("context-not-object",{"value":True,"context":[]})]:envelope(name,value)
    changed=deepcopy(complete);changed['request']['input']['sample']=1.0;envelope("request-integer-float-distinct",changed,True)
    requests=[]
    for value in [acquisition,{"z":"é🧪\u007f\n","a":-0.0},[2**80,1e-7,True]]:
        requests.append({"value":value,"encoded":json.dumps(value,sort_keys=True,allow_nan=False)+'\n'})
    result={"schema":"eal-acquisition-reference/1","source_revision":REVISION,
            "provenance":"Synthetic configuration, envelope and keyed-identity witnesses; no measurements or real credentials",
            "comparison":"Exact registry values/errors, HMAC identities, valid JSON values, envelopes and request bytes; malformed JSON compares exception category because parser syntax wording is native",
            "secret_hex":SECRET.hex(),"pinned_files":{"collector.bin":pinned.decode()},
            "registry":registry,"digests":digests,"strict_json":strict,"envelopes":envelopes,"requests":requests}
    output=json.dumps(result,ensure_ascii=False,allow_nan=False,indent=2)+'\n'
    path=ROOT/'fixtures/runtime/acquisition-reference.json'
    if args.check:
        if path.read_text()!=output:raise SystemExit('Acquisition reference drift')
    else:path.parent.mkdir(parents=True,exist_ok=True);path.write_text(output)
    print(json.dumps({key:len(result[key]) for key in ('registry','digests','strict_json','envelopes','requests')}))


if __name__=='__main__':main()
