use super::*;
use crate::AspicError;

// Retain source diagnostic category labels without changing the native
// machine-readable Invalid/Incomplete distinction.
fn execution_error(error: AspicError) -> AspicError {
    match error {
        AspicError::Incomplete(message) => AspicError::Incomplete(format!(
            "Bounded ASPIC execution failed: BudgetExceeded: {message}"
        )),
        AspicError::Invalid(message) => AspicError::Invalid(format!(
            "Bounded ASPIC execution failed: Method execution failed: ValueError: {message}"
        )),
    }
}

/// Immutable, checked observation snapshot. Detached JSON getters cannot alter
/// the private provenance proof. There is no unchecked public constructor or
/// deserializer for independently supplied formal theories and source maps.
#[derive(Clone, Debug)]
pub struct CompiledTheory {
    theory: Value,
    source_map: Value,
    assessment: Value,
    limits: ExecutionLimits,
    integrity: [String; 3],
}
impl CompiledTheory {
    pub(super) fn new(
        theory: Value,
        source_map: Value,
        assessment: Value,
        limits: ExecutionLimits,
    ) -> Result<Self, AspicError> {
        let integrity = [
            canonical::canonical_digest(&theory, &limits)?,
            canonical::canonical_digest(&source_map, &limits)?,
            canonical::canonical_digest(&assessment, &limits)?,
        ];
        let snapshot = Self {
            theory,
            source_map,
            assessment,
            limits,
            integrity,
        };
        snapshot.checked()?;
        Ok(snapshot)
    }
    fn checked(&self) -> Result<(), AspicError> {
        let check = || -> Result<(), String> {
            self.limits.validate()?;
            let digests = [
                canonical::canonical_digest(&self.theory, &self.limits)?,
                canonical::canonical_digest(&self.source_map, &self.limits)?,
                canonical::canonical_digest(&self.assessment, &self.limits)?,
            ];
            if digests != self.integrity {
                return Err("compiled theory, assessment or source map was modified".into());
            }
            let mapping = &self.source_map;
            let goal = mapping["goal"]["claim"]
                .as_str()
                .ok_or("Invalid compiled goal claim")?;
            if mapping["profile"] != PROFILE
                || mapping["theory_digest"] != digests[0]
                || mapping["assessment_digest"] != digests[2]
                || mapping["snapshot_digest"] != snapshot_digest(mapping, &self.limits)?
                || mapping["backend"] != backend(&self.limits)?
                || mapping["execution_limits"] != self.limits.describe()
                || mapping["goal"]["atom"] != self.theory["goal"]
                || mapping["claims"][goal]["atom"] != self.theory["goal"]
                || self.assessment["claims"].get(goal).is_none()
                || [
                    "source_digest",
                    "assessed_at",
                    "context_fingerprint",
                    "method_registry_fingerprint",
                ]
                .iter()
                .any(|key| mapping[*key] != self.assessment[*key])
            {
                return Err("compiled theory and provenance identities disagree".into());
            }
            Ok(())
        };
        check().map_err(|error| {
            AspicError::Invalid(format!("Compiled snapshot integrity check failed: {error}"))
        })
    }
    pub fn theory(&self) -> Result<Value, AspicError> {
        self.checked()?;
        Ok(self.theory.clone())
    }
    pub fn source_map(&self) -> Result<Value, AspicError> {
        self.checked()?;
        Ok(self.source_map.clone())
    }
    pub fn assessment(&self) -> Result<Value, AspicError> {
        self.checked()?;
        Ok(self.assessment.clone())
    }
    pub fn limits(&self) -> Result<ExecutionLimits, AspicError> {
        self.checked()?;
        Ok(self.limits.clone())
    }
    /// Solve under the same captured limits, retaining the independent authored assessment.
    pub fn solve(&self) -> Result<Value, AspicError> {
        self.checked()?;
        self.solve_checked()
    }
    fn solve_checked(&self) -> Result<Value, AspicError> {
        let contract = crate::registry::aspic_contract();
        let payload = json!({"theory":self.theory});
        if canonical::canonical_json_bytes(&payload, &self.limits)?.len() > contract.max_input_bytes
        {
            return Err(AspicError::Invalid(
                "Bounded ASPIC execution failed: Method input exceeds byte limit".into(),
            ));
        }
        let errors =
            methods::schema_errors_with_limits(&payload, &contract.input_schema, "$", &self.limits);
        if !errors.is_empty() {
            return Err(AspicError::Invalid(format!(
                "Bounded ASPIC execution failed: {}",
                errors.join("; ")
            )));
        }
        let formal = crate::solver::solve_aspic(&payload, &self.limits).map_err(execution_error)?;
        crate::registry::check_output_bytes(&formal, &self.limits).map_err(execution_error)?;
        let errors =
            methods::schema_errors_with_limits(&formal, &contract.output_schema, "$", &self.limits);
        if !errors.is_empty() {
            return Err(AspicError::Invalid(format!(
                "Bounded ASPIC execution failed: Method output contract violation: {}",
                errors.join("; ")
            )));
        }
        let mut by_rule: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        for argument in formal["arguments"]
            .as_array()
            .ok_or_else(|| AspicError::Invalid("ASPIC result has no arguments".into()))?
        {
            if let Some(rule) = argument["rule_id"].as_str() {
                by_rule
                    .entry(rule.into())
                    .or_default()
                    .push(argument["label"].clone());
            }
        }
        let mut routes = json!({});
        for (name, origin) in self.source_map["arguments"]
            .as_object()
            .ok_or_else(|| AspicError::Invalid("Compiled source map has no arguments".into()))?
        {
            let rule = origin["rule_id"].as_str().ok_or_else(|| {
                AspicError::Invalid("Compiled argument has no rule identity".into())
            })?;
            let labels = by_rule.get(rule).cloned().unwrap_or_default();
            let status = if labels.iter().any(|v| v == "in") {
                "accepted"
            } else if labels.iter().any(|v| v == "undecided") {
                "undecided"
            } else if labels.is_empty() {
                "unconstructed"
            } else {
                "rejected"
            };
            routes[name] =
                json!({"labels":labels,"status":status,"rule_id":rule,"emitted":origin["emitted"]});
        }
        let claim = self.source_map["goal"]["claim"]
            .as_str()
            .ok_or_else(|| AspicError::Invalid("Compiled goal has no claim identity".into()))?;
        let claim_status = if self.assessment["claims"][claim]["status"] == "out_of_scope" {
            "out_of_scope"
        } else {
            match formal["query_status"].as_str() {
                Some("accepted") => "supported",
                Some("rejected" | "undecided") => "contested",
                _ => "unsupported",
            }
        };
        Ok(
            json!({"profile":PROFILE,"formal":formal,"routes":routes,"claim":claim,"claim_status":claim_status,"source_map":self.source_map}),
        )
    }
    pub fn to_dict(&self) -> Result<Value, AspicError> {
        self.checked()?;
        let projected = self.solve_checked()?;
        let claim = projected["claim"]
            .as_str()
            .ok_or_else(|| AspicError::Invalid("Compiled result has no goal".into()))?;
        Ok(
            json!({"profile":PROFILE,"theory":self.theory,"source_map":self.source_map,"diagnostics":[],"formal":projected["formal"],"routes":projected["routes"],"claim":claim,"claim_status":projected["claim_status"],"authored_claim_status":self.assessment["claims"][claim]["status"],"source_digest":self.source_map["source_digest"],"snapshot_digest":self.source_map["snapshot_digest"]}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot() -> CompiledTheory {
        let context = json!({});
        compile_eal_aspic("language \"EAL/3\"\nenvironment lab {\nrequire true\n}\nclaim goal {\nstatement \"A bounded claim.\"\nenvironment lab\n}\n",&json!({}),CompileOptions::new("goal","2026-09-23T12:00:00Z",&context)).unwrap()
    }
    #[test]
    fn private_snapshot_mutations_and_reconstructed_forgeries_fail_integrity() {
        for part in 0..3 {
            let mut changed = snapshot();
            match part {
                0 => changed.theory["goal"] = json!("unrelated"),
                1 => changed.source_map["claims"]["goal"]["statement"] = json!("Changed meaning"),
                _ => changed.assessment["claims"]["goal"]["status"] = json!("out_of_scope"),
            }
            assert!(
                changed
                    .solve()
                    .unwrap_err()
                    .to_string()
                    .contains("snapshot integrity check failed")
            );
            assert!(changed.to_dict().is_err());
            assert!(changed.theory().is_err());
            assert!(changed.source_map().is_err());
            assert!(changed.assessment().is_err());
            assert!(changed.limits().is_err());
        }
        let original = snapshot();
        let mut mapping = original.source_map.clone();
        mapping["goal"]["claim"] = json!("another");
        mapping["snapshot_digest"] = json!(snapshot_digest(&mapping, &original.limits).unwrap());
        assert!(
            CompiledTheory::new(
                original.theory,
                mapping,
                original.assessment,
                original.limits
            )
            .unwrap_err()
            .to_string()
            .contains("provenance identities disagree")
        );
    }
}
