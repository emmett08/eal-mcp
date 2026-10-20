# Pinned MCP descriptors

`reference.json` retains the actual `tools/list` descriptors returned by a
FastMCP client connected to the pinned Python `create_server` application:
11 operator tools and 3 registered tools. Each tool keeps the SDK's public JSON
aliases, description, title, input schema, output schema, metadata and original
enumeration position. SDK `None` fields are omitted. No backend identity or
schema fields are normalized away.

The fixture also retains 11 rejected tool calls, two strict integral-float
rejections, and 10 admission results from the actual FastMCP Host/Origin
middleware with the launcher's strict options. The latter uses a synthetic
204-response handler; native TCP tests map an accepted MCP initialization to
HTTP 200 and compare all rejected statuses exactly. Authentication is tested
separately through actual HTTP requests.

The synthetic registered entry is validated but never collected. Its private
context and source do not appear in descriptors. This is a protocol fixture,
not an observation about an external service or a performance measurement.
The source revision, relevant source-file hashes, required framework pins and
the complete development environment package inventory accompany the capture.

Recreate with the exact development dependencies, outside normal native tests:

```sh
python -m venv /tmp/eal-app-oracle
/tmp/eal-app-oracle/bin/python -m pip install fastmcp==4.0.10 mcp==2.2.0 antlr4-python3-runtime==4.13.2 jsonschema==4.23.0 filelock==3.20.0
/tmp/eal-app-oracle/bin/python tools/capture-mcp-reference.py --reference ../earl-audit
/tmp/eal-app-oracle/bin/python tools/capture-mcp-reference.py --reference ../earl-audit --check
```

The capture rejects a different source revision, tracked source changes and
different required package versions. Rechecking the full retained provenance
also requires the recorded transitive development package versions.

The integral-float errors expose the development Pydantic version in their
diagnostic URL. This backend wording is recorded separately from the shared
JSON Schema rejections and operation errors; both backends reject these inputs.
