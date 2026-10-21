# EAL packet context experiment

[EAL-CONTEXT-001](eal-context-experiment.md) asks whether bounded historical
retrieval saves cumulative model tokens while preserving engineering decisions,
objections and qualifications. The attached protocol is retained byte for byte.
[Results](results.md) report the executed parts and the remaining empirical work.

**The 144-session model pilot has not run.** This directory contains executed
planning simulations, synthetic host-policy checks and a native packet diagnostic.
None supplies empirical model token savings or answer-quality measurements.

The user explicitly requested inclusion in this repository. These optional
research files are separate from the Rust production workspace. Normal builds,
tests, binaries and dependencies do not invoke them. The narrow scope exception
is recorded in the root `AGENTS.md` and [execution status](execution-status.json).

## Reproduce the planning calculations

Use Python 3.12 and the versions in
[planning/requirements.txt](planning/requirements.txt). The original execution
used Python 3.12.14. Install dependencies in a separate environment:

```sh
python3 -m venv /tmp/eal-context-env
/tmp/eal-context-env/bin/python -m pip install -r experiments/eal-context-001/planning/requirements.txt
/tmp/eal-context-env/bin/python experiments/eal-context-001/planning/allocation.py
/tmp/eal-context-env/bin/python experiments/eal-context-001/planning/sensitivity.py
```

These commands overwrite the corresponding planning result JSON files. The
[execution record](planning/execution.json) retains the original invocations,
versions, timings and SHA-256 hashes. Each of 43 cells uses 6,000 synthetic
studies; no provider or network calls occur. The
[calculation notes](planning/calculation-notes.md) state assumptions and limits.

## Reproduce the host-policy rehearsal

The history implementation uses an injected token-counting function. Its checks
and rehearsal use synthetic packet fixtures and a labelled byte counter. They
exercise eviction, restoration and failure accounting without a language model.
See [rehearsal-results.json](rehearsal-results.json) for the actual scope.

```sh
python3 -m unittest discover -s experiments/eal-context-001 -p test_history.py -v
python3 experiments/eal-context-001/rehearse.py
```

## Reproduce the native diagnostic

The protocol freezes reference `de3a99dfb1003f43f75d7bfe18007179ecb4f4aa`
and candidate `71661ea05225157961fe32b355db62dac42e3bba` from PR #9.
The experiment PR is based on current `main`; its production binary is not
silently substituted for either experimental revision. Prepare separate clean
checkouts and build both with the repository's pinned Rust toolchain:

```sh
git fetch origin de3a99dfb1003f43f75d7bfe18007179ecb4f4aa 71661ea05225157961fe32b355db62dac42e3bba
git worktree add --detach ../eal-context-reference de3a99dfb1003f43f75d7bfe18007179ecb4f4aa
git worktree add --detach ../eal-context-candidate 71661ea05225157961fe32b355db62dac42e3bba
(cd ../eal-context-reference && cargo build --workspace --locked)
(cd ../eal-context-candidate && cargo build --workspace --locked)
python3 experiments/eal-context-001/native_probe.py --candidate-root ../eal-context-candidate --reference-root ../eal-context-reference --output experiments/eal-context-001/native-results.json
```

Use fresh checkout paths if these already exist. The diagnostic uses temporary
private data stores, actual native processes and the stated packet budgets.
[Native results](native-results.json) record exact revisions, binary hashes,
checks and limitations. This is a machinery diagnostic rather than a model run.

## Run the empirical pilot next

[execution-status.json](execution-status.json) records the missing prerequisites.
The current host has no authenticated model endpoint exposing per-request usage.
Native assistant delegation does not provide the controlled history, exact model
configuration or token instrument required by this protocol.

Before the 11,520 scheduled user turns plus tool cycles, freeze the model,
tokenizer, endpoint prices and monetary ceiling. Author and independently review
24 scenario families and their event schedules, time-indexed gold and
answerability ledgers. Integrate metered provider requests with checked
`ModelContextBuilder` messages, actual registered tools and the host policies.
Reconcile raw provider usage and validate the six end-to-end calibration cases.
Then assign two blinded assessors for every primary decision answer.

The synthetic rehearsal cases are machinery fixtures. They do not meet those
scenario-authoring, provider-instrumentation or independent-scoring requirements.
[record-schema.json](record-schema.json) remains the protocol's planned logging
field contract; it is not an executable JSON Schema or a completed event logger.
The research runner for live model sessions remains to be implemented.

Keep the original pilot and confirmation separate. Planning simulations do not
establish the empirical variance, final allocation, practical acceptance of the
quality margins or production readiness.
