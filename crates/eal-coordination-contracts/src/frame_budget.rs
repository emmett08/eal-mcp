//! Encoded dispatch accounting before dense graphs replicate opaque payloads.

use crate::{Dispatch, Error, MAX_FRAME_BYTES, ServerMessage};

pub(crate) fn check(skeleton: &Dispatch, previous_payload_bytes: &[usize]) -> Result<(), Error> {
    let bytes = encoded_size(skeleton, previous_payload_bytes)?;
    if bytes > MAX_FRAME_BYTES {
        return Err(Error::Protocol(format!(
            "dispatch requires {bytes} encoded bytes, exceeding frame limit {MAX_FRAME_BYTES}"
        )));
    }
    Ok(())
}

/// Skeleton payload slots contain null. All other metadata is already exact.
fn encoded_size(skeleton: &Dispatch, previous_payload_bytes: &[usize]) -> Result<usize, Error> {
    let skeleton_bytes = serde_json::to_vec(&ServerMessage::Dispatch {
        dispatch: skeleton.clone(),
    })
    .map_err(|error| Error::Protocol(format!("dispatch cannot be encoded: {error}")))?
    .len();
    let (slots, payload_bytes) = skeleton.calls.iter().fold((0, 0), |total, call| {
        let own_slots = usize::from(call.previous.is_some());
        let own_bytes = if own_slots == 1 {
            previous_payload_bytes[call.participant]
        } else {
            0
        };
        let peer_bytes: usize = call
            .incoming
            .iter()
            .map(|&sender| previous_payload_bytes[sender])
            .sum();
        (
            total.0 + own_slots + call.peers.len(),
            total.1 + own_bytes + peer_bytes,
        )
    });
    // The transport limit includes the terminating JSON-lines newline.
    Ok(skeleton_bytes - 4 * slots + payload_bytes + 1)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::{CallContext, Edge, PeerValue};

    #[test]
    fn accounting_matches_actual_wire_bytes_for_repeated_escaped_and_short_values() {
        let payloads = [json!(1), json!("a\n\"β"), json!({"x":[true, null]})];
        let previous_payload_bytes: Vec<_> = payloads
            .iter()
            .map(|value| serde_json::to_vec(value).unwrap().len())
            .collect();
        let calls = (0..3)
            .map(|participant| CallContext {
                participant,
                round: 1,
                edges: vec![Edge { from: 2, to: 0 }],
                incoming: vec![2],
                previous: Some(payloads[participant].clone()),
                peers: vec![PeerValue {
                    participant: 2,
                    round: 0,
                    payload: payloads[2].clone(),
                }],
            })
            .collect();
        let actual = Dispatch { round: 1, calls };
        let measured = serde_json::to_vec(&ServerMessage::Dispatch {
            dispatch: actual.clone(),
        })
        .unwrap()
        .len()
            + 1;
        let mut skeleton = actual;
        for call in &mut skeleton.calls {
            call.previous = Some(Value::Null);
            for peer in &mut call.peers {
                peer.payload = Value::Null;
            }
        }
        assert_eq!(
            encoded_size(&skeleton, &previous_payload_bytes).unwrap(),
            measured
        );
    }
}
