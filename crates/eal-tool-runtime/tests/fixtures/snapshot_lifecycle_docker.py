#!/usr/bin/env python3
"""Model daemon-owned resources while the native supervisor controls real delays."""

import json
import os
from pathlib import Path
import sys
import time


root = Path(__file__).parent
config = json.loads((root / "config.json").read_text())
state_path = root / "snapshot.json"
args = sys.argv[1:]


def event(operation, **fields):
    with (root / "events.jsonl").open("a") as output:
        output.write(json.dumps({"operation": operation, "pid": os.getpid(), **fields}) + "\n")


def option(name):
    return args[args.index(name) + 1]


if args[:2] == ["image", "inspect"]:
    print("sha256:" + "0" * 64)
elif args[0] == "create":
    name = option("--name")
    owner = name.removesuffix("-inputs")
    assert option("--label") == "engarg.snapshot-owner=" + owner
    state_path.write_text(json.dumps({"name": name, "owner": owner}))
    event("create", name=name)
    time.sleep(config["create_delay"])
    print("1" * 64)
elif args[:2] == ["container", "inspect"]:
    state = json.loads(state_path.read_text())
    assert args[-1] == state["name"]
    if ".Config.Labels" in option("--format"):
        event("ownership", name=args[-1])
        print("foreign-owner" if config["foreign_owner"] else state["owner"])
    else:
        event("mounts")
        print(json.dumps([
            {"Type": "volume", "Name": "source_volume", "Destination": "/src", "RW": True},
            {"Type": "volume", "Name": "fixture_volume", "Destination": "/fixtures", "RW": True},
        ]))
elif args[0] == "cp":
    destination = args[-1].split(":", 1)[1]
    assert destination in ["/src", "/fixtures"]
    assert sys.stdin.buffer.read(), "upload must contain an archive"
    event("source upload" if destination == "/src" else "fixture upload")
    if destination == "/src":
        time.sleep(config["upload_delay"])
elif args[0] == "run":
    assert state_path.exists()
    assert sys.stdin.buffer.read() == b"bounded-lifecycle-test\n"
    event("worker", name=option("--name"))
    print("worker reached")
elif args[:2] == ["container", "rm"]:
    name = args[-1]
    snapshot = name.endswith("-inputs")
    event("remove snapshot" if snapshot else "remove worker", name=name, volumes="--volumes" in args)
    if snapshot:
        assert "--volumes" in args
        assert json.loads(state_path.read_text())["name"] == name
        state_path.unlink()
else:
    raise AssertionError("unexpected Docker command: " + repr(args))
