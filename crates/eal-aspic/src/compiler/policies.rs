use super::translation::Translation;
use super::*;
impl Translation<'_> {
    pub(super) fn finish(
        mut self,
        options: &CompileOptions<'_>,
    ) -> Result<(Value, Value), AspicError> {
        let program = self.program;
        let claims = &self.claims;
        let evidence = &self.evidence;
        let annotations = &self.annotations;
        let directive = |item: &s::ArgumentationDirective| annotations.directive(item);
        let mapping = &mut self.mapping;
        let premises = &mut self.premises;
        let contraries = &mut self.contraries;
        let structural = &self.structural;
        let basis_needed = &mut self.basis_needed;
        for item in &program.argumentation_directives {
            if item.kind == "contrary" {
                let other = item
                    .other
                    .as_deref()
                    .ok_or("Contrary directive has no target")?;
                let pair = json!({"attacker":claims[item.name.as_str()],"target":claims[other]});
                contraries.push(pair.clone());
                let mut origin = pair;
                origin["target_kind"] = json!("claim");
                origin["attacker_name"] = json!(item.name);
                origin["target_name"] = json!(other);
                origin["annotation"] = directive(item)?;
                mapping["contraries"].as_array_mut().unwrap().push(origin);
            }
        }
        if *basis_needed || premises.is_empty() {
            premises.push(json!({"atom":structural,"kind":"axiom"}));
            mapping["structural_basis"] = json!({"atom":structural,"meaning":"Authored ungrounded relation or empty snapshot; no observation"});
        }
        let mut theory = json!({"premises":self.premises,"rules":self.rules,"contraries":self.contraries,"goal":claims[options.goal]});
        let preferences: Vec<_> = program
            .argumentation_directives
            .iter()
            .filter(|d| d.kind == "prefer")
            .collect();
        let selected_preference =
            options
                .preference
                .filter(|p| !p.is_empty())
                .unwrap_or(if preferences.is_empty() {
                    "minimum_rank"
                } else {
                    "last_link_partial"
                });
        if !preferences.is_empty() && selected_preference != "last_link_partial" {
            return Err("Reviewed priorities require last_link_partial preference".into());
        }
        if options.semantics != "grounded"
            || options.query_mode != "sceptical"
            || selected_preference != "minimum_rank"
        {
            theory["options"] = json!({"semantics":options.semantics,"query_mode":options.query_mode,"preference":selected_preference});
        }
        if !preferences.is_empty() {
            let mut priorities = Vec::new();
            for item in preferences {
                let other = item
                    .other
                    .as_deref()
                    .ok_or("Priority directive has no target")?;
                let (kind, higher, lower, eligible) = if evidence.contains_key(item.name.as_str()) {
                    (
                        "premise",
                        evidence[item.name.as_str()].clone(),
                        evidence[other].clone(),
                        mapping["evidence"][&item.name]["available"] == true
                            && mapping["evidence"][other]["available"] == true,
                    )
                } else {
                    let a = &mapping[if program.arguments.contains_key(&item.name) {
                        "arguments"
                    } else {
                        "assumptions"
                    }][&item.name];
                    let b = &mapping[if program.arguments.contains_key(other) {
                        "arguments"
                    } else {
                        "assumptions"
                    }][other];
                    (
                        "rule",
                        a["rule_id"]
                            .as_str()
                            .ok_or("Missing priority rule")?
                            .to_owned(),
                        b["rule_id"]
                            .as_str()
                            .ok_or("Missing priority rule")?
                            .to_owned(),
                        a["emitted"] == true && b["emitted"] == true,
                    )
                };
                if eligible {
                    priorities.push(json!({"kind":kind,"higher":higher,"lower":lower}));
                }
            }
            theory["priorities"] = json!(priorities);
        }
        for table in [
            "claims",
            "evidence",
            "assumptions",
            "arguments",
            "objections",
        ] {
            for (name, entry) in mapping[table].as_object_mut().unwrap() {
                if let Some(source) = program.source_files.get(name) {
                    entry["source_file"] = json!(source);
                }
                if table == "arguments" {
                    let argument = &program.arguments[name];
                    if let Some(transfer) = &program.reasoning[&argument.reasoning].transfer {
                        entry["scope_transfer"] =
                            serde_json::to_value(transfer).map_err(|error| error.to_string())?;
                    }
                }
            }
        }

        Ok((theory, self.mapping))
    }
}
