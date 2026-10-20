use eal_runtime::scheduler::CollectionScheduler;
use std::{
    sync::{
        Arc, Barrier, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).into()).collect()
}

#[test]
fn unapproved_calls_stay_on_the_caller_in_requested_order() {
    let caller = thread::current().id();
    let visited = Mutex::new(Vec::new());
    let requested = names(&["first", "second", "third"]);
    let results = CollectionScheduler::new(4)
        .unwrap()
        .run(
            &requested,
            |_| false,
            |name| {
                assert_eq!(thread::current().id(), caller);
                visited.lock().unwrap().push(name.to_owned());
                Ok(name.to_uppercase())
            },
        )
        .unwrap();
    assert_eq!(*visited.lock().unwrap(), requested);
    assert_eq!(results.keys().cloned().collect::<Vec<_>>(), requested);
    assert_eq!(
        results.values().cloned().collect::<Vec<_>>(),
        names(&["FIRST", "SECOND", "THIRD"])
    );
}

#[test]
fn explicit_parallel_runs_overlap_and_serial_calls_form_barriers() {
    let first = Barrier::new(2);
    let second = Barrier::new(2);
    let completed = Mutex::new(Vec::new());
    let active = AtomicUsize::new(0);
    let maximum = AtomicUsize::new(0);
    let requested = names(&["a", "b", "unsafe", "d", "e"]);
    let results = CollectionScheduler::new(2)
        .unwrap()
        .run(
            &requested,
            |name| name != "unsafe",
            |name| {
                let running = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum.fetch_max(running, Ordering::SeqCst);
                match name {
                    "a" | "b" => {
                        first.wait();
                    }
                    "d" | "e" => {
                        second.wait();
                    }
                    _ => {
                        assert_eq!(running, 1);
                    }
                }
                completed.lock().unwrap().push(name.to_owned());
                active.fetch_sub(1, Ordering::SeqCst);
                Ok(name.to_uppercase())
            },
        )
        .unwrap();
    let completed = completed.lock().unwrap();
    assert_eq!(completed[2], "unsafe");
    assert!(completed[..2].iter().all(|n| n == "a" || n == "b"));
    assert!(completed[3..].iter().all(|n| n == "d" || n == "e"));
    assert_eq!(maximum.load(Ordering::SeqCst), 2);
    assert_eq!(results.keys().cloned().collect::<Vec<_>>(), requested);
}

#[test]
fn a_freed_slot_refills_before_a_slower_call_finishes() {
    let state = Arc::new((Mutex::new((false, false)), Condvar::new()));
    let requested = names(&["slow", "fast", "third"]);
    let result = CollectionScheduler::new(2)
        .unwrap()
        .run(
            &requested,
            |_| true,
            |name| {
                let (lock, event) = &*state;
                let mut state = lock.lock().unwrap();
                match name {
                    "slow" => {
                        state.0 = true;
                        event.notify_all();
                        let deadline = Instant::now() + Duration::from_secs(3);
                        while !state.1 {
                            let (pair, timeout) = event
                                .wait_timeout(
                                    state,
                                    deadline.saturating_duration_since(Instant::now()),
                                )
                                .unwrap();
                            state = pair;
                            assert!(
                                !timeout.timed_out() || state.1,
                                "third call did not refill the fast slot"
                            );
                        }
                    }
                    "fast" => {
                        while !state.0 {
                            state = event.wait(state).unwrap();
                        }
                    }
                    _ => {
                        state.1 = true;
                        event.notify_all();
                    }
                }
                Ok(name.to_owned())
            },
        )
        .unwrap();
    assert_eq!(result.keys().cloned().collect::<Vec<_>>(), requested);
}

#[test]
fn invalid_configuration_and_duplicates_fail_before_callbacks() {
    assert!(CollectionScheduler::new(0).is_err());
    assert!(CollectionScheduler::new(33).is_err());
    let calls = AtomicUsize::new(0);
    let result = CollectionScheduler::default().run(
        &names(&["same", "same"]),
        |_| true,
        |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    );
    assert!(result.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn callback_failure_joins_inflight_work_before_returning() {
    let rendezvous = Barrier::new(2);
    let ended = AtomicUsize::new(0);
    let result = CollectionScheduler::new(2).unwrap().run(
        &names(&["failure", "other"]),
        |_| true,
        |name| {
            rendezvous.wait();
            ended.fetch_add(1, Ordering::SeqCst);
            if name == "failure" {
                Err("expected failure".into())
            } else {
                Ok(name.to_owned())
            }
        },
    );
    assert_eq!(result.unwrap_err(), "expected failure");
    assert_eq!(ended.load(Ordering::SeqCst), 2);
}

#[test]
fn each_parallel_grant_is_resolved_once_in_request_order() {
    let classified = Mutex::new(Vec::new());
    let requested = names(&["parallel-a", "parallel-b", "barrier", "parallel-c"]);
    CollectionScheduler::new(2)
        .unwrap()
        .run(
            &requested,
            |name| {
                classified.lock().unwrap().push(name.to_owned());
                name != "barrier"
            },
            |name| Ok(name.to_owned()),
        )
        .unwrap();
    assert_eq!(*classified.lock().unwrap(), requested);
}
