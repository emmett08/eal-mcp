#!/usr/bin/env python3
"""Run synthetic history mechanics, never the empirical long-session pilot.

Usage: python3 experiments/eal-context-001/rehearse.py
The 32,768 ceiling here is UTF-8 serialised request BYTES, deliberately not the
protocol's model-token ceiling. No byte/token conversion or model score is made.
"""
from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import sys
import unittest

from history import History, Reference, compact_bytes
from test_history import MachineryCalibration, exchange


ROOT = Path(__file__).resolve().parent
BYTE_CAP = 32768


def synthetic_packet(reference, limits=(16384, 1024)):
    # Deliberately invented fixtures. These are not projections produced by EAL.
    return json.dumps({"fixture_origin": "synthetic_history_machinery_only",
                       "assessment_id": reference.assessment_id,
                       "summary_complete": limits[0] >= 16384,
                       "omitted": {"details": "synthetic_byte_pressure"} if limits[0] < 16384 else {},
                       "fixture_padding": "0123456789abcdef" * (32 if limits[0] < 16384 else 128)},
                      separators=(",", ":"))


def run_trace(condition, horizon):
    host = History(condition, "Synthetic machinery exercise. Qualify unavailable assessment details.",
                   {"ready": "synthetic decision identifier"},
                   [{"name": "synthetic_common_tool", "description": "No external or model calls."}])
    anchors = []
    occupancy = []
    transferred = 0
    missing_historical = 0
    for turn in range(1, horizon + 1):
        host.begin_turn(f"Synthetic history turn {turn}: inspect the scheduled entry and preserve its revision identity.")
        if turn <= 8 or turn % 10 == 0:
            entry = (turn - 1) % 8
            ref = Reference(f"entry-{entry}", "ready", f"assessment-{turn}", f"source-{entry}-{turn}",
                            "context-fixed", "methods-fixed", f"2040-01-01T00:{turn // 60:02d}:{turn % 60:02d}Z")
            payload = synthetic_packet(ref)
            host.add_packet(ref, exchange(f"anchor-{turn}", payload))
            transferred += len(payload.encode())
            anchors.append(ref)
        if turn % 12 == 0 and anchors:
            ref = anchors[0]  # A deliberate long revisit to a historical revision.
            if condition == "B":
                replay = host.cache_lookup(ref)
                # A new tool exchange gets a fresh call ID, with exact body replay.
                payload = replay[-1]["content"]
                host.add_packet(ref, exchange(f"cache-{turn}", payload))
                transferred += len(payload.encode())
            elif condition == "C":
                payload = host.reload(ref, synthetic_packet)
                host.add_packet(ref, exchange(f"reload-{turn}-small", payload), (4096, 1024))
                transferred += len(payload.encode())
                if not json.loads(payload)["summary_complete"]:
                    payload = host.reload(ref, synthetic_packet)
                    host.add_packet(ref, exchange(f"reload-{turn}-default", payload))
                    transferred += len(payload.encode())
        request = host.prepare(compact_bytes, BYTE_CAP)
        if condition == "A" and turn % 12 == 0 and anchors:
            # Observe the history actually supplied after emergency eviction.
            present = any(block.packet and block.reference == anchors[0]
                          for prior in host.turns for block in prior.blocks)
            missing_historical += not present
        occupancy.append(compact_bytes(request))
        host.finish_turn(f"Synthetic placeholder answer {turn}; no correctness or retention score is assigned.")
    return {
        "condition": condition, "scheduled_synthetic_turns": horizon,
        "completed_synthetic_turns": horizon, "shared_anchor_exposures": len(anchors),
        "request_size_unit": "UTF-8 bytes of complete serialised synthetic request",
        "request_sampling": "once per synthetic user turn after scripted exchanges; no model request cycles",
        "request_byte_cap": BYTE_CAP, "peak_request_bytes": max(occupancy),
        "final_request_bytes": occupancy[-1], "cumulative_serialised_request_bytes": sum(occupancy),
        "logical_historical_lookups": host.logical_lookups,
        "host_cache_hits": host.cache_hits, "synthetic_service_callback_calls": host.server_reloads,
        "synthetic_packet_body_bytes_exposed": transferred,
        "A_historical_revisits_without_visible_packet": missing_historical,
        "packet_exchange_evictions": sum(event["type"] == "packet_eviction" for event in host.events),
        "emergency_turn_evictions": sum(event["type"] == "emergency_turn_eviction" for event in host.events),
        "events": host.events,
    }


def main():
    started = datetime.now(timezone.utc).isoformat()
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(MachineryCalibration)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        return 1
    traces = [run_trace(condition, horizon) for horizon in (40, 80, 120) for condition in "ABC"]
    output = {
        "status": "synthetic_host_machinery_rehearsal_completed_not_empirical_pilot",
        "protocol_id": "EAL-CONTEXT-001", "protocol_version": "0.1.0",
        "command": "python3 experiments/eal-context-001/rehearse.py",
        "started_at_utc": started, "completed_at_utc": datetime.now(timezone.utc).isoformat(),
        "python": sys.version, "platform": platform.platform(),
        "code_sha256": {name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
                        for name in ("history.py", "test_history.py", "rehearse.py")},
        "calibration_checks_run": result.testsRun, "calibration_check_failures": len(result.failures),
        "calibration_check_errors": len(result.errors), "synthetic_history_traces": len(traces),
        "synthetic_turns_completed": sum(row["completed_synthetic_turns"] for row in traces),
        "model_sessions": 0, "provider_model_requests": 0, "provider_usage": None,
        "quality_scores": None,
        "limitations": [
            "All packets, user turns and placeholder answers in this rehearsal are synthetic fixtures.",
            "No EAL service, ModelContextBuilder or provider request is executed by this rehearsal.",
            "The cap and recorded occupancies are serialised UTF-8 bytes, not model tokens or token estimates.",
            "Three trace lengths and three policies are machinery configurations, not independent scenario families.",
            "The six software checks do not validate assessors or replace the protocol's six independently reviewed calibration cases.",
            "No model-directed retrieval, empirical efficiency, answer correctness or retention result is established.",
            "The production adapter must supply actual checked model messages, actual MCP schemas, and a validated full-request token counter.",
        ],
        "traces": traces,
    }
    path = ROOT / "rehearsal-results.json"
    path.write_text(json.dumps(output, indent=2) + "\n")
    print(f"Saved {path.name}: {result.testsRun} checks, {len(traces)} synthetic traces, "
          f"{output['synthetic_turns_completed']} synthetic turns; zero model requests.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
