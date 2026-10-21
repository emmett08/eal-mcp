#!/usr/bin/env python3
"""Sensitivity to effects close to the predeclared decision boundaries."""
import json
from pathlib import Path
from allocation import simulate, SEED
specs = [(96,.05,.85,(0.,0.,0.)), (384,.10,.88,(0.,0.,0.)),
         (384,.10,.8,(-.01,-.01,-.01)), (768,.10,.8,(-.01,-.01,-.01)),
         (1536,.10,.8,(-.01,-.01,-.01))]
results = []
for j, (n, sd, ratio, means) in enumerate(specs):
    x=simulate(n, sd, ratio=ratio, quality_means=means, seed=SEED+100000+j)
    results.append(x)
    print(n, sd, ratio, means, x["joint_success"])
Path(__file__).with_name("sensitivity-results.json").write_text(json.dumps(results, indent=2)+"\n")
