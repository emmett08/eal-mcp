//! Iterative acyclicity and longest-path checks for premise dependencies.
use super::Checker;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

impl Checker<'_> {
    pub(super) fn dependencies(&mut self) {
        let p = self.program;
        let mut graph: BTreeMap<_, BTreeSet<_>> = p
            .claims
            .keys()
            .map(|name| (name.clone(), BTreeSet::new()))
            .collect();
        for argument in p.arguments.values() {
            if let Some(dependencies) = graph.get_mut(&argument.conclusion) {
                dependencies.extend(
                    argument
                        .premises
                        .iter()
                        .filter(|name| p.claims.contains_key(*name))
                        .cloned(),
                );
            }
        }
        let mut incoming: BTreeMap<_, BTreeSet<_>> = graph
            .keys()
            .map(|name| (name.clone(), BTreeSet::new()))
            .collect();
        let mut remaining: BTreeMap<_, _> = graph
            .iter()
            .map(|(name, deps)| (name.clone(), deps.len()))
            .collect();
        let mut depth: BTreeMap<_, usize> = graph.keys().map(|name| (name.clone(), 0)).collect();
        for (name, deps) in &graph {
            for dependency in deps {
                incoming.get_mut(dependency).unwrap().insert(name.clone());
            }
        }
        let mut ready: VecDeque<_> = p
            .claims
            .keys()
            .filter(|name| remaining[*name] == 0)
            .cloned()
            .collect();
        let mut visited = 0usize;
        while let Some(name) = ready.pop_front() {
            visited += 1;
            for dependent in &incoming[&name] {
                let candidate = depth[&name] + 1;
                let value = depth.get_mut(dependent).unwrap();
                *value = (*value).max(candidate);
                let count = remaining.get_mut(dependent).unwrap();
                *count -= 1;
                if *count == 0 {
                    ready.push_back(dependent.clone());
                }
            }
        }
        if visited != graph.len()
            && let Some(blocked) = p.claims.keys().find(|name| remaining[*name] > 0)
        {
            self.error(
                "dependency_cycle",
                "Premise claims must form an acyclic dependency graph",
                Some(blocked),
            );
        }
        if let Some((name, count)) =
            p.claims
                .keys()
                .map(|name| (name, &depth[name]))
                .fold(None, |best, item| match best {
                    Some((_, count)) if count >= item.1 => best,
                    _ => Some(item),
                })
            && *count > p.limits.premise_depth
        {
            self.error(
                "resource_limit",
                format!(
                    "Premise chains may contain at most {} edges",
                    p.limits.premise_depth
                ),
                Some(name),
            );
        }
    }
}
