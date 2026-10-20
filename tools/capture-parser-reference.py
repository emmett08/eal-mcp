#!/usr/bin/env python3
"""Refresh checked-in parser oracles from the pinned, unmodified Python source.

Development only. Normal Rust tests read the captured JSON without Python.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import json
import subprocess
import sys
from pathlib import Path

SOURCE_REVISION = "e2d6f346a37f92483b9f9a13923b98feb053447b"
ANTLR_RUNTIME = "4.13.2"
ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures/parser"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def capture(reference: Path) -> dict:
    actual = subprocess.check_output(["git", "-C", str(reference), "rev-parse", "HEAD"], text=True).strip()
    if actual != SOURCE_REVISION:
        raise SystemExit(f"Expected source {SOURCE_REVISION}, found {actual}")
    if subprocess.run(["git", "-C", str(reference), "diff", "--quiet", "HEAD", "--", "src/eal", "grammar/EAL.g4"]).returncode:
        raise SystemExit("The pinned reference production source has local modifications")
    if importlib.metadata.version("antlr4-python3-runtime") != ANTLR_RUNTIME:
        raise SystemExit(f"Install antlr4-python3-runtime=={ANTLR_RUNTIME} before capture")
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(reference / "src"))
    from antlr4 import CommonTokenStream, InputStream, ParserRuleContext
    from antlr4.error.ErrorListener import ErrorListener
    from eal.generated.EALLexer import EALLexer
    from eal.generated.EALParser import EALParser
    from eal.parser import EALSyntaxError, parse

    class Errors(ErrorListener):
        def __init__(self):
            self.items = []

        def syntaxError(self, recognizer, offendingSymbol, line, column, msg, e):
            self.items.append({"line": line, "column": column + 1, "message": msg})

    descriptions = json.loads((FIXTURES / "manifest.json").read_text())
    cases = []
    covered_rules = set()
    for description in descriptions:
        source_path = FIXTURES / description["path"]
        source_bytes = source_path.read_bytes()
        source = source_bytes.decode("utf-8")
        lexer = EALLexer(InputStream(source))
        lexical = Errors()
        lexer.removeErrorListeners()
        lexer.addErrorListener(lexical)
        stream = CommonTokenStream(lexer)
        stream.fill()
        parser = EALParser(stream)
        syntactic = Errors()
        parser.removeErrorListeners()
        parser.addErrorListener(syntactic)
        tree = parser.program()
        recognised = not lexical.items and not syntactic.items
        case = {**description, "sha256": digest(source_bytes), "recognised": recognised}
        if recognised:
            case["tree"] = tree.toStringTree(recog=parser)
            tokens = []
            for token in stream.tokens:
                if token.type == -1:
                    continue
                tokens.append({"type": token.type, "text": token.text, "line": token.line,
                               "column": token.column + 1, "start": token.start,
                               "end": token.stop + 1, "channel": token.channel})
            case["tokens"] = tokens
            stack = [tree]
            while stack:
                node = stack.pop()
                if isinstance(node, ParserRuleContext):
                    covered_rules.add(parser.ruleNames[node.getRuleIndex()])
                    stack.extend(node.children or [])
        else:
            case["lexer_errors"] = lexical.items
            case["parser_errors"] = syntactic.items
        imported = description.get("imports", {})
        resolver = None
        if imported:
            resolver = lambda path: (FIXTURES / imported[path]).read_bytes().decode("utf-8")
        try:
            program = parse(source, resolver=resolver, origin=description.get("origin", ""))
            case["lowered"] = True
            case["language"] = program.language
        except EALSyntaxError as exc:
            case["lowered"] = False
            case["lowering_error"] = str(exc)
        if recognised != description["expect_recognised"]:
            raise SystemExit(f"Unexpected grammar outcome: {description['name']}")
        if case["lowered"] != description["expect_lowered"]:
            raise SystemExit(f"Unexpected lowering outcome: {description['name']}: {case.get('lowering_error')}")
        cases.append(case)
    grammar = (reference / "grammar/EAL.g4").read_bytes()
    return {"schema": "eal-parser-reference/1", "source_repository": "https://github.com/emmett08/earl",
            "source_revision": SOURCE_REVISION, "source_package_version": "3.2.4",
            "grammar_sha256": digest(grammar), "python_antlr_runtime": ANTLR_RUNTIME,
            "span_convention": "Unicode scalar offsets, end exclusive; line and column are 1-based; EOF excluded",
            "rule_coverage": {"covered": sorted(covered_rules), "uncovered": sorted(set(EALParser.ruleNames) - covered_rules)},
            "cases": cases}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path, help="Unmodified earl checkout at the pinned source revision")
    parser.add_argument("--check", action="store_true", help="Fail if checked-in reference differs")
    args = parser.parse_args()
    result = capture(args.reference.resolve())
    cases = result.pop("cases")
    # One case per generated line keeps the oracle compact. Review authored
    # source and manifest files for case meaning; refresh, never hand-edit here.
    header = json.dumps(result, ensure_ascii=False, indent=2)[:-2]
    rows = [json.dumps(case, ensure_ascii=False, separators=(",", ":")) for case in cases]
    data = (header + ',\n  "cases": [\n    ' + ',\n    '.join(rows) + '\n  ]\n}\n').encode("utf-8")
    destination = FIXTURES / "reference.json"
    if args.check:
        if destination.read_bytes() != data:
            raise SystemExit("Parser oracle drift: rerun capture and inspect the change")
    else:
        destination.write_bytes(data)


if __name__ == "__main__":
    main()
