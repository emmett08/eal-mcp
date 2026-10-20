#!/usr/bin/env python3
"""Development-only static compatibility check against a supplied pinned consumer.

Requires Python jsonschema and Node >=24. No network, browser, Ajv or live bridge
is used. Normal Rust tests do not invoke this script or require these packages.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
CONSUMER_REVISION = "a38bff4a193634556ef81e64f9b7823a1b1de6ec"
CONSUMER_BLOB = "16512f96824d5b218e5b790b79777a6f16217d21"
MAX_BYTES, MAX_ARGUMENTS, MAX_DEFEATS = 4 * 1024 * 1024, 4096, 65536


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def fixture_cases():
    explicit = [case for case in load(ROOT / "fixtures/aspic/reference.json")["cases"]
                if case["solve"]["ok"]]
    compiled = [case for case in load(ROOT / "fixtures/aspic/compiler-reference.json")["cases"]
                if "error" not in case]
    return explicit, compiled


def checked_consumer(path: Path) -> str:
    content = path.read_bytes()
    blob = hashlib.sha1(b"blob " + str(len(content)).encode() + b"\0" + content).hexdigest()
    if blob != CONSUMER_BLOB:
        raise ValueError(f"Expected pinned consumer blob {CONSUMER_BLOB}, found {blob}")
    source = content.decode("utf-8")
    # These two functions contain the consumer's additional graph checks. Their
    # bodies are copied byte-for-byte; only dependency-free module framing is added.
    helper = source[source.index("export class ViewValidationError"):source.index("const encoder")]
    graph = source[source.index("export function groundedLabels"):
                   source.index("export function parseAspicView")]
    return helper + graph + "\nexport { validateGraph };\n"


def generated_views(binary: Path, work: Path, explicit: list, compiled: list) -> list:
    work.mkdir()
    environment = dict(os.environ, PATH="")

    def put(name: str, value):
        path = (work / name).resolve()
        if not path.is_relative_to(work.resolve()):
            raise ValueError(f"Fixture path escapes temporary workspace: {name!r}")
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value if isinstance(value, str) else json.dumps(value), encoding="utf-8")

    def run(*arguments):
        result = subprocess.run([str(binary), "--workspace", str(work), *arguments],
                                capture_output=True, text=True, env=environment, timeout=30)
        if result.returncode:
            raise ValueError(f"Native command {arguments!r} failed ({result.returncode}): "
                             + (result.stderr or result.stdout))
        return result.stdout

    def flags(case):
        limits = case.get("limits", {})
        if not limits:
            return []
        put("limits.toml", "[limits]\n" + "".join(f"{key} = {value}\n" for key, value in limits.items()))
        return ["--limits", "limits.toml"]

    def export(name, result, selected):
        put("result.json", result)
        output = run(*selected, "export-aspic", "result.json")
        if len(output.encode("utf-8")) > MAX_BYTES:
            raise ValueError(f"{name}: emitted JSON file exceeds consumer import policy")
        return {"name": name, "view": json.loads(output)}

    views = []
    for case in explicit:
        selected = flags(case)
        put("theory.json", case["payload"])
        result = json.loads(run(*selected, "aspic", "theory.json"))
        views.append(export("explicit-" + case["name"], result, selected))
    for case in compiled:
        selected = flags(case)
        put("source.eal", case["source"])
        put("records.json", case["records"])
        for name, source in case.get("imports", {}).items():
            put(name, source)
        options = []
        for key, option in [("semantics", "--semantics"), ("query_mode", "--query-mode"),
                            ("preference", "--preference")]:
            if case.get(key):
                options.extend([option, case[key]])
        result = json.loads(run(*selected, "compile-aspic", "source.eal", "--claim", case["goal"],
                                "--now", case["now"], "--context", json.dumps(case["context"]),
                                "--records", "records.json", *options))
        # The development CLI does not accept current binding-digest overrides.
        # These runs test emitted-view compatibility, not differential equality.
        views.append(export("compiled-" + case["name"], result, selected))
    return views


def check_schema_and_policy(views: list, explicit: list, compiled: list):
    expected = {"explicit-" + case["name"] for case in explicit}
    expected.update("compiled-" + case["name"] for case in compiled)
    names = [case["name"] for case in views]
    if len(set(names)) != len(names) or set(names) != expected:
        raise ValueError("Views do not cover exactly the current successful retained fixture cases")
    schema = load(ROOT / "contracts/schemas/aspic-view.schema.json")
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema)
    for case in views:
        view = case["view"]
        validator.validate(view)
        # Equivalent count policies in validateAspicView, independently of its
        # unchanged graph functions. A host may select larger producer budgets.
        if len(json.dumps(view, ensure_ascii=False, separators=(",", ":")).encode()) > MAX_BYTES:
            raise ValueError(f"{case['name']}: view exceeds consumer byte policy")
        if len(view["arguments"]) > min(MAX_ARGUMENTS, view["execution_limits"]["formal_arguments"]):
            raise ValueError(f"{case['name']}: graph exceeds consumer argument policy")
        if len(view["defeats"]) > min(MAX_DEFEATS, view["execution_limits"]["formal_defeats"]):
            raise ValueError(f"{case['name']}: graph exceeds consumer defeat policy")


GRAPH_RUNNER = r"""
import {readFileSync} from 'node:fs';
import {validateGraph} from './graph.ts';
const cases=JSON.parse(readFileSync(process.argv[2],'utf8'));
for(const item of cases){
  try{validateGraph(item.view);}
  catch(error){throw new Error(`${item.name}: ${error.message}`);}
}
const base=cases.find(item=>item.view.defeats.length>0 &&
  item.view.arguments.some(a=>a.top==='defeasible')).view;
const rejected=[];
for(const [name,change] of [
  ['duplicate IDs',v=>v.arguments.push(structuredClone(v.arguments[0]))],
  ['wrong strength',v=>v.arguments[0].strength=0],
  ['wrong grounded label',v=>v.arguments[0].label=v.arguments[0].label==='in'?'out':'in'],
  ['duplicate witness',v=>v.defeats.push(structuredClone(v.defeats[0]))],
  ['wrong query status',v=>v.query_status=v.query_status==='accepted'?'rejected':'accepted'],
  ['unknown extension member',v=>v.extensions[0].push('unknown')],
  ['derivation cycle',v=>{const a=v.arguments.find(a=>a.top==='defeasible');a.direct_subarguments=[a.id];}]
]){
  const view=structuredClone(base);change(view);
  try{validateGraph(view);}catch{rejected.push(name);}
}
if(rejected.length!==7)throw new Error('A forged-graph control was not rejected');
console.log(JSON.stringify({graph_views:cases.length,rejected_controls:rejected}));
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("consumer", type=Path, help="Unmodified pinned src/domain/aspic.ts (no network fetch)")
    parser.add_argument("binary", type=Path, help="Built native eal-core development CLI")
    parser.add_argument("--node", default="node", help="Node >=24 executable")
    parser.add_argument("--views", type=Path, help="Reuse previously generated native views; names must match current fixtures")
    parser.add_argument("--save-views", type=Path, help="Optionally retain freshly generated or checked views")
    args = parser.parse_args()
    node = shutil.which(args.node)
    if node is None:
        parser.error("Node >=24 is required")
    version = subprocess.check_output([node, "--version"], text=True).strip()
    if int(version.removeprefix("v").split(".")[0]) < 24:
        parser.error(f"Node >=24 is required, found {version}")
    binary = args.binary.resolve(strict=True)
    graph = checked_consumer(args.consumer)
    explicit, compiled = fixture_cases()
    with tempfile.TemporaryDirectory(prefix="eal-consumer-check-") as directory:
        temporary = Path(directory)
        views = load(args.views) if args.views else generated_views(binary, temporary / "work", explicit, compiled)
        check_schema_and_policy(views, explicit, compiled)
        path = temporary / "views.json"
        path.write_text(json.dumps(views), encoding="utf-8")
        (temporary / "graph.ts").write_text(graph, encoding="utf-8")
        (temporary / "check.mjs").write_text(GRAPH_RUNNER, encoding="utf-8")
        result = subprocess.run([node, str(temporary / "check.mjs"), str(path)],
                                capture_output=True, text=True, timeout=60, check=True)
        summary = json.loads(result.stdout)
        if args.save_views:
            args.save_views.write_text(json.dumps(views), encoding="utf-8")
    summary.update({
        "consumer_revision": CONSUMER_REVISION, "consumer_git_blob": CONSUMER_BLOB,
        "explicit_views": len(explicit), "compiled_views": len(compiled),
        "published_schema": "Draft202012Validator: passed", "consumer_policy": "passed",
        "node": version, "provided_binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "views": "cached; native commands not rerun" if args.views else "fresh native CLI output; PATH empty",
        "scope": "Unmodified consumer graph function bodies plus separately validated schema/policy; no Ajv, Vue, browser or live bridge",
    })
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
