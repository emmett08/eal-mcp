#!/usr/bin/env python3
"""Development-only identity/rebinding witnesses from the unchanged source lock."""
from __future__ import annotations
import argparse
import ast
from copy import deepcopy
import json
from pathlib import Path
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]


def capture(reference: Path):
    if subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip() != REVISION:
        raise ValueError("Reference revision differs from the source lock")
    if subprocess.check_output(["git", "-C", str(reference), "status", "--porcelain", "--", "src/eal", "grammar"], text=True):
        raise ValueError("Reference production sources must be unchanged")
    sys.path.insert(0, str(reference / "src"))
    from eal.collection_identity import CollectionIdentityValidator
    from eal.evaluator import canonical_digest
    import eal.observation_reuse as reuse
    from eal.parser import parse

    module = ast.parse((reference / "tests/test_observation_reuse.py").read_text())
    source = next(ast.literal_eval(node.value) for node in module.body if isinstance(node, ast.Assign)
                  and any(isinstance(t, ast.Name) and t.id == "SOURCE" for t in node.targets))
    program = parse(source)
    context = {"site": "bench"}
    binding = "a" * 64
    expected = reuse.ObservationRebinder._expected(program, "measured", context, binding, None)
    original = {"schema": "EAL/observation-record/1", "source_digest": program.source_digest,
                **expected, "run_id": "original-run", "started_at": "2020-01-01T00:00:00Z",
                "collected_at": "2020-01-01T00:00:00Z", "ingested_at": "2026-09-23T12:00:00Z",
                "status": "ok", "value": {"passed": True},
                "data_digest": canonical_digest({"passed": True}),
                "stdout_digest": "0" * 64, "stderr_digest": "0" * 64,
                "stdout_bytes": 100, "stderr_bytes": 0}
    identities = []

    def identity(label, collection, target_context=context, digest=program.source_digest):
        case = {"name": label, "collection": collection, "context": target_context, "source_digest": digest}
        try:
            case["records"] = CollectionIdentityValidator().validate(collection, source_digest=digest, context=target_context)
        except (TypeError, ValueError, OverflowError) as error:
            case["error"] = str(error)
        identities.append(case)

    collection = {"source_digest": program.source_digest, "context": context, "records": {"measured": original}}
    identity("same-source-context", collection)
    identity("changed-source", collection, digest="changed")
    identity("changed-context", collection, {"site": "elsewhere"})
    for label, value in [("nonobject", []), ("missing-records", {"source_digest": program.source_digest, "context": context}),
                         ("bad-record", {"source_digest": program.source_digest, "context": context, "records": {"measured": "wrong"}})]:
        identity(label, value)
    for label, left, right in [("integer-float-context", 1, 1.0), ("integer-boolean-context", 1, True),
                               ("negative-zero-context", 0.0, -0.0)]:
        identity(label, {"source_digest": program.source_digest, "context": {"x": left}, "records": {}}, {"x": right})

    class Store:
        def get(self, record_id, *, kind):
            assert kind == "observation" and record_id == self.record["run_id"]
            return deepcopy(self.record)

    store = Store()
    rebinder = reuse.ObservationRebinder(store)
    reuse.uuid4 = lambda: "derived-run"
    reuse.utc_now = lambda: "2026-09-23T12:34:56Z"
    cases = []

    def prepare(label, record=original, *, supplied=None, selected_source=source, selected_context=context,
                evidence="measured", selected_binding=binding, execution=None):
        store.record = deepcopy(record)
        supplied = deepcopy(record if supplied is None else supplied)
        case = {"name": label, "source": selected_source, "context": selected_context,
                "evidence_id": evidence, "binding_digest": selected_binding, "execution_digest": execution,
                "stored": deepcopy(record), "supplied": supplied}
        try:
            case["prepared"] = rebinder.prepare_record(parse(selected_source), selected_context, evidence, supplied,
                                                        current_binding_digest=selected_binding,
                                                        current_execution_digest=execution)
        except (TypeError, ValueError, OverflowError) as error:
            case["error"] = str(error)
        cases.append(case)

    prepare("exact-original")
    prepare("source-rationale-revision", selected_source=source.replace("Measurement supports the bounded claim.", "Revised rationale."))
    prepare("missing-current-binding", selected_binding=None)
    prepare("invalid-current-binding", selected_binding="A" * 64)
    prepare("different-current-binding", selected_binding="b" * 64)
    prepare("invalid-execution-digest", execution="invalid")
    prepare("different-evidence-id", selected_source=source.replace("measured", "another_id"), evidence="another_id")
    prepare("undeclared-evidence-id", evidence="absent")
    prepare("different-context", selected_context={"site": "elsewhere"})
    prepare("different-input", selected_source=source.replace('"value": 7', '"value": 8'))
    for label, key, value in [
        ("failed-observation", "status", "error"), ("wrong-schema", "schema", "wrong"),
        ("missing-run-id", "run_id", None), ("bad-collected-time", "collected_at", "2020-01-01T00:00:00"),
        ("bad-observed-time", "observed_at", "yesterday"), ("wrong-data-digest", "data_digest", "1" * 64),
        ("changed-kind", "evidence_kind", "other"), ("changed-input-digest", "input_digest", "1" * 64),
        ("changed-request-digest", "request_digest", "1" * 64),
        ("changed-environment-fingerprint", "environment_fingerprint", "1" * 64),
        ("changed-tool-version", "tool_version", "2"),
    ]:
        changed = deepcopy(original); changed[key] = value
        prepare(label, changed)
    changed = deepcopy(original); del changed["value"]; prepare("missing-value", changed)
    changed = deepcopy(original); changed["value"] = {"passed": False}; prepare("changed-data", changed)
    changed = deepcopy(original); changed["stdout"] = "private legacy stream"; changed["stderr"] = "private legacy diagnostic"
    prepare("strip-legacy-streams", changed)
    changed = deepcopy(original); changed["origin_run_id"] = "earliest-run"; changed["reused_from_run_id"] = "previous-run"
    prepare("preserve-earliest-origin", changed)
    changed = deepcopy(original); changed["process_environment_digest"] = "c" * 64
    prepare("missing-current-execution", changed)
    prepare("matching-current-execution", changed, execution="c" * 64)
    prepare("changed-current-execution", changed, execution="d" * 64)
    changed = deepcopy(original); changed["started_at"] = "2026-09-23T12:00:00Z"
    prepare("edited-original-metadata", supplied=changed)
    return {"schema": "eal-runtime-collection-reference/1", "source_revision": REVISION,
            "provenance": "Synthetic acquisition correspondence; UUID and rebound time fixed only in the capture process",
            "identity_cases": identities, "reuse_cases": cases}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    output = ROOT / "fixtures/runtime/collection-reference.json"
    encoded = json.dumps(capture(args.reference.resolve()), ensure_ascii=False, indent=2, allow_nan=False) + "\n"
    if args.check:
        if not output.exists() or output.read_text() != encoded:
            raise SystemExit("Collection identity/reuse oracle drift")
    else:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(encoded)


if __name__ == "__main__":
    main()
