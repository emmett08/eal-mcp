#!/usr/bin/env python3
"""Check installed native CLI, catalogue, stdio and HTTP application boundaries.

This standard-library development orchestrator launches only the explicitly
selected native executables. Every native child receives an empty PATH, so its
normal executable search cannot find Python. Inputs are synthetic software
fixtures, not provider or model-quality measurements. The fresh workspace and
its JSON report are retained for inspection.
"""
from __future__ import annotations
import argparse
import datetime as dt
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

TOKEN = "synthetic-installed-operator-token.ABC_123~"
TOKEN_ENV = "EAL_SMOKE_OPERATOR_TOKEN"
PROFILE = "all"
OPERATOR_TOOLS = {"eal_describe", "eal_format", "eal_validate", "eal_plan", "eal_collect", "eal_collect_claim", "eal_reason", "eal_compile_aspic", "eal_explain", "eal_packet", "eal_grounded"}
REGISTERED_TOOLS = {"eal_sources", "eal_find_claims", "eal_assess_known"}
SOURCE = '''language "EAL/3"
environment lab {
 require site == "bench"
}
tool reader {
 version "1"
}
evidence reading {
 tool reader
 kind test
 environment lab
 max_age 3600
 input {"sample":1}
 require ok == true
}
reasoning measured {
 method "structured/1"
 rationale "The supplied report supports the selected test result."
}
claim works {
 statement "The supplied test passed at this bench."
 environment lab
}
argument result = [evidence reading] via measured => works
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prefix", type=Path, required=True, help="Installed native binary directory")
    parser.add_argument("--workspace", type=Path, help="Empty directory or new location for retained smoke evidence")
    args = parser.parse_args()
    binaries = args.prefix.resolve()
    required = ["eal", "eal-mcp", "eal-host", "eal-command-supervisor", "eal-method-worker"]
    for name in required:
        assert (binaries / name).is_file() and os.access(binaries / name, os.X_OK), f"Installed companion is missing or not executable: {binaries/name}"
    root = args.workspace.resolve() if args.workspace else Path(tempfile.mkdtemp(prefix="eal-app-installed-smoke-"))
    if root.exists() and (not root.is_dir() or any(root.iterdir())):
        parser.error("--workspace must name an empty directory or a new location")
    root.mkdir(parents=True, exist_ok=True)
    now = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    (root / "case.eal").write_text(SOURCE, encoding="utf-8")
    (root / "context.json").write_text('{"site":"bench"}', encoding="utf-8")
    registry = root / "tools.toml"
    registry.write_text("[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n", encoding="utf-8")
    (root / "observation.json").write_text(json.dumps({"observed_at": now, "context": {"site": "bench"},
        "request": {"tool": "reader", "tool_version": "1", "input": {"sample": 1}, "context": {"site": "bench"}},
        "value": {"ok": True, "secret": "collector-secret"}}), encoding="utf-8")
    environment = {"PATH": "", "LANG": "C.UTF-8", "NO_PROXY": "*", TOKEN_ENV: TOKEN}
    checked = []
    def run(executable, arguments, *, code=0, request=None, extra_environment=None):
        child_env = dict(environment)
        child_env.update(extra_environment or {})
        assert child_env["PATH"] == "", "Native children must have an empty PATH"
        completed = subprocess.run([str(binaries/executable), *map(str, arguments)], cwd=root,
            env=child_env, input=None if request is None else json.dumps(request).encode("utf-8"),
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=45)
        stdout = completed.stdout.decode("utf-8")
        assert completed.returncode == code, (executable, arguments, completed.returncode, stdout, completed.stderr.decode("utf-8"))
        # json.loads rejects any extra non-whitespace stdout before or after the response.
        result = json.loads(stdout)
        assert isinstance(result, dict), (executable, stdout)
        return result, stdout, completed.stderr.decode("utf-8")
    common = ["--workspace", root, "--registry", registry, "--methods", PROFILE]
    def cli(*arguments, **kwargs):
        return run("eal", [*common, *arguments], **kwargs)[0]
    def host(request, *arguments, code=0, extra_environment=None):
        return run("eal-host", [*common, "--timeout", "15", *arguments], request=request, code=code, extra_environment=extra_environment)

    assert cli("validate", "case.eal")["valid"] is True
    assert cli("plan", "case.eal", "--claim", "works")["evidence_ids"] == ["reading"]
    entry = cli("register", "case.eal", "--entry-id", "demo", "--claim", "works", "--context", "@context.json")
    assert entry["entry_id"] == "demo" and entry["claims"] == ["works"]
    assert "source" not in entry
    assert cli("sources")["entries"][0]["entry_id"] == "demo"
    assert cli("find", "supplied", "--claim", "works")["matches"][0]["source_digest"] == entry["source_digest"]
    checked.append("production CLI validate/plan/register/sources/find")

    first = cli("assess-known", "demo", "--claim", "works", "--now", now, "--reuse", "fresh")
    assert first["status"] == "supported" and first["collected_count"] == 1 and first["reused_count"] == 0
    reused = cli("assess-known", "demo", "--claim", "works", "--now", now)
    assert reused["status"] == "supported" and reused["reused_count"] == 1 and reused["collected_count"] == 0
    packet = cli("packet", reused["assessment_id"], "--claim", "works")
    assert packet["schema"] == "EAL/assessment-packet/2" and packet["assessment_id"] == reused["assessment_id"]
    assert packet["collection_id"] == reused["collection_id"]
    assert "collector-secret" not in json.dumps(packet)
    explanation = cli("explain", reused["assessment_id"], "--claim", "works")
    assert explanation["result"]["status"] == "supported"
    checked.append("durable registered assessment, compatible observation reuse, explanation and packet")

    prepared = cli("model-context", "demo", "--claim", "works", "--question", "Did the supplied test pass?", "--now", now)
    assert prepared["schema"] == "EAL/model-context/2"
    assert prepared["assessment"]["reused_count"] == 1 and prepared["assessment"]["collected_count"] == 0
    assert prepared["messages"][1] == {"role": "user", "content": "Did the supplied test pass?"}
    assert "collector-secret" not in json.dumps(prepared["messages"])
    history = cli("history", "demo", "--source-digest", entry["source_digest"])
    assert history["assessments"][0]["assessment_id"] == prepared["assessment"]["assessment_id"]
    assert history["assessments"][0]["claim_statuses"]["works"] == "supported"
    checked.append("model context reuses evidence, excludes raw tool values, and appears in revision-bound history")

    answer, stdout, stderr = host({"operation": "grounded", "arguments": ["a", "b", "c"], "attacks": [["a", "b"], ["b", "c"]]})
    assert answer["protocol_version"] == "2025-11-25" and answer["is_error"] is False
    assert answer["result"]["accepted"] == ["a", "c"] and answer["result"]["rejected"] == ["b"]
    answer, _, _ = host({"operation": "assess_known", "entry_id": "demo", "claim": "works"}, "--known-entry", "demo")
    assert answer["result"]["status"] == "supported" and answer["result"]["reused_count"] == 1
    assert answer["result"]["collected_count"] == 0
    checked.append("actual eal-host stdio full and restricted SDK sessions with one JSON stdout response")

    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    def post(url, payload, *, bearer=TOKEN, origin=None, host_header=None):
        headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream", "MCP-Protocol-Version": "2025-11-25"}
        if bearer is not None:
            headers["Authorization"] = "Bearer " + bearer
        if origin is not None:
            headers["Origin"] = origin
        if host_header is not None:
            headers["Host"] = host_header
        request = urllib.request.Request(url, json.dumps(payload).encode("utf-8"), headers=headers)
        try:
            with opener.open(request, timeout=5) as response:
                return response.status, response.read()
        except urllib.error.HTTPError as error:
            return error.code, error.read()
    for restricted in [False, True]:
        # Reserve a local port, then start the installed server; no executable lookup is involved.
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        url = f"http://127.0.0.1:{port}/mcp"
        label = "restricted" if restricted else "full"
        stdout_path = root / f"http-{label}.stdout"
        stderr_path = root / f"http-{label}.stderr"
        arguments = [*common, "--transport", "http", "--host", "127.0.0.1", "--port", str(port), "--token-env", TOKEN_ENV]
        if restricted:
            arguments += ["--known-entry", "demo"]
        with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
            server = subprocess.Popen([str(binaries/"eal-mcp"), *map(str, arguments)], cwd=root, env=environment, stdout=out, stderr=err)
            try:
                initialize = {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "installed-native-smoke", "version": "1"}}}
                deadline = time.monotonic() + 20
                while True:
                    assert server.poll() is None, ("HTTP server failed", server.returncode, stderr_path.read_text())
                    try:
                        status, _ = post(url, initialize, bearer=None)
                        if status == 401:
                            break
                    except (OSError, urllib.error.URLError):
                        pass
                    assert time.monotonic() < deadline, ("HTTP server startup deadline", stderr_path.read_text())
                    time.sleep(0.025)
                status, body = post(url, initialize)
                initialized = json.loads(body)
                assert status == 200 and initialized["result"]["protocolVersion"] == "2025-11-25", (status, body)
                status, body = post(url, {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
                tools = {tool["name"] for tool in json.loads(body)["result"]["tools"]}
                assert status == 200 and tools == (REGISTERED_TOOLS if restricted else OPERATOR_TOOLS), tools
                for bearer, origin, host_header, expected in [(None, None, None, 401), ("wrong-token", None, None, 401), (TOKEN, "https://foreign.example", None, 403), (TOKEN, None, "foreign.example", 421)]:
                    assert post(url, initialize, bearer=bearer, origin=origin, host_header=host_header)[0] == expected
                if restricted:
                    answer, stdout, _ = host({"operation": "sources"}, "--url", url, "--token-env", TOKEN_ENV)
                    assert answer["is_error"] is False and answer["result"]["sources"][0]["entry_id"] == "demo"
                    answer, stdout, _ = host({"operation": "assess_known", "entry_id": "demo", "claim": "works"}, "--url", url, "--token-env", TOKEN_ENV)
                    assert answer["result"]["status"] == "supported" and answer["result"]["reused_count"] == 1
                    rejected, _, _ = host({"operation": "validate", "source": SOURCE}, "--url", url, "--token-env", TOKEN_ENV, code=2)
                    assert "does not offer eal_validate" in rejected["error"]
                    checked.append("authenticated restricted HTTP inventory, source selection and reusable assessment; operator tool unavailable")
                else:
                    secret_source = SOURCE.replace("The supplied test passed at this bench.", TOKEN)
                    answer, stdout, _ = host({"operation": "format", "source": secret_source}, "--url", url, "--token-env", TOKEN_ENV)
                    assert answer["is_error"] is False and answer["protocol_version"] == "2025-11-25"
                    assert TOKEN not in stdout and "[redacted]" in answer["result"]["source"]
                    assert TOKEN not in json.dumps(answer["content"])
                    answer, _, _ = host({"operation": "reason", "source": SOURCE, "context": {"site": "bench"}, "collection_id": reused["collection_id"], "now": now}, "--url", url, "--token-env", TOKEN_ENV)
                    assert answer["result"]["claims"]["works"]["status"] == "supported"
                    checked.append("authenticated full HTTP inventory and reasoning; host redacts bearer from structured and encoded content")
                checked.append(f"{label} HTTP rejects missing/wrong bearer, foreign Origin, foreign Host")
            finally:
                if server.poll() is None:
                    server.send_signal(signal.SIGINT)
                    try:
                        server.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        server.kill()
                        server.wait(timeout=5)
                assert server.returncode == 0, (label, server.returncode, stderr_path.read_text())
        assert stdout_path.read_bytes() == b"", ("HTTP server emitted unexpected stdout", stdout_path.read_text())
    checked.append("all installed native children ran with PATH empty; HTTP servers emitted no stdout")
    assert len(checked) == 9, checked
    result = {"result": "passed", "prefix": str(binaries), "workspace": str(root), "observed_at": now,
              "profile": PROFILE, "child_path": "", "check_groups": len(checked), "checks": checked}
    (root / "smoke-result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result))

if __name__ == "__main__":
    main()
