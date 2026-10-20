#!/usr/bin/env python3
"""Retain pure host/client/redaction behavior from the pinned Python source (development only)."""
from __future__ import annotations
import argparse
import hashlib
from importlib.metadata import version
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser();parser.add_argument("source",type=Path);parser.add_argument("--output",type=Path,default=ROOT/"fixtures/host/reference.json");args=parser.parse_args()
    assert subprocess.check_output(["git","-C",str(args.source),"rev-parse","HEAD"],text=True).strip()==REVISION
    for package,pin in {"fastmcp":"4.0.10","mcp":"2.2.0","jsonschema":"4.23.0"}.items():assert version(package)==pin
    sys.path.insert(0,str(args.source/"src"))
    from eal.host import parse_request
    from eal.client_transports import validate_http_url
    from eal.host_redaction import sanitise_response
    requests=[
        'not JSON','[]','{"operation":"exec","argv":["echo"]}',
        '{"operation":"validate","source":"","argv":["echo"]}',
        '{"operation":"validate"}','{"operation":"validate","source":4}',
        '{"operation":"validate","source":"","source":"different"}',
        '{"operation":"collect","source":"","context":[]}',
        '{"operation":"collect","source":"","context":{"number":1e309}}',
        '{"operation":"collect_claim","source":"","context":{},"claim":""}',
        '{"operation":"sources","limit":true}','{"operation":"sources","limit":1.0}',
        '{"operation":"grounded","arguments":null,"attacks":[]}',
        '{"operation":"grounded","arguments":[],"attacks":[["a"]]}',
        '{"operation":"compile_aspic","source":"","context":{},"collection_id":"saved","goal":null}',
        '{"operation":"compile_aspic","source":"","context":{},"collection_id":"saved","goal":" "}',
        '{"operation":"compile_aspic","source":"","context":{},"collection_id":null,"goal":"ready"}',
        '{"operation":"compile_aspic","source":"","context":{},"collection_id":"saved","goal":"ready","semantics":[]}',
        '{"operation":"describe"}','{"operation":"plan","source":"source","claim":"works"}',
        '{"operation":"collect","source":"source","context":{"$serde_json::private::Number":"literal"},"evidence_ids":null}',
        '{"operation":"sources","limit":'+str(10**400)+'}',
        '{"operation":"compile_aspic","source":"source","context":{},"collection_id":"saved","goal":"ready","semantics":"grounded"}',
        '{"operation":"grounded","arguments":["a","b"],"attacks":[["a","b"]]}',
    ]
    def record(function,value):
        try:return {"input":value,"output":function(value)}
        except (ValueError,TypeError) as error:return {"input":value,"error":str(error)}
    urls=["http://localhost/mcp","http://127.0.0.1/mcp","http://[::1]:8000/mcp","https://engineering.example/mcp","HTTP://LOCALHOST:80/a?", "http://localhost/mcp?#","stdio://localhost","https:///mcp","https://user:secret@host/mcp","http://@localhost/mcp","https://host/mcp?token=x","https://host/mcp#secret","http://host:65536/mcp","http://host:0/mcp","http://bad host/mcp"]
    redactions=[]
    def redact(token,response):
        entry={"token":token,"input":response}
        try:entry["output"]=sanitise_response(response,token)
        except ValueError as error:entry["error"]=str(error)
        redactions.append(entry)
    for token in ["synthetic-secret",'opaque\\"secret',"result","content","type","text","_meta","redacted","[redacted]","[","a"]:
        payload={"nested":[{token:"first","prefix-"+token+"-suffix":token}]}
        redact(token,{"operation":"validate","protocol_version":"2025-11-25","is_error":False,"result":payload,"content":[{"type":"text","text":json.dumps(payload),"_meta":{token:token}}]})
    token="synthetic-secret"
    redact(token,{"result":{"encoded":json.dumps(json.dumps({token:token}))}})
    redact(token,{"content":[{"type":"text","text":'{"\\u0073\\u0065\\u0063\\u0072\\u0065\\u0074":true,'}]})
    redact(token,{"result":{token:"first","[redacted]":"second"}})
    redact(token,{"result":json.dumps({token:"first","[redacted]":"second"})})
    redact(token,{"content":[{"type":"text","text":'{\n  "result": true\n}'}]})
    redact("a[redacted]b",{"result":{"aa[redacted]bb":"aa[redacted]bb"}})
    redact(None,{"result":{"key":"value"},"content":[{"type":"text","text":"{invalid"}]})
    for kind in ["text","image","audio","resource_link","resource"]:
        redact("meta",{"content":[{"type":kind,"meta":{"meta":"meta"},"mimeType":"meta","remote-meta":"meta"}]})
    files=["host.py","host_redaction.py","client_transports.py","credentials.py","server_settings.py","operation_contracts.py"]
    result={"source_revision":REVISION,"provenance":{"framework_pins":{package:version(package) for package in ["fastmcp","mcp","jsonschema"]},"source_files_sha256":{"src/eal/"+name:hashlib.sha256((args.source/"src/eal"/name).read_bytes()).hexdigest() for name in files},"normalization":"No normalization of retained pure helper results; native JSON object member order is semantically ignored."},"requests":[record(parse_request,value) for value in requests],"urls":[record(validate_http_url,value) for value in urls],"redactions":redactions}
    if subprocess.check_output(["git","-C",str(args.source),"status","--porcelain","--untracked-files=no"],text=True):raise ValueError("Reference checkout changed during capture")
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n")
if __name__=="__main__":main()
