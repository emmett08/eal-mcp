//! Bind worker-authored messages and answers to findings issued through their role gateway.
use super::*;

impl Coordinator {
    pub(super) fn admit(
        &self,
        role: &str,
        call_id: &str,
        context: &Value,
        value: &Value,
    ) -> Result<Vec<Value>, String> {
        closed(
            value,
            if role == "assessor" {
                &["message", "consumed_message_ids"]
            } else {
                &["answer", "consumed_message_ids"]
            },
            "Response",
        )?;
        let received = array(context, "received_messages")?;
        let ids = received
            .iter()
            .map(|message| string(message, "message_id").map(|id| json!(id)))
            .collect::<Result<Vec<_>, _>>()?;
        if value["consumed_message_ids"] != json!(ids) {
            return Err("Response fields or consumed message identities differ".into());
        }
        let identity = if role == "assessor" {
            let message = &value["message"];
            let kind = string(message, "kind")?;
            let mut fields = vec![
                "schema",
                "kind",
                "message_id",
                "task_id",
                "task_digest",
                "sender",
                "recipient",
                "reply_to",
                "attempt",
                "finding_id",
            ];
            match kind {
                "outcome" => fields.push("assessment_refs"),
                "disposition" => fields.extend(["code", "outcome"]),
                _ => {
                    return Err(
                        "Handoff differs from the closed coordination message contract".into(),
                    );
                }
            }
            closed(message, &fields, "Handoff")?;
            if message["schema"] != MESSAGE_SCHEMA
                || message["task_id"] != self.task.task_id
                || message["task_digest"] != self.anchor["task_digest"]
                || message["sender"] != "assessor"
                || message["recipient"] != "reviewer"
                || message["reply_to"] != "request"
                || message["attempt"].as_u64() != Some(0)
            {
                return Err("Handoff replaced a trusted task identity".into());
            }
            let id = string(message, "message_id")?;
            if id == "request"
                || id.is_empty()
                || id.len() > 128
                || !id.as_bytes()[0].is_ascii_alphanumeric()
                || !id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_.:/-".contains(&c))
            {
                return Err("Reply requires a distinct bounded message identity".into());
            }
            string(message, "finding_id")?
        } else {
            if !value["answer"].is_object() {
                return Err("Answer must be an object".into());
            }
            string(&value["answer"], "finding_id")?
        };
        let operation = if role == "assessor" {
            "query"
        } else {
            "lookup"
        };
        if !self.state.events()?.iter().any(|event| {
            event["kind"] == operation
                && event["data"]["call_id"] == call_id
                && event["data"]["finding_id"] == identity
        }) {
            return Err(
                "Worker must request its referenced finding through its role gateway".into(),
            );
        }
        let finding = self.issued(identity)?;
        if role == "assessor" {
            let message = &value["message"];
            if finding["check"]["ok"] == true {
                let rows = finding["assessments"]
                    .as_object()
                    .ok_or("Invalid finding assessments")?;
                let references = Value::Object(
                    rows.iter()
                        .map(|(claim, row)| (claim.clone(), row["assessment_id"].clone()))
                        .collect(),
                );
                if message["kind"] != "outcome" || message["assessment_refs"] != references {
                    return Err("Outcome must retain every actual assessment reference".into());
                }
            } else {
                let expected = if ["evidence_unavailable", "effect_outcome_unknown"]
                    .contains(&string(&finding["check"], "code")?)
                {
                    "incomplete"
                } else {
                    "rejected"
                };
                if message["kind"] != "disposition"
                    || message["code"] != finding["check"]["code"]
                    || message["outcome"] != expected
                {
                    return Err("Unassessed task requires its matching typed disposition".into());
                }
            }
            Ok(vec![message.clone()])
        } else {
            if received
                .first()
                .and_then(|value| value.get("finding_id"))
                .and_then(Value::as_str)
                != Some(identity)
            {
                return Err("Reviewer substituted the addressed finding".into());
            }
            let errors =
                crate::profiles::validate_answer(&self.anchor, &finding, &value["answer"])?;
            if !errors.is_empty() {
                return Err(crate::ledger::canonical_json(&json!(errors))?);
            }
            Ok(vec![])
        }
    }
}
