#!/usr/bin/env python3
"""Capture catalogue metadata and checked admission from the unchanged pinned source.

Only the wall clock is fixed. Native fixtures compare every portable field and
independently check backend identities; no collectors or method callbacks run.
"""
from __future__ import annotations
import argparse
import ast
from copy import deepcopy
from datetime import datetime,timedelta,timezone
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from unittest.mock import patch

REVISION="e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT=Path(__file__).resolve().parents[1]
def source(reference):
    for node in ast.parse((reference/"tests/test_catalogue.py").read_text()).body:
        if isinstance(node,ast.Assign)and any(isinstance(t,ast.Name)and t.id=="SOURCE"for t in node.targets):return ast.literal_eval(node.value)
    raise ValueError("No pinned catalogue source")
def cases(reference):
    base=source(reference);result=[]
    def add(name,steps,files=None):result.append({"name":name,"files":files or{"source.eal":base},"steps":steps})
    reg={"op":"register","path":"source.eal","context":{"site":"bench"},"claims":["ready"],"label":"Readiness","save":"first"}
    add("register-search-reload",[reg,{"op":"register","path":"source.eal","context":{"site":"bench"},"claims":["ready"],"label":"Readiness"},{"op":"reload"},{"op":"get","entry_id":"source.eal","include_source":True},{"op":"list"},{"op":"find","query":"readiness","claim":"ready","context":{"site":"bench"}},{"op":"find","query":"service is ready"},{"op":"find","claim":"missing"},{"op":"find","context":{"site":"field"}}])
    changed=base.replace("The service is ready.","The bounded revised service is ready.")
    add("immutable-revisions",[reg,{"op":"write","path":"source.eal","source":changed},{"op":"get","entry_id":"source.eal","include_source":True,"save":"second"},{"op":"get_revision","entry_id":"source.eal","source_digest":"$first.source_digest","include_source":True},{"op":"revisions","entry_id":"source.eal"},{"op":"write","path":"source.eal","source":changed.replace("claim ready","claim gone").replace("=> ready","=> gone")},{"op":"get","entry_id":"source.eal"},{"op":"get_revision","entry_id":"source.eal","source_digest":"$second.source_digest"},{"op":"write","path":"source.eal","source":"invalid source"},{"op":"get","entry_id":"source.eal"},{"op":"get_cached","entry_id":"source.eal"}])
    add("nested-default-refresh-source-bug",[{"op":"register","path":"nested/source.eal","save":"first"},{"op":"get","entry_id":"nested/source.eal","include_source":True,"native_fix":"existing_generated_id_refresh"}],{"nested/source.eal":base})
    add("nested-explicit-id",[{"op":"register","path":"nested/source.eal","entry_id":"readiness","context":{"site":"bench"}},{"op":"get","entry_id":"readiness","include_source":True}],{"nested/source.eal":base})
    add("fixed-metadata",[reg,{"op":"register","path":"source.eal","label":"Changed","context":{"site":"bench"},"claims":["ready"]},{"op":"register","path":"source.eal","entry_id":"different"},{"op":"list"}])
    for name,options in [("empty-id",{"entry_id":""}),("path-id",{"entry_id":"a/b"}),("numeric-id",{"entry_id":"1abc"}),("long-id",{"entry_id":"a"*129}),("blank-label",{"label":"\u001c\u001d\u001e\u001f"}),("long-label",{"label":"Δ"*129}),("unicode-label",{"label":"Δ"*128}),("context-array",{"context":[]}),("context-large",{"context":{"padding":"x"*8192}}),("context-exact-byte-limit",{"context":{"padding":"x"*8178}}),("empty-claims",{"claims":[]}),("duplicate-claims",{"claims":["ready","ready"]}),("missing-claims",{"claims":["z_absent","a_absent"]})]:add(name,[{"op":"register","path":"source.eal",**options}])
    add("input-controls",[{"op":"register","path":"/source.eal"},{"op":"register","path":"../outside.eal"},{"op":"register","path":"source.txt"},{"op":"list","limit":0},{"op":"list","offset":10001},{"op":"find","query":"x"*257},{"op":"find","context":[]},{"op":"get_revision","entry_id":"absent","source_digest":"A"*64},{"op":"get_revision","entry_id":"absent","source_digest":"a"*64,"method_registry_fingerprint":"bad"},{"op":"get","entry_id":"absent"},{"op":"revisions","entry_id":"absent"},{"op":"register_tree","directory":"/outside"},{"op":"register_tree","limit":0}])
    add("bounded-discovery",[{"op":"register_tree","directory":"nested","context":{"site":"bench"}},{"op":"register_tree","directory":"nested","limit":1}],{"nested/good.eal":base,"nested/bad.eal":"invalid source","nested/ignored.txt":base})
    imported='language "EAL/3"\nimport "support.eal" as support\nclaim ready {\n statement "The bounded condition holds."\n environment support.lab\n}\n'
    support='language "EAL/3"\nenvironment lab {\n require site == "bench"\n}\n'
    add("import-bundle-revision",[{"op":"register","path":"source.eal","save":"first"},{"op":"write","path":"support.eal","source":support.replace("bench","field")},{"op":"get","entry_id":"source.eal","include_source":True},{"op":"get_revision","entry_id":"source.eal","source_digest":"$first.source_digest","include_source":True}],{"source.eal":imported,"support.eal":support})
    add("historical-source-context-and-selected-claims",[reg,{"op":"put","kind":"collection","record_id":"collection-good","payload":{"source_digest":"$first.source_digest","context":{"site":"bench"},"records":{"observation":{"value":{"secret":"not catalogue metadata"}}}}},{"op":"put","kind":"collection","record_id":"collection-other","payload":{"source_digest":"$first.source_digest","context":{"site":"field"},"records":{}}},{"op":"put","kind":"assessment","record_id":"assessment-good","payload":{"source_digest":"$first.source_digest","context_fingerprint":{"digest":{"site":"bench"}},"method_registry_fingerprint":"$first.method_registry_fingerprint","collection_id":"collection-good","assessed_at":"2040-01-01T00:00:00Z","claims":{"ready":{"status":"supported"},"unselected":{"status":"contested"}}}},{"op":"put","kind":"assessment","record_id":"assessment-other","payload":{"source_digest":"$first.source_digest","context_fingerprint":"other","claims":{"ready":{"status":"unsupported"}}}},{"op":"runs","entry_id":"source.eal"}])
    add("literal-context-identity-keys",[{"op":"register","path":"source.eal","context":{"registered_at":"literal","method_registry_fingerprint":"literal","span":{"line":7},"contract":{"identity_basis":"literal"}}},{"op":"find","context":{"registered_at":"literal","method_registry_fingerprint":"literal","span":{"line":7},"contract":{"identity_basis":"literal"}}}])
    return result

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("reference",type=Path);parser.add_argument("--check",action="store_true");args=parser.parse_args();reference=args.reference.resolve()
    if subprocess.check_output(["git","-C",str(reference),"rev-parse","HEAD"],text=True).strip()!=REVISION:raise SystemExit("Reference checkout must be pinned at "+REVISION)
    if subprocess.check_output(["git","-C",str(reference),"status","--porcelain","--","src/eal","grammar","tests"],text=True):raise SystemExit("Reference source, grammar and tests must remain unchanged")
    sys.path.insert(0,str(reference/"src"))
    from eal.catalogue import WorkspaceKnowledgeCatalogue
    from eal.runtime import ReasoningService
    from eal.evaluator import canonical_digest
    captured=[]
    for case in cases(reference):
        case=deepcopy(case)
        with TemporaryDirectory() as directory:
            workspace=Path(directory)
            for path,text in case["files"].items():target=workspace/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text)
            ticks=iter(range(10000));clock=lambda:(datetime(2040,1,1,tzinfo=timezone.utc)+timedelta(microseconds=next(ticks))).isoformat(timespec="microseconds").replace("+00:00","Z")
            service=ReasoningService(workspace);catalogue=WorkspaceKnowledgeCatalogue(service);saved={}
            def resolve(value):
                if isinstance(value,str)and value.startswith("$"):name,field=value[1:].split(".",1);return saved[name][field]
                if isinstance(value,dict):
                    if set(value)=={"digest"}:return canonical_digest(value["digest"])
                    return {k:resolve(v)for k,v in value.items()}
                if isinstance(value,list):return[resolve(v)for v in value]
                return value
            with patch("eal.catalogue.utc_now",clock),patch("eal.store.utc_now",clock):
                for step in case["steps"]:
                    call=resolve({key:value for key,value in step.items()if key not in("expected","save","native_fix")});op=call.pop("op")
                    try:
                        if op=="write":(workspace/call["path"]).write_text(call["source"]);output=None
                        elif op=="reload":service=ReasoningService(workspace);catalogue=WorkspaceKnowledgeCatalogue(service);output=None
                        elif op=="put":output=service.store.put(**call)
                        elif op=="get_cached":output=catalogue.get(**call,refresh=False)
                        else:output=getattr(catalogue,op)(**call)
                        step["expected"]={"value":output}
                        if"save"in step:saved[step["save"]]=deepcopy(output)
                    except (ValueError,TypeError,KeyError,OSError)as error:step["expected"]={"error":str(error),"kind":type(error).__name__}
        captured.append(case)
    output=json.dumps({"schema":"EAL/catalogue-reference/1","source_revision":REVISION,"comparison":"Every portable metadata field; wallclock timestamps and backend method-registry fingerprints are independently checked and normalized only at known result fields. Native intentionally fixes refreshing previously generated path entry IDs.","cases":captured},ensure_ascii=False,allow_nan=False,indent=2)+"\n";path=ROOT/"fixtures/runtime/catalogue-reference.json"
    if args.check:
        if path.read_text()!=output:raise SystemExit("Catalogue reference drift")
    else:path.parent.mkdir(parents=True,exist_ok=True);path.write_text(output)
    print(json.dumps({"cases":len(captured),"steps":sum(len(c["steps"])for c in captured)}))
if __name__=="__main__":main()
