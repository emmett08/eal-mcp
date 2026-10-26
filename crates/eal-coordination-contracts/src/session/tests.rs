//! Independently specified protocol failures and directed visibility contracts.

use serde_json::{Value, json};

use super::*;
use crate::{
    Adaptation, ClientMessage, DebateConfig, Edge, MAX_PAYLOAD_DEPTH, ServerMessage, Topology,
};

fn spec(topology: Topology) -> SessionSpec {
    SessionSpec {
        schema_version: SCHEMA_VERSION.into(),
        debate: DebateConfig {
            participants: 4,
            rounds: 1,
            topology,
            adaptation: Adaptation::default(),
        },
        max_payload_bytes: 1024,
    }
}

fn result(round: usize) -> RoundResult {
    RoundResult {
        round,
        responses: (0..4)
            .map(|participant| PeerValue {
                participant,
                round,
                payload: json!({"sender":participant,"source_round":round}),
            })
            .collect(),
    }
}

#[test]
fn independent_pass_then_ring_has_only_predecessor_and_own_previous() {
    let mut session = Session::new(spec(Topology::Ring {})).unwrap();
    assert!(session.ready());
    assert!(!session.done());
    assert_eq!(session.total_calls(), 8);
    assert_eq!(session.rounds(), 2);
    let independent = session.dispatch().unwrap();
    assert!(!session.ready());
    assert_eq!(independent.round, 0);
    assert_eq!(independent.calls.len(), 4);
    for call in independent.calls {
        assert!(call.edges.is_empty());
        assert!(call.incoming.is_empty());
        assert!(call.previous.is_none());
        assert!(call.peers.is_empty());
    }
    // Responses arrive in reverse order. Routing must still use participant IDs.
    let mut responses = result(0);
    responses.responses.reverse();
    session.accept(responses).unwrap();
    assert_eq!(session.completed_calls(), 4);
    let peer_round = session.dispatch().unwrap();
    for call in peer_round.calls {
        let predecessor = (call.participant + 3) % 4;
        assert_eq!(call.round, 1);
        assert_eq!(call.edges.len(), 4);
        assert_eq!(call.incoming, vec![predecessor]);
        assert_eq!(call.peers.len(), 1);
        assert_eq!(call.peers[0].participant, predecessor);
        assert_eq!(call.peers[0].round, 0);
        assert_eq!(call.peers[0].payload["sender"], predecessor);
        assert_eq!(call.previous.unwrap()["sender"], call.participant);
    }
    session.accept(result(1)).unwrap();
    assert!(session.done());
    assert!(!session.ready());
    assert_eq!(session.completed_calls(), session.total_calls());
    assert!(session.dispatch().is_err());
    assert!(session.accept(result(1)).is_err());
}

#[test]
fn opaque_null_previous_payload_survives_the_actual_dispatch_wire_round_trip() {
    let mut session = Session::new(spec(Topology::Ring {})).unwrap();
    let independent = ServerMessage::Dispatch {
        dispatch: session.dispatch().unwrap(),
    };
    let wire = serde_json::to_value(&independent).unwrap();
    assert!(wire["dispatch"]["calls"][0].get("previous").is_none());
    assert_eq!(
        serde_json::from_value::<ServerMessage>(wire).unwrap(),
        independent
    );

    let mut responses = result(0);
    for response in &mut responses.responses {
        response.payload = Value::Null;
    }
    session.accept(responses).unwrap();
    let dependent = session.dispatch().unwrap();
    assert!(
        dependent
            .calls
            .iter()
            .all(|call| call.previous == Some(Value::Null))
    );
    let message = ServerMessage::Dispatch {
        dispatch: dependent,
    };
    let wire = serde_json::to_value(&message).unwrap();
    assert_eq!(
        wire["dispatch"]["calls"][0].get("previous"),
        Some(&Value::Null)
    );
    assert_eq!(
        serde_json::from_value::<ServerMessage>(wire).unwrap(),
        message
    );
}

#[test]
fn opaque_literals_and_exact_numbers_survive_client_and_server_wire_round_trips() {
    for source in [
        r#"{"$serde_json::private::Number":"1"}"#,
        r#"{"$serde_json::private::RawValue":"null"}"#,
        "9007199254740993123456789123456789",
        "1.0000000000000000000001",
        "-0",
        "null",
    ] {
        let expected =
            eal_collector_contracts::strict_json_exact(source.as_bytes(), Default::default())
                .unwrap();
        let mut responses = result(0);
        for response in &mut responses.responses {
            response.payload = expected.clone();
        }
        let client = ClientMessage::Results { result: responses };
        let bytes = serde_json::to_vec(&client).unwrap();
        let decoded: ClientMessage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, client, "client bytes: {source}");
        let exact = eal_collector_contracts::strict_json_exact(&bytes, Default::default()).unwrap();
        assert_eq!(
            serde_json::from_value::<ClientMessage>(exact).unwrap(),
            client,
            "client value: {source}"
        );
        let mut session = Session::new(spec(Topology::Ring {})).unwrap();
        session.dispatch().unwrap();
        let ClientMessage::Results { result } = decoded else {
            panic!("wrong decoded client kind");
        };
        session.accept(result).unwrap();
        let server = ServerMessage::Dispatch {
            dispatch: session.dispatch().unwrap(),
        };
        let bytes = serde_json::to_vec(&server).unwrap();
        let decoded: ServerMessage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, server, "server bytes: {source}");
        let exact = eal_collector_contracts::strict_json_exact(&bytes, Default::default()).unwrap();
        assert_eq!(
            serde_json::from_value::<ServerMessage>(exact).unwrap(),
            server,
            "server value: {source}"
        );
        let ServerMessage::Dispatch { dispatch } = decoded else {
            panic!("wrong decoded server kind");
        };
        for call in dispatch.calls {
            assert_eq!(call.previous, Some(expected.clone()));
            assert_eq!(call.peers[0].payload, expected);
        }
    }
}

#[test]
fn payload_shape_limits_precede_serialisation_and_preserve_the_round_barrier() {
    let nested = |depth| (0..depth).fold(Value::Null, |child, _| Value::Array(vec![child]));
    let mut session = Session::new(spec(Topology::Ring {})).unwrap();
    session.dispatch().unwrap();
    let mut excessive_depth = result(0);
    excessive_depth.responses[3].payload = nested(MAX_PAYLOAD_DEPTH + 1);
    let error = session.accept(excessive_depth).unwrap_err();
    assert!(error.to_string().contains("maximum depth"));
    assert_eq!(session.completed_calls(), 0);
    assert!(!session.ready());
    let mut maximum_depth = result(0);
    maximum_depth.responses[3].payload = nested(MAX_PAYLOAD_DEPTH);
    session.accept(maximum_depth).unwrap();
    assert_eq!(session.completed_calls(), 4);

    let mut selected = spec(Topology::Ring {});
    selected.max_payload_bytes = 16;
    let mut session = Session::new(selected).unwrap();
    session.dispatch().unwrap();
    let mut too_many_nodes = result(0);
    for response in &mut too_many_nodes.responses {
        response.payload = Value::Null;
    }
    too_many_nodes.responses[3].payload = Value::Array(vec![Value::Null; 16]);
    let error = session.accept(too_many_nodes).unwrap_err();
    assert!(error.to_string().contains("maximum node count"));
    assert_eq!(session.completed_calls(), 0);
}

#[test]
fn results_require_dispatch_and_dispatch_requires_the_complete_round() {
    let mut session = Session::new(spec(Topology::FullyConnected {})).unwrap();
    assert!(session.accept(result(0)).is_err());
    session.dispatch().unwrap();
    assert!(session.dispatch().is_err());
    let mut partial = result(0);
    partial.responses.pop();
    assert!(session.accept(partial).is_err());
    assert!(session.dispatch().is_err());
    assert_eq!(session.completed_calls(), 0);
    session.accept(result(0)).unwrap();
    let round = session.dispatch().unwrap();
    for call in round.calls {
        assert_eq!(call.peers.len(), 3);
        assert!(call.peers.iter().all(|peer| peer.round == 0));
        assert!(
            call.peers
                .iter()
                .all(|peer| peer.participant != call.participant)
        );
    }
}

#[test]
fn invalid_identities_never_partially_advance_a_round() {
    let mut invalid = vec![result(1)];
    let mut duplicate = result(0);
    duplicate.responses[3].participant = 0;
    invalid.push(duplicate);
    let mut unknown = result(0);
    unknown.responses[3].participant = 4;
    invalid.push(unknown);
    let mut late = result(0);
    late.responses[3].round = 1;
    invalid.push(late);
    let mut extra = result(0);
    extra.responses.push(extra.responses[0].clone());
    invalid.push(extra);
    for response in invalid {
        let mut session = Session::new(spec(Topology::Chain {})).unwrap();
        session.dispatch().unwrap();
        assert!(session.accept(response).is_err());
        assert!(!session.ready());
        assert_eq!(session.completed_calls(), 0);
        session.accept(result(0)).unwrap();
        let accepted = session.dispatch().unwrap();
        assert_eq!(accepted.round, 1);
        assert_eq!(accepted.calls[1].peers[0].payload["sender"], 0);
    }
}

#[test]
fn scheduled_graphs_replace_edges_only_after_complete_rounds() {
    let mut selected = spec(Topology::Ring {});
    selected.debate.rounds = 3;
    selected.debate.adaptation = Adaptation::Scheduled {
        round_edges: vec![vec![Edge { from: 0, to: 2 }], vec![]],
    };
    let mut session = Session::new(selected).unwrap();
    for round in 0..4 {
        let dispatch = session.dispatch().unwrap();
        assert_eq!(dispatch.round, round);
        if round == 2 {
            assert_eq!(dispatch.calls[2].incoming, vec![0]);
            assert_eq!(dispatch.calls[2].peers[0].round, 1);
            for participant in [0, 1, 3] {
                assert!(dispatch.calls[participant].peers.is_empty());
            }
        }
        if round == 3 {
            assert!(dispatch.calls.iter().all(|call| call.peers.is_empty()));
            assert!(dispatch.calls.iter().all(|call| call.previous.is_some()));
        }
        session.accept(result(round)).unwrap();
    }
    assert!(session.done());
    assert_eq!(session.completed_calls(), 16);
}

#[test]
fn payload_limit_counts_encoded_json_and_failed_results_remain_replaceable() {
    let mut selected = spec(Topology::Ring {});
    selected.max_payload_bytes = 4;
    let mut session = Session::new(selected).unwrap();
    session.dispatch().unwrap();
    let mut oversized = result(0);
    for response in &mut oversized.responses {
        response.payload = Value::Null;
    }
    oversized.responses[3].payload = json!("abc"); // Five encoded bytes.
    assert!(session.accept(oversized.clone()).is_err());
    assert_eq!(session.completed_calls(), 0);
    oversized.responses[3].payload = json!("ab");
    session.accept(oversized).unwrap();
    assert_eq!(session.completed_calls(), 4);
}

#[test]
fn admission_checks_every_bound_and_future_graph() {
    let baseline = spec(Topology::Ring {});
    let mut invalid = baseline.clone();
    invalid.schema_version = "eal.debate/2".into();
    assert!(Session::new(invalid).is_err());
    for max_payload_bytes in [0, MAX_PAYLOAD_BYTES + 1, usize::MAX] {
        let mut invalid = baseline.clone();
        invalid.max_payload_bytes = max_payload_bytes;
        assert!(Session::new(invalid).is_err());
    }
    let mut invalid = baseline.clone();
    invalid.debate.rounds = 2;
    invalid.debate.adaptation = Adaptation::Scheduled {
        round_edges: vec![vec![Edge { from: 4, to: 0 }]],
    };
    assert!(Session::new(invalid).is_err());
    let mut invalid = baseline;
    invalid.debate.participants = 17;
    assert!(Session::new(invalid).is_err());
}

#[test]
fn dense_dispatch_budget_fails_before_payload_replication() {
    let mut selected = spec(Topology::FullyConnected {});
    selected.debate.participants = 16;
    selected.max_payload_bytes = MAX_PAYLOAD_BYTES;
    let mut session = Session::new(selected).unwrap();
    session.dispatch().unwrap();
    // 16 accepted 256 KiB payloads become 256 copies in the dense dispatch.
    let payload = json!("x".repeat(262_144));
    let response = RoundResult {
        round: 0,
        responses: (0..16)
            .map(|participant| PeerValue {
                participant,
                round: 0,
                payload: payload.clone(),
            })
            .collect(),
    };
    session.accept(response).unwrap();
    let error = session.dispatch().err().unwrap();
    assert!(error.to_string().contains("frame limit"));
    assert!(session.ready());
    assert_eq!(session.completed_calls(), 16);
}

#[test]
fn wire_messages_reject_unknown_nested_fields() {
    let start = ClientMessage::Start {
        spec: spec(Topology::Ring {}),
    };
    let encoded = serde_json::to_value(start).unwrap();
    for path in [vec![], vec!["spec"], vec!["spec", "debate"]] {
        let mut invalid = encoded.clone();
        let mut object = &mut invalid;
        for part in path {
            object = &mut object[part];
        }
        object["unknown"] = json!(true);
        assert!(serde_json::from_value::<ClientMessage>(invalid).is_err());
    }
    let results = ClientMessage::Results { result: result(0) };
    let mut invalid = serde_json::to_value(results).unwrap();
    invalid["result"]["responses"][0]["unknown"] = json!(true);
    assert!(serde_json::from_value::<ClientMessage>(invalid).is_err());
    for invalid in [
        json!({"kind":"completed","rounds":2,"calls":8,"unknown":true}),
        json!({"kind":"failed","message":"failed","unknown":true}),
    ] {
        assert!(serde_json::from_value::<ServerMessage>(invalid).is_err());
    }
}
