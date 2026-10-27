#!/usr/bin/env python3
"""Synthetic bounded fixture only. Production acquisition uses the official SDK."""
import json
import sys
for line in sys.stdin:
    message = json.loads(line)
    identifier = message.get("id")
    if identifier is None:
        continue
    method = message["method"]
    if method == "initialize":
        result = {"protocolVersion": "2025-11-25", "capabilities": {"tools": {}},
                  "serverInfo": {"name": "synthetic-direct", "version": "1"}}
    elif method == "tools/list":
        result = {"tools": [{"name": "measurement", "description": "Synthetic fixture",
                  "inputSchema": {"type": "object", "properties": {"case": {"type": "string"}}, "required": ["case"], "additionalProperties": False}}]}
    elif method == "tools/call":
        case = message["params"]["arguments"]["case"]
        value = {"value": {"passed": False, "count": 9007199254740993}, "observed_at": "2026-01-02T03:04:05Z"}
        if case == "error":
            result = {"isError": True, "content": [{"type": "text", "text": "measurement failed"}]}
        elif case == "text":
            result = {"content": [{"type": "text", "text": json.dumps(value)}]}
        elif case == "duplicate":
            result = {"content": [{"type": "text", "text": '{"value":1,"value":2}'}]}
        elif case == "large":
            result = {"structuredContent": {"value": "x" * 20000}, "content": []}
        else:
            result = {"structuredContent": value, "content": []}
    else:
        print(json.dumps({"jsonrpc": "2.0", "id": identifier, "error": {"code": -32601, "message": "unknown method"}}), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": identifier, "result": result}), flush=True)
