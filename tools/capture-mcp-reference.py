#!/usr/bin/env python3
"""Retain actual pinned FastMCP list_tools results; development use only.

The native tests consume the retained JSON. This script never runs a collector,
changes the reference checkout, or substitutes a different framework version.
"""
from __future__ import annotations

import argparse
import asyncio
import hashlib
from importlib.metadata import distributions, version
import json
import logging
from pathlib import Path
import subprocess
import sys
import tempfile

REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ROOT = Path(__file__).resolve().parents[1]
PINS = {"fastmcp": "4.0.10", "mcp": "2.2.0", "antlr4-python3-runtime": "4.13.2",
        "jsonschema": "4.23.0", "filelock": "3.20.0"}
OPERATOR = {"eal_describe", "eal_format", "eal_validate", "eal_plan", "eal_collect",
            "eal_collect_claim", "eal_reason", "eal_compile_aspic", "eal_explain",
            "eal_packet", "eal_grounded"}
REGISTERED = {"eal_sources", "eal_find_claims", "eal_assess_known"}
ERROR_REQUESTS = {
    "operator": [
        ("validate-source-number", "eal_validate", {"source": 7}),
        ("validate-unknown-field", "eal_validate", {"source": "", "unknown": True}),
        ("validate-missing-source", "eal_validate", {}),
        ("grounded-arguments-string", "eal_grounded", {"arguments": "a", "attacks": []}),
        ("unknown-tool", "eal_unregistered", {}),
        ("plan-empty-source", "eal_plan", {"source": "", "claim": "outcome"}),
    ],
    "registered": [
        ("sources-boolean-limit", "eal_sources", {"limit": True}),
        ("sources-fractional-limit", "eal_sources", {"limit": 1.5}),
        ("registered-context-injection", "eal_assess_known",
         {"entry_id": "synthetic", "claim": "outcome", "context": {}}),
        ("sources-zero-limit", "eal_sources", {"limit": 0}),
        ("registered-unexposed-entry", "eal_assess_known", {"entry_id": "unexposed", "claim": "outcome"}),
    ],
}
SOURCE = '''language "EAL/3"
environment synthetic {
  require true
}
tool reader {
  version "1"
}
evidence observation {
  tool reader
  kind test
  environment synthetic
  max_age 60
  require passed == true
}
reasoning measured {
  method "structured/1"
  rationale "A synthetic observation supports this bounded claim."
}
claim outcome {
  statement "The synthetic predicate holds."
  environment synthetic
}
argument support = [evidence observation] via measured => outcome
'''


async def http_admission():
    """Run the actual framework middleware with the source launcher's options."""
    from fastmcp.server.http import HostOriginGuardMiddleware
    cases = [
        ("wildcard-bind-loopback-client", "0.0.0.0", [], [], "127.0.0.1:8000", None),
        ("default-localhost-client", "127.0.0.1", [], [], "localhost:8000", None),
        ("default-ipv6-loopback-client", "127.0.0.1", [], [], "[::1]:8000", None),
        ("same-request-origin", "127.0.0.1", [], [], "127.0.0.1:8000", "http://127.0.0.1:8000"),
        ("different-loopback-origin", "127.0.0.1", [], [], "127.0.0.1:8000", "https://localhost:9999"),
        ("proxy-same-origin", "0.0.0.0", ["engineering.example"], [], "engineering.example", "http://engineering.example"),
        ("proxy-different-scheme", "0.0.0.0", ["engineering.example"], [], "engineering.example", "https://engineering.example"),
        ("explicit-proxy-origin", "0.0.0.0", ["engineering.example"], ["https://engineering.example"], "engineering.example", "https://engineering.example"),
        ("foreign-host", "127.0.0.1", [], [], "foreign.example", None),
        ("foreign-origin", "127.0.0.1", [], [], "127.0.0.1:8000", "https://foreign.example"),
    ]
    result = []
    for name, host, hosts, origins, header, origin in cases:
        async def accepted(scope, receive, send):
            await send({"type": "http.response.start", "status": 204, "headers": []})
            await send({"type": "http.response.body", "body": b""})

        statuses = []
        async def send(message):
            if message["type"] == "http.response.start":
                statuses.append(message["status"])
        async def receive():
            return {"type": "http.request", "body": b"", "more_body": False}

        middleware = HostOriginGuardMiddleware(accepted, allowed_hosts=[host, *hosts],
                                               allowed_origins=origins, mode="strict")
        headers = [(b"host", header.encode("ascii"))]
        if origin is not None:
            headers.append((b"origin", origin.encode("ascii")))
        await middleware({"type": "http", "scheme": "http", "server": (host, 8000),
                          "headers": headers, "method": "POST", "path": "/mcp"}, receive, send)
        if len(statuses) != 1:
            raise ValueError(f"Invalid HTTP middleware result: {name}")
        result.append({"name": name, "bind_host": host, "allowed_hosts": hosts,
                       "allowed_origins": origins, "host_header": header, "origin": origin,
                       "status": statuses[0]})
    return result


def git(reference: Path, *arguments: str) -> str:
    return subprocess.check_output(["git", "-C", str(reference), *arguments], text=True).strip()


async def capture(reference: Path) -> dict:
    # Expected rejected calls are retained below rather than logged as tracebacks.
    logging.disable(logging.CRITICAL)
    for package, expected in PINS.items():
        actual = version(package)
        if actual != expected:
            raise ValueError(f"{package} must be {expected}, observed {actual}")
    if git(reference, "rev-parse", "HEAD") != REVISION:
        raise ValueError("Reference checkout is not the pinned revision")
    if git(reference, "status", "--porcelain", "--untracked-files=no"):
        raise ValueError("Reference checkout has tracked modifications")
    sys.path.insert(0, str(reference / "src"))
    from fastmcp import Client
    from eal.catalogue import WorkspaceKnowledgeCatalogue
    from eal.runtime import ReasoningService
    from eal.server import create_server

    async def descriptors(server, expected, exposure):
        # The real pinned client initializes an in-process MCP connection and
        # sends list_tools, rather than inspecting Python function signatures.
        async with Client(server) as client:
            tools = await client.list_tools()
            names = {tool.name for tool in tools}
            if names != expected or len(tools) != len(expected):
                raise ValueError(f"Unexpected live tool inventory: {sorted(names)}")
            errors = []
            for name, tool, arguments in ERROR_REQUESTS[exposure]:
                case = {"name": name, "tool": tool, "arguments": arguments}
                try:
                    result = await client.call_tool(tool, arguments, raise_on_error=False)
                except Exception as error:
                    case["error"] = {"type": type(error).__name__, "message": str(error)}
                else:
                    if not result.is_error:
                        raise ValueError(f"Invalid request unexpectedly succeeded: {name}")
                    case["result"] = {
                        "is_error": bool(result.is_error),
                        "structured_content": result.structured_content,
                        "content": [item.model_dump(mode="json", by_alias=True, exclude_none=True)
                                    for item in result.content],
                    }
                errors.append(case)
            calls = []
            if exposure == "registered":
                for name, arguments in [("sources-integral-float-limit", {"limit": 1.0}),
                                        ("sources-integral-float-offset", {"offset": 0.0})]:
                    result = await client.call_tool("eal_sources", arguments, raise_on_error=False)
                    calls.append({"name": name, "tool": "eal_sources", "arguments": arguments,
                                  "comparison": "Rejection outcome; Pydantic diagnostic is backend-specific",
                                  "result": {"is_error": bool(result.is_error),
                                             "structured_content": result.structured_content,
                                             "content": [item.model_dump(mode="json", by_alias=True, exclude_none=True)
                                                         for item in result.content]}})
            return ([tool.model_dump(mode="json", by_alias=True, exclude_none=True)
                     for tool in tools], errors, calls)

    with tempfile.TemporaryDirectory(prefix="eal-mcp-reference-") as directory:
        workspace = Path(directory)
        (workspace / "synthetic.eal").write_text(SOURCE, encoding="utf-8")
        service = ReasoningService(workspace)
        catalogue = WorkspaceKnowledgeCatalogue(service)
        catalogue.register("synthetic.eal", entry_id="synthetic", claims=["outcome"],
                           context={"private_operator_value": "not-public-descriptor-data"})
        operator, operator_errors, _ = await descriptors(create_server(service), OPERATOR, "operator")
        registered, registered_errors, registered_calls = await descriptors(
            create_server(service, known_entries=["synthetic"]), REGISTERED, "registered")
    files = ["src/eal/server.py", "src/eal/operations.py", "src/eal/operation_contracts.py",
             "src/eal/mcp_guard.py", "src/eal/catalogue.py", "src/eal/registered_assessment.py"]
    result = {
        "provenance": {
            "source_revision": REVISION,
            "source_files_sha256": {path: hashlib.sha256((reference / path).read_bytes()).hexdigest()
                                    for path in files},
            "framework_pins": PINS,
            "environment_distributions": dict(sorted((distribution.metadata["Name"], distribution.version)
                                                       for distribution in distributions())),
            "capture": "FastMCP Client(create_server(...)) initialize and list_tools",
            "normalization": "Object key order and SDK omission of absent/None optional fields only; no non-null descriptor field or value removed",
        },
        "operator_tools": operator,
        "registered_tools": registered,
        "operator_errors": operator_errors,
        "registered_errors": registered_errors,
        "registered_calls": registered_calls,
        "http_admission": await http_admission(),
    }
    if git(reference, "status", "--porcelain", "--untracked-files=no"):
        raise ValueError("Reference checkout changed during capture")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", required=True, type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "fixtures/mcp/reference.json")
    parser.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    result = asyncio.run(capture(arguments.reference.resolve()))
    encoded = json.dumps(result, ensure_ascii=False, allow_nan=False, sort_keys=True, indent=2) + "\n"
    if arguments.check:
        if arguments.output.read_text(encoding="utf-8") != encoded:
            raise SystemExit("Retained MCP descriptors differ from the pinned live framework")
        print("MCP reference verified: 11 operator and 3 registered descriptors")
    else:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(encoded, encoding="utf-8")
        print(f"Captured 11 operator and 3 registered descriptors: {arguments.output}")


if __name__ == "__main__":
    main()
