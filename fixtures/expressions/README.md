# Expression reference cases

`reference.json` retains 350 synthetic expression cases from the pinned Python
implementation. The cases include strict Boolean/scalar typing, unknown fields,
error propagation through Boolean expressions, arbitrary-size integer
arithmetic, exact integer/binary64 comparisons, signed remainders and zero,
underflow/overflow, compensated summation and its integer fallback boundaries,
rational unit conversions and dimensional checks.

Float outcomes are compared by their IEEE-754 binary64 bits. Integer outcomes
retain their exact decimal values. Quantity outcomes retain exact rational
numerators, denominators and dimensions. Tests also compare static diagnostics,
source expression formatting and semantic JSON interchange.

The capture records its Python version because Python's numerical implementation
can change between releases. For example, `sum` in this pinned development
reference uses CPython 3.12.14 with a 64-bit C `long`. Rust preserves that
numerical profile on every target, including Windows: floating elements use
compensation until the interpreter's integer fallback boundary is reached.
The normal Rust test suite reads the captured data and requires no Python
interpreter.

One malformed direct call, `sum()`, raises an uncaught `IndexError` in the Python
evaluator. Both static validators reject its arity. The Rust evaluator additionally
returns `invalid_expression` for that direct call; its recorded Python exception
is excluded from exact runtime-reason comparison.

To reproduce the development reference:

```sh
python3 tools/capture-expression-reference.py ../earl-audit --check
```

The tool requires that interpreter profile, the exact source revision, unchanged
production sources and ANTLR Python runtime recorded by the source lock.
