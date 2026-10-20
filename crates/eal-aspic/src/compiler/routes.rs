use super::translation::Translation;
use super::*;
impl Translation<'_> {
    pub(super) fn arguments(&mut self) -> Result<(), AspicError> {
        let program = self.program;
        let assessment = self.assessment;
        let claims = &self.claims;
        let evidence = &self.evidence;
        let assumptions = &self.assumptions;
        let annotations = &self.annotations;
        let rank = |name: &str| annotations.rank(name);
        let annotated = |kind: &str, name: &str| annotations.annotated(kind, name);
        let mapping = &mut self.mapping;
        let rules = &mut self.rules;
        let structural = &self.structural;
        let basis_needed = &mut self.basis_needed;
        for name in sorted(program.arguments.keys()) {
            let argument = &program.arguments[name];
            let method = &program.reasoning[&argument.reasoning];
            let antecedents = unique(
                argument
                    .evidence
                    .iter()
                    .chain(&method.backing)
                    .map(|name| evidence[name.as_str()].clone())
                    .chain(
                        argument
                            .assumptions
                            .iter()
                            .map(|name| assumptions[name.as_str()].clone()),
                    )
                    .chain(
                        argument
                            .premises
                            .iter()
                            .map(|name| claims[name.as_str()].clone()),
                    ),
            );
            let eligible = assessment["arguments"][name]["locally_usable"] == true;
            let strict = annotations.entries.contains_key(&("strict", name));
            let rule_id = symbol("r", name);
            let applicability = symbol("x", &rule_id);
            mapping["arguments"][name] = json!({"rule_id":rule_id,"applicability_atom":if strict{Value::Null}else{json!(applicability)},"conclusion_atom":claims[argument.conclusion.as_str()],"conclusion":argument.conclusion,"antecedents":antecedents,"emitted":eligible,"reasoning":argument.reasoning,"rationale":method.rationale,"rule_kind":if strict{"strict"}else{"defeasible"},"strict_annotation":annotated("strict",name)?,"rank":if strict{Value::Null}else{json!(rank(name))},"rank_annotation":annotated("rank",name)?,"origin":argument.origin,"span":location(program,name)});
            if eligible {
                let mut actual = antecedents;
                if actual.is_empty() {
                    actual.push(structural.clone());
                    *basis_needed = true;
                }
                let mut rule = json!({"id":rule_id,"kind":if strict{"strict"}else{"defeasible"},"antecedents":actual,"consequent":claims[argument.conclusion.as_str()]});
                if !strict {
                    rule["name"] = json!(applicability);
                    rule["rank"] = json!(rank(name));
                }
                rules.push(rule);
            }
        }

        Ok(())
    }
    pub(super) fn objections(&mut self) -> Result<(), AspicError> {
        let program = self.program;
        let assessment = self.assessment;
        let limits = &program.limits;
        let claims = &self.claims;
        let evidence = &self.evidence;
        let objections = &self.objections;
        let annotations = &self.annotations;
        let rank = |name: &str| annotations.rank(name);
        let annotated = |kind: &str, name: &str| annotations.annotated(kind, name);
        let mapping = &mut self.mapping;
        let rules = &mut self.rules;
        let contraries = &mut self.contraries;
        let structural = &self.structural;
        let basis_needed = &mut self.basis_needed;
        let scopes = validation::objection_scopes(program);
        for name in sorted(program.objections.keys()) {
            let objection = &program.objections[name];
            let scope = scopes[name]
                .first()
                .ok_or("Objection has no checked environment scope")?;
            let antecedents = unique(
                objection
                    .evidence
                    .iter()
                    .map(|name| evidence[name.as_str()].clone())
                    .chain(
                        objection
                            .premises
                            .iter()
                            .map(|name| claims[name.as_str()].clone()),
                    ),
            );
            let eligible = assessment["environments"][scope]["status"] == "matched"
                && objection
                    .evidence
                    .iter()
                    .all(|name| mapping["evidence"][name]["available"] == true);
            let rule_id = symbol("j", name);
            let applicability = symbol("x", &rule_id);
            let mut targets: Vec<(&str, &str)> = Vec::new();
            match objection.target_kind.as_str() {
                "objection" => targets.push(("objection", &objection.target)),
                "assumption" => targets.push(("assumption", &objection.target)),
                _ => {
                    for route in sorted(program.arguments.keys()) {
                        let argument = &program.arguments[route];
                        if program.claims[&argument.conclusion].environment != *scope {
                            continue;
                        }
                        if match objection.target_kind.as_str() {
                            "claim" => argument.conclusion == objection.target,
                            "argument" => route == objection.target,
                            "reasoning" => argument.reasoning == objection.target,
                            _ => false,
                        } {
                            targets.push(("argument", route));
                        }
                    }
                }
            }
            mapping["objections"][name] = json!({"atom":objections[name],"rule_id":rule_id,"applicability_atom":applicability,"target_kind":objection.target_kind,"target":objection.target,"targets":targets.iter().map(|(kind,name)|json!({"kind":kind,"name":name})).collect::<Vec<_>>(),"antecedents":antecedents,"rank":rank(name),"rank_annotation":annotated("rank",name)?,"emitted":eligible,"span":location(program,name)});
            if eligible {
                let mut actual = antecedents;
                if actual.is_empty() {
                    actual.push(structural.clone());
                    *basis_needed = true;
                }
                rules.push(json!({"id":rule_id,"kind":"defeasible","antecedents":actual,"consequent":objections[name],"name":applicability,"rank":rank(name)}));
            }
            for (kind, target) in targets {
                let table = match kind {
                    "argument" => "arguments",
                    "assumption" => "assumptions",
                    _ => "objections",
                };
                let owner = &mapping[table];
                if kind == "argument" && owner[target]["rule_kind"] == "strict" {
                    return Err(format!(
                        "Objection {} targets strict route {}; a strict rule cannot be undercut",
                        repr(name, limits)?,
                        repr(target, limits)?
                    )
                    .into());
                }
                let target_rule = owner
                    .get(target)
                    .and_then(|value| value["rule_id"].as_str())
                    .map(str::to_owned)
                    .unwrap_or_else(|| symbol("j", target));
                let pair = json!({"attacker":objections[name],"target":symbol("x",&target_rule)});
                contraries.push(pair.clone());
                let mut origin = pair;
                origin["objection"] = json!(name);
                origin["target_kind"] = json!(kind);
                origin["target_name"] = json!(target);
                mapping["contraries"].as_array_mut().unwrap().push(origin);
            }
        }

        Ok(())
    }
}
