#!/usr/bin/env python3
"""Development-only witnesses from the pinned MAS Protocol Lab Store."""
from __future__ import annotations
import argparse
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
STORE_SHA256 = "f2510cbad31646c372d0006cb50be3f9d3f027a540c9142a0a4a09c1ea84b638"


def capture(toolkit: Path):
    store_source = toolkit / "runtime/mas_protocol/store.py"
    if hashlib.sha256(store_source.read_bytes()).hexdigest() != STORE_SHA256:
        raise ValueError("MAS Protocol Lab Store differs from the pinned source")
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(toolkit / "runtime"))
    os.environ["MAS_PROTOCOL_BIN"] = str(toolkit / "rust-protocol/target/release/mas-protocol-codec")
    from mas_protocol import store as source
    from mas_protocol.protocol import ProtocolError
    scenarios = []

    def envelope(**changes):
        return {"message_id": "m", "run_id": "run", "sender": "a", "recipient": "b", "version": "1", "kind": "request", "payload": {"text": "€", "nested": {"z": 1, "a": False}, "$serde_json::private::Number": "literal"}, **changes}

    with tempfile.TemporaryDirectory() as directory:
        def scenario(name, steps):
            clock = [1000.25]
            source.time.time = lambda: clock[0]
            store = source.Store(Path(directory) / (name + ".sqlite"))
            retained = []
            for operation, args in steps:
                record = {"operation": operation, "arguments": deepcopy(args)}
                try:
                    if operation == "advance": clock[0] += args; value = None
                    elif operation == "set_active":
                        with store.transaction() as db: db.execute("UPDATE runs SET active=? WHERE id='run'", (args,))
                        value = None
                    elif operation == "send": value = store.send(args["envelope"], args["actor"])
                    else: value = getattr(store, operation)(**args)
                    record["result"] = value
                except ProtocolError as error: record["error"] = error.to_dict()
                except ValueError as error: record["error"] = {"code": "operation_rejected", "message": str(error)}
                retained.append(record)
            scenarios.append({"name": name, "steps": retained})

        create = {"run_id": "run", "agents": ["a", "b"]}
        first = envelope()
        scenario("delivery-redelivery-ack-identity", [
            ("create_run", create), ("send", {"envelope": first, "actor": "a"}),
            ("send", {"envelope": envelope(payload={"$serde_json::private::Number": "literal", "nested": {"a": False, "z": 1}, "text": "€"}), "actor": "a"}),
            ("send", {"envelope": envelope(payload={"changed": True}), "actor": "a"}),
            ("ack", {"run_id": "run", "message_id": "m", "recipient": "b"}),
            ("receive", {"run_id": "run", "recipient": "b"}),
            ("receive", {"run_id": "run", "recipient": "b"}),
            ("ack", {"run_id": "run", "message_id": "m", "recipient": "a"}),
            ("ack", {"run_id": "run", "message_id": "m", "recipient": "b"}),
            ("ack", {"run_id": "run", "message_id": "m", "recipient": "b"}),
            ("receive", {"run_id": "run", "recipient": "b"}), ("inspect", {"run_id": "run"})])
        scenario("frozen-config-and-exact-actors", [("create_run", create), ("create_run", create),
            ("create_run", {**create, "agents": ["b", "a"]}),
            ("send", {"envelope": first, "actor": "unknown"}),
            ("send", {"envelope": envelope(sender="unknown", recipient="unknown"), "actor": "a"}),
            ("send", {"envelope": envelope(recipient="unknown"), "actor": "a"}),
            ("receive", {"run_id": "run", "recipient": "unknown"}),
            ("inspect", {"run_id": "absent"}), ("inspect", {"run_id": "run"})])
        scenario("deadline-does-not-reopen-or-block-receipts", [
            ("create_run", {**create, "wall_seconds": 1}), ("send", {"envelope": first, "actor": "a"}),
            ("reserve_call", {"run_id": "run", "actor": "a", "call_id": "call"}), ("advance", 1),
            ("create_run", {**create, "wall_seconds": 1}), ("send", {"envelope": first, "actor": "a"}),
            ("send", {"envelope": envelope(payload={"changed": True}), "actor": "a"}),
            ("send", {"envelope": envelope(message_id="new"), "actor": "a"}),
            ("reserve_call", {"run_id": "run", "actor": "a", "call_id": "call"}),
            ("receive", {"run_id": "run", "recipient": "b"}),
            ("ack", {"run_id": "run", "message_id": "m", "recipient": "b"}),
            ("finish_call", {"run_id": "run", "actor": "a", "call_id": "call", "evidence": {"retained": False, "count": 0, "estimate": 1.0}}),
            ("finish_call", {"run_id": "run", "actor": "a", "call_id": "call", "evidence": {}}),
            ("inspect", {"run_id": "run"})])
        size = len(source.dump(first).encode())
        scenario("exact-stored-json-byte-budget", [("create_run", {**create, "max_bytes": size}),
            ("send", {"envelope": first, "actor": "a"}), ("send", {"envelope": first, "actor": "a"}),
            ("send", {"envelope": envelope(message_id="n"), "actor": "a"}), ("inspect", {"run_id": "run"})])
        scenario("message-and-call-budgets", [("create_run", {**create, "max_messages": 1, "max_calls": 1}),
            ("send", {"envelope": first, "actor": "a"}), ("send", {"envelope": envelope(message_id="n"), "actor": "a"}),
            ("reserve_call", {"run_id": "run", "actor": "a", "call_id": " "}),
            ("reserve_call", {"run_id": "run", "actor": "a", "call_id": " "}),
            ("reserve_call", {"run_id": "run", "actor": "b", "call_id": "next"}),
            ("finish_call", {"run_id": "run", "actor": "b", "call_id": " ", "evidence": {}}),
            ("finish_call", {"run_id": "run", "actor": "a", "call_id": " ", "evidence": {"execution_id": "software-test"}}),
            ("finish_call", {"run_id": "run", "actor": "a", "call_id": " ", "evidence": {"execution_id": "software-test"}}),
            ("inspect", {"run_id": "run"})])
        scenario("active-version-and-duplicate-priority", [("create_run", create), ("send", {"envelope": first, "actor": "a"}),
            ("set_active", "2"), ("send", {"envelope": first, "actor": "a"}),
            ("send", {"envelope": envelope(message_id="n"), "actor": "a"}), ("inspect", {"run_id": "run"})])
        scenario("large-integer-budgets-remain-exact", [("create_run", {**create, "max_messages": 10**100, "max_bytes": 10**120, "max_calls": 10**80}),
            ("send", {"envelope": envelope(payload={"large": 10**300, "zero": 0, "negative": -10**300}), "actor": "a"}),
            ("reserve_call", {"run_id": "run", "actor": "a", "call_id": "call"}), ("inspect", {"run_id": "run"})])
        invalid = [("run_id", "bad id"), ("run_id", "café"), ("agents", ["a"]), ("agents", ["a", "a"]),
                   ("agents", ["a", 1]), ("max_messages", 0), ("max_bytes", False), ("max_calls", 1.0)]
        scenario("create-input-admission", [("create_run", {**create, field: value}) for field, value in invalid] + [("inspect", {"run_id": "run"})])
        scenario("receive-input-and-missing-message", [("create_run", create)] + [
            ("receive", {"run_id": "run", "recipient": "b", "limit": limit}) for limit in [0, 101, True, 1.0]] + [
            ("ack", {"run_id": "run", "message_id": "missing", "recipient": "b"}),
            ("reserve_call", {"run_id": "run", "actor": "a", "call_id": ""}),
            ("finish_call", {"run_id": "run", "actor": "a", "call_id": "missing", "evidence": {}})])
    return {"schema": "eal-native-lab-store-reference/1", "store_sha256": STORE_SHA256,
            "provenance": "Pinned toolkit Store; deterministic per-scenario clock; official baseline Rust codec; no model execution", "scenarios": scenarios}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("toolkit", type=Path); parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    output = ROOT / "fixtures/coordination/lab-store-reference.json"
    encoded = json.dumps(capture(args.toolkit.resolve()), ensure_ascii=False, allow_nan=False, indent=2) + "\n"
    if args.check:
        if not output.exists() or output.read_text() != encoded: raise SystemExit("Lab Store reference drift")
    else:
        output.parent.mkdir(parents=True, exist_ok=True); output.write_text(encoded)


if __name__ == "__main__": main()
