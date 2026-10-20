use super::translation::Translation;
use super::*;
impl Translation<'_> {
    pub(super) fn observations(&mut self) -> Result<(), AspicError> {
        let program = self.program;
        let records = self.records;
        let assessment = self.assessment;
        let limits = &program.limits;
        let claims = &self.claims;
        let evidence = &self.evidence;
        let assumptions = &self.assumptions;
        let annotations = &self.annotations;
        let rank = |name: &str| annotations.rank(name);
        let annotated = |kind: &str, name: &str| annotations.annotated(kind, name);
        let mapping = &mut self.mapping;
        let premises = &mut self.premises;
        let rules = &mut self.rules;
        for name in claims.keys() {
            mapping["claims"][*name] = json!({"atom":claims[name],"statement":program.claims[*name].statement,"environment":program.claims[*name].environment,"span":location(program,name)});
        }
        for name in evidence.keys() {
            let verdict = &assessment["evidence"][*name];
            let available = verdict["status"] == "available";
            let supplied = records.get(*name).filter(|record| !record.is_null());
            let label = repr(name, limits)?;
            let record_digest = supplied
                .map(|record| canonical::canonical_digest(record, limits))
                .transpose()
                .map_err(|error| format!("Evidence {label} cannot be fingerprinted: {error}"))?;
            let record = if available { supplied } else { None };
            if record.is_some_and(|record| !record["tool_binding_digest"].is_string()) {
                return Err(format!(
                    "Available evidence {} lacks a binding digest",
                    repr(name, limits)?
                )
                .into());
            }
            let identity=record.map_or(Value::Null,|record|json!({"run_id":record["run_id"],"data_digest":record["data_digest"],"request_digest":record["request_digest"],"collected_at":record["collected_at"],"tool_binding_digest":record["tool_binding_digest"]}));
            mapping["evidence"][*name] = json!({"atom":evidence[name],"available":available,"identity":identity,"record_digest":record_digest,"reasons":verdict["reasons"],"availability_issues":verdict["availability_issues"],"rank":rank(name),"rank_annotation":annotated("rank",name)?,"span":location(program,name)});
            if available {
                premises.push(json!({"atom":evidence[name],"kind":"ordinary","rank":rank(name)}));
            }
        }
        for name in assumptions.keys() {
            let assumption = &program.assumptions[*name];
            let supported = matches!(
                assessment["assumptions"][*name]["status"].as_str(),
                Some("supported" | "contested")
            );
            let rule_id = symbol("v", name);
            let applicability = symbol("x", &rule_id);
            mapping["assumptions"][*name] = json!({"atom":assumptions[name],"rule_id":rule_id,"applicability_atom":applicability,"validation":assumption.validation,"rank":rank(name),"rank_annotation":annotated("rank",name)?,"emitted":supported,"span":location(program,name)});
            if supported {
                rules.push(json!({"id":rule_id,"kind":"defeasible","antecedents":[evidence[assumption.validation.as_str()]],"consequent":assumptions[name],"name":applicability,"rank":rank(name)}));
            }
        }

        Ok(())
    }
}
