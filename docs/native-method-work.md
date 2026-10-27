# Native method work admission

EAL/4 takes operational limits independently of language semantics. A host may
set `method_work` in its limits TOML; for example:

```sh
printf '%s\n' '[limits]' 'method_work = 1' > limits.toml
eal-host run case.eal --tools tools.toml --workspace run --claim claim \
  --limits limits.toml
```

The default allowance is 1,000,000,000 units per native method invocation. It
admits every valid input within the seven standard methods' existing fixed size
bounds. This is a conservative **admission charge**, not measured CPU instructions,
elapsed time, or a cross-machine performance guarantee. There is no deadline
simulation and no successful partial answer. When the charge exceeds the
allowance, assessment returns `status: incomplete`, empty `details`, and the
required and available units in `resource`. No method result predicate is treated
as established. A completed computation whose result fails a predicate retains
its measured result and is distinct from this incomplete calculation.

A unit models one scalar operation, collection visit or lookup. Elementary library
and arbitrary-precision numeric operations count as single operations in this
structural model; their bit complexity is excluded. JSON bytes, nodes, depth and
method input schemas have their own admission limits. The charge is computed
with saturating arithmetic after JSON/schema admission. Complete-query validation
also respects this charge, so it cannot run the calculation before an assessment
that lacks enough work allowance. Operational limits participate in the compiled
program identity and are retained by the normal host request path.

Let `J` count JSON values, including containers, and `L = 64(J + 1)`. The linear
term covers input checking, bounded scans, result construction and fixed scalar
bookkeeping. The authoritative formulas are in `methods/work.rs`:

| Standard method | Admission charge | Relation to the implemented algorithm |
|---|---|---|
| `deductive/1` | `L + 2^a (J + a)(a + 1)` | At most `2^a` valuations; each visits the formula tree and builds/looks up an assignment with at most `a` atoms. Valid inputs have at most 12 atoms and 512 formula nodes. |
| `inductive/1` | `L + 256` | Fixed AS241 polynomial approximation and Wilson interval scalar calculation; no loop over the reported trial count. |
| `abductive/1` | `L + 16h²` | At most `h` hypotheses; the quadratic term bounds ordered-map construction/lookups and posterior scans, including worst-case tree visits. |
| `causal/1` | `L` | Mean and variance use a fixed number of scans over the two acquired groups. Rational arithmetic and square root are elementary operations in this model. |
| `counterfactual/1` | `L + 8(v + 1)²(e + 1)` | At most `v` readiness rounds, with at most `e` dependency checks per round and at most `v` linear membership comparisons; two affine solves are covered by the linear term. |
| `analogical/1` | `L + 16f²` | At most `f` selected features; ordered-map lookups, uniqueness/missing-feature ordering and comparisons are bounded by the quadratic term plus complete input traversal. |
| `temporal/1` | `L` | A fixed number of scans over already ordered sampled events; no search over unsampled times. |

Here `h` is the candidate count, `v` the number of structural variables, `e` the
number of declared structural parent links, and `f` the number of selected
features. Deductive counting caps the shift at 12 for malformed inputs; this does
not admit extra atoms, which retain the calculation's ordinary validation error
when sufficient work is available. Existing finite input bounds remain in force.

`structured/1` does no native calculation; a native computation-work exhaustion
cell is therefore inapplicable. It still requires actual scoped observations and
shares the ordinary acquisition/assessment resource limits. Optional engineering
methods and installed external/native extensions retain their own implementation
bounds and supervisor contracts; this standard-method allowance is not claimed
to meter those implementations.

`crates/eal-core/tests/method_work.rs` exercises every standard computational method
at its default budget, with insufficient work, and at the exact reported charge.
The completed details must be identical at the latter two adequate allowances.
A separately asserted deductive countermodel distinguishes a completed negative
from work exhaustion. EngArg's composed matrix additionally exercises the six
EngArg-native inference methods through exported EAL, real acquisition and the
installed host, including challenge and freshness gates.
