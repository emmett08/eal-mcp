#!/usr/bin/env python3
"""Capture pinned canonical JSON witnesses; normal Rust tests use retained data."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import random
import struct
import subprocess
import sys

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    reference = args.reference.resolve()
    revision = subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip()
    if revision != REVISION:
        parser.error("reference checkout differs from the pinned revision")
    if subprocess.check_output(["git", "-C", str(reference), "status", "--porcelain", "--", "src/eal", "grammar"]):
        parser.error("reference source must be unchanged")
    sys.path.insert(0, str(reference / "src"))
    from eal.evaluator import _canonical_bytes

    floats = [0.0, -0.0, 1.0, -1.0, 1e-4, 1e-5, 1e-6, 1e-7,
              1e15, 1e16, 1e20, 1e21, 1.2345678901234567,
              float.fromhex("0x0.0000000000001p-1022"), sys.float_info.max,
              sys.float_info.min]
    for exponent in range(-307, 309, 7):
        value = 10.0 ** exponent
        if math.isfinite(value):
            floats.extend([value, math.nextafter(value, 0), math.nextafter(value, math.inf)])
    rng = random.Random(20261020)
    while len(floats) < 800:
        value = struct.unpack(">d", rng.getrandbits(64).to_bytes(8, "big"))[0]
        if math.isfinite(value):
            floats.append(value)
    cases = []
    for value in floats:
        canonical = _canonical_bytes(value)
        cases.append({"source": json.dumps(value), "bits": struct.pack(">d", value).hex(),
                      "canonical_utf8": canonical.decode(), "sha256": hashlib.sha256(canonical).hexdigest()})
    result = {"schema": "eal-canonical-reference/1", "source_revision": REVISION,
              "seed": 20261020, "cases": cases}
    output = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    target = ROOT / "fixtures/canonical/reference.json"
    if args.check:
        if target.read_text() != output:
            raise SystemExit("Canonical reference drift")
    else:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(output)
    print(f"Canonical reference: {len(cases)} finite binary64 cases")


if __name__ == "__main__":
    main()
