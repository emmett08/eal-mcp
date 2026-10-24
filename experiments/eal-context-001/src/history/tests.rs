//! Independent assertions for the original six history-policy obligations.
use super::*;
fn reference(revision: &str) -> Reference {
    Reference {
        entry_id: "entry".into(),
        claim: "ready".into(),
        assessment_id: format!("assessment-{revision}"),
        source_digest: format!("source-{revision}"),
        context_fingerprint: "context".into(),
        method_registry_fingerprint: "methods".into(),
        assessed_at: format!("2040-01-01T00:00:0{revision}Z"),
    }
}
fn history(condition: char) -> History {
    History::new(
        condition,
        "immutable brief",
        json!({"ready":"release criterion"}),
        vec![json!({"name":"fixed tool"})],
    )
    .unwrap()
}
#[test]
fn packet_age_eviction_preserves_complete_exchange_and_ordinary_turn() {
    for condition in ['A', 'B', 'C'] {
        let mut host = history(condition);
        for turn in 1..=4 {
            host.begin_turn(&format!("user {turn}")).unwrap();
            if turn == 1 {
                host.add_packet(
                    &reference("0"),
                    exchange("call-1", "synthetic packet"),
                    DEFAULT_LIMITS,
                )
                .unwrap();
            }
            host.finish_turn(&format!("answer {turn}")).unwrap();
        }
        assert!(host.turns[0].blocks.iter().any(|b| b.packet));
        host.begin_turn("user 5").unwrap();
        let request = host.request();
        let messages = request["messages"].as_array().unwrap();
        assert!(messages.contains(&json!({"role":"user","content":"user 1"})));
        assert!(messages.contains(&json!({"role":"assistant","content":"answer 1"})));
        let calls = messages
            .iter()
            .filter(|message| message.get("tool_calls").is_some())
            .count();
        let results = messages
            .iter()
            .filter(|message| message["role"] == "tool")
            .count();
        assert_eq!(calls, usize::from(condition == 'A'));
        assert_eq!(results, calls);
    }
}
#[test]
fn omitted_detail_requires_explicit_calls_and_failed_retrieval_consumes_ladder() {
    let mut host = history('C');
    host.begin_turn("omitted").unwrap();
    host.add_packet(
        &reference("0"),
        exchange("call-1", "{\"summary_complete\":false}"),
        DEFAULT_LIMITS,
    )
    .unwrap();
    assert_eq!(host.server_reloads, 0);
    let mut limits = vec![];
    for _ in RELOAD_LADDER {
        host.reload(&reference("0"), |_, selected| {
            limits.push(selected);
            Ok(())
        })
        .unwrap();
    }
    assert_eq!(limits, RELOAD_LADDER);
    assert_eq!(
        host.reload(&reference("0"), |_, _| Ok(())),
        Err(HistoryError::RetrievalExhausted)
    );
    host.finish_turn("qualified synthetic placeholder").unwrap();
    host.begin_turn("next").unwrap();
    assert_eq!(
        host.reload(&reference("0"), |_, selected| Ok(selected))
            .unwrap(),
        RELOAD_LADDER[0]
    );
    assert_eq!(
        history('A').cache_lookup(&reference("0")),
        Err(HistoryError::Permission)
    );
    let mut broken = history('C');
    broken.begin_turn("failed request").unwrap();
    broken
        .add_packet(&reference("0"), exchange("c", "x"), DEFAULT_LIMITS)
        .unwrap();
    for _ in RELOAD_LADDER {
        assert_eq!(
            broken.reload::<()>(&reference("0"), |_, _| Err("transport failed".into())),
            Err(HistoryError::Retrieval("transport failed".into()))
        );
    }
    assert_eq!(broken.server_reloads, 3);
    assert_eq!(
        broken.reload(&reference("0"), |_, _| Ok(())),
        Err(HistoryError::RetrievalExhausted)
    );
}
#[test]
fn revision_cache_and_index_preserve_original_identity_without_summary() {
    for condition in ['A', 'B'] {
        let mut host = history(condition);
        host.begin_turn("default").unwrap();
        assert!(
            host.add_packet(&reference("0"), exchange("c", "x"), RELOAD_LADDER[0])
                .is_err()
        );
    }
    let mut host = history('B');
    host.begin_turn("original").unwrap();
    let original = exchange("c1", "historical semantic payload");
    host.add_packet(&reference("0"), original.clone(), DEFAULT_LIMITS)
        .unwrap();
    host.finish_turn("answer").unwrap();
    host.begin_turn("revision").unwrap();
    host.add_packet(
        &reference("1"),
        exchange("c2", "revised payload"),
        DEFAULT_LIMITS,
    )
    .unwrap();
    let mut retrieved = host.cache_lookup(&reference("0")).unwrap();
    retrieved[1]["content"] = json!("mutation");
    assert_eq!(host.cache_lookup(&reference("0")).unwrap(), original);
    let request = host.request();
    let index = request["messages"][2]["content"].as_str().unwrap();
    assert!(!index.contains("historical semantic payload"));
    assert!(!index.contains("revised payload"));
    assert!(index.contains(&reference("0").assessed_at));
    assert_eq!(host.index.len(), 2);
    assert!(history('A').cache.is_empty());
    assert!(history('C').cache.is_empty());
    let mut altered = reference("0");
    altered.assessed_at = "changed".into();
    assert!(
        host.add_packet(&altered, exchange("c3", "x"), DEFAULT_LIMITS)
            .is_err()
    );
}
#[test]
fn emergency_eviction_preserves_protected_index_and_current_turn() {
    let mut host = history('A');
    host.begin_turn("old question").unwrap();
    host.add_packet(
        &reference("0"),
        exchange("c1", &"x".repeat(10_000)),
        DEFAULT_LIMITS,
    )
    .unwrap();
    host.finish_turn("old answer").unwrap();
    host.begin_turn("current question").unwrap();
    let cap = compact_bytes(&host.request()) - 5000;
    let request = host.prepare(|value| Ok(compact_bytes(value)), cap).unwrap();
    let text = request.to_string();
    for required in ["immutable brief", "current question", "reference_index"] {
        assert!(text.contains(required));
    }
    for absent in ["old question", "old answer", "tool_call_id"] {
        assert!(!text.contains(absent));
    }
    assert_eq!(
        host.prepare(|value| Ok(compact_bytes(value)), 1),
        Err(HistoryError::ContextOverflow)
    );
    assert!(
        host.prepare(|_| Err(HistoryError::Invalid("bad counter")), 1000)
            .is_err()
    );
}
#[test]
fn timeout_and_cycle_exhaustion_keep_failure_penalty() {
    let mut budget = TurnBudget::new(0.0);
    for _ in 0..6 {
        budget.before_cycle(1.0).unwrap();
    }
    assert_eq!(
        budget.before_cycle(2.0),
        Err(HistoryError::RetrievalExhausted)
    );
    assert_eq!(
        TurnBudget::new(10.0).before_cycle(130.0),
        Err(HistoryError::Timeout)
    );
    assert_eq!(
        deadline_restricted_latency(None, false, 120.0).unwrap(),
        120.0
    );
    assert_eq!(
        deadline_restricted_latency(Some(2.0), false, 120.0).unwrap(),
        120.0
    );
    assert_eq!(
        deadline_restricted_latency(Some(2.0), true, 120.0).unwrap(),
        2.0
    );
    assert!(deadline_restricted_latency(Some(f64::NAN), true, 120.0).is_err());
}
#[test]
fn missing_telemetry_stays_missing_and_reasoning_is_counted_once() {
    assert!(normalise_usage(None, true).unwrap()["total_tokens"].is_null());
    assert_eq!(
        normalise_usage(Some(&json!({"input_tokens":0,"output_tokens":0})), true).unwrap()["total_tokens"],
        0
    );
    let usage = json!({"input_tokens":100,"output_tokens":20,"cached_input_tokens":60,"reasoning_tokens":5});
    assert_eq!(
        normalise_usage(Some(&usage), true).unwrap()["total_tokens"],
        120
    );
    assert_eq!(
        normalise_usage(Some(&usage), false).unwrap()["total_tokens"],
        125
    );
    assert!(
        normalise_usage(Some(&json!({"input_tokens":100})), true).unwrap()["total_tokens"]
            .is_null()
    );
    assert!(
        normalise_usage(
            Some(&json!({"input_tokens":1,"cached_input_tokens":2})),
            true
        )
        .is_err()
    );
    assert!(normalise_usage(Some(&json!({"input_tokens":true})), true).is_err());
}
#[test]
fn partial_duplicate_or_reordered_tool_exchanges_are_rejected_atomically() {
    let mut host = history('C');
    host.begin_turn("turn").unwrap();
    let original = host.request();
    for messages in [
        vec![exchange("c", "x")[0].clone()],
        vec![exchange("c", "x")[1].clone()],
        {
            let mut messages = exchange("c", "x");
            messages.extend(exchange("c", "y"));
            messages
        },
    ] {
        assert!(
            host.add_packet(&reference("0"), messages, DEFAULT_LIMITS)
                .is_err()
        );
        assert_eq!(host.request(), original);
    }
}
#[test]
fn usage_retains_arbitrary_nonnegative_integer_precision() {
    let raw: Value = serde_json::from_str(
        "{\"input_tokens\":18446744073709551616,\"output_tokens\":18446744073709551616}",
    )
    .unwrap();
    assert_eq!(
        normalise_usage(Some(&raw), true).unwrap()["total_tokens"].to_string(),
        "36893488147419103232"
    );
    assert!(normalise_usage(Some(&json!({"input_tokens":1.0})), true).is_err());
}
