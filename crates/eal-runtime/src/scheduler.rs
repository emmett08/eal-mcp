//! Requested-order collection with explicit parallel grants and serial barriers.
use indexmap::IndexMap;
use std::{collections::BTreeSet, sync::mpsc, thread};

#[derive(Clone, Debug)]
pub struct CollectionScheduler {
    max_workers: usize,
}

impl Default for CollectionScheduler {
    fn default() -> Self {
        Self { max_workers: 4 }
    }
}

impl CollectionScheduler {
    pub fn new(max_workers: usize) -> Result<Self, String> {
        if !(1..=32).contains(&max_workers) {
            return Err("max_workers must be an integer from 1 through 32".into());
        }
        Ok(Self { max_workers })
    }
    pub fn max_workers(&self) -> usize {
        self.max_workers
    }

    /// Refill each freed slot immediately. All in-flight work is joined before
    /// returning, including on a callback error. The callback carries its own
    /// limits, context, cancellation and lease ownership; no thread globals apply.
    pub fn run<T, P, F>(
        &self,
        names: &[String],
        is_parallel_safe: P,
        collect_one: F,
    ) -> Result<IndexMap<String, T>, String>
    where
        T: Send,
        P: Fn(&str) -> bool,
        F: Fn(&str) -> Result<T, String> + Sync,
    {
        if names.iter().collect::<BTreeSet<_>>().len() != names.len() {
            return Err("Evidence identifiers must be unique".into());
        }
        let mut values: Vec<Option<T>> = (0..names.len()).map(|_| None).collect();
        let mut start = 0;
        while start < names.len() {
            if !is_parallel_safe(&names[start]) {
                values[start] = Some(collect_one(&names[start])?);
                start += 1;
                continue;
            }
            let mut end = start + 1;
            while end < names.len() && is_parallel_safe(&names[end]) {
                end += 1;
            }
            if end - start == 1 || self.max_workers == 1 {
                for index in start..end {
                    values[index] = Some(collect_one(&names[index])?);
                }
            } else {
                thread::scope(|scope| -> Result<(), String> {
                    let (sender, receiver) = mpsc::channel();
                    let launch =
                        |index: usize, sender: mpsc::Sender<(usize, Result<T, String>)>| {
                            let collect_one = &collect_one;
                            let name: &str = names[index].as_str();
                            scope.spawn(move || {
                                let outcome =
                                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        collect_one(name)
                                    }))
                                    .unwrap_or_else(|_| Err("Collection worker panicked".into()));
                                let _ = sender.send((index, outcome));
                            });
                        };
                    let mut next = start;
                    let mut active = 0;
                    while next < end && active < self.max_workers {
                        launch(next, sender.clone());
                        next += 1;
                        active += 1;
                    }
                    let mut failure = None;
                    while active != 0 {
                        let (index, result) = receiver
                            .recv()
                            .map_err(|_| "Collection worker disconnected")?;
                        active -= 1;
                        match result {
                            Ok(value) => values[index] = Some(value),
                            Err(error) if failure.is_none() => failure = Some(error),
                            Err(_) => {}
                        }
                        if failure.is_none() && next < end {
                            launch(next, sender.clone());
                            next += 1;
                            active += 1;
                        }
                    }
                    failure.map_or(Ok(()), Err)
                })?;
            }
            start = end;
            if start < names.len() {
                // The scan already classified this barrier as unsafe. Resolve
                // each host grant exactly once, as in the source scheduler.
                values[start] = Some(collect_one(&names[start])?);
                start += 1;
            }
        }
        Ok(names
            .iter()
            .cloned()
            .zip(
                values.into_iter().map(|value| {
                    value.expect("each successfully scheduled request produced a result")
                }),
            )
            .collect())
    }
}
