//! Private composition records retain lexical scope and declaration identity.
use super::*;
#[derive(Clone)]
pub(super) struct Scope {
    pub(super) prefix: String,
    pub(super) bindings: Bindings,
    pub(super) span: SourceSpan,
    pub(super) pattern: Option<String>,
    pub(super) application: Option<String>,
    pub(super) depth: usize,
    pub(super) stack: Vec<(String, Option<usize>)>,
    pub(super) guards: BTreeSet<String>,
    pub(super) source_file: Option<String>,
    pub(super) defaults: BTreeMap<String, Field>,
}
impl Scope {
    pub(super) fn root(origin: &str) -> Self {
        Self {
            prefix: String::new(),
            bindings: Bindings::new(),
            span: SourceSpan::default(),
            pattern: None,
            application: None,
            depth: 0,
            stack: Vec::new(),
            guards: BTreeSet::new(),
            source_file: (!origin.is_empty()).then(|| origin.into()),
            defaults: BTreeMap::new(),
        }
    }
}

#[derive(Clone)]
pub(super) enum Entity {
    Environment(s::Environment),
    Tool(s::Tool),
    Evidence(s::Evidence),
    Assumption(s::Assumption),
    Reasoning(s::Reasoning),
    Claim(s::Claim),
    Argument(s::Argument),
    Objection(s::Objection),
    Pattern(s::Pattern),
    Application(s::Application),
    Directive(s::ArgumentationDirective),
}
impl Entity {
    pub(super) fn kind(&self) -> &'static str {
        match self {
            Self::Environment(_) => "environment",
            Self::Tool(_) => "tool",
            Self::Evidence(_) => "evidence",
            Self::Assumption(_) => "assumption",
            Self::Reasoning(_) => "reasoning",
            Self::Claim(_) => "claim",
            Self::Argument(_) => "argument",
            Self::Objection(_) => "objection",
            Self::Pattern(_) => "pattern",
            Self::Application(_) => "application",
            Self::Directive(_) => "directive",
        }
    }
    pub(super) fn name(&self) -> &str {
        match self {
            Self::Environment(v) => &v.name,
            Self::Tool(v) => &v.name,
            Self::Evidence(v) => &v.name,
            Self::Assumption(v) => &v.name,
            Self::Reasoning(v) => &v.name,
            Self::Claim(v) => &v.name,
            Self::Argument(v) => &v.name,
            Self::Objection(v) => &v.name,
            Self::Pattern(v) => &v.name,
            Self::Application(v) => &v.name,
            Self::Directive(v) => &v.name,
        }
    }
    pub(super) fn rename(&mut self, name: String) {
        match self {
            Self::Environment(v) => v.name = name,
            Self::Tool(v) => v.name = name,
            Self::Evidence(v) => v.name = name,
            Self::Assumption(v) => v.name = name,
            Self::Reasoning(v) => v.name = name,
            Self::Claim(v) => v.name = name,
            Self::Argument(v) => v.name = name,
            Self::Objection(v) => v.name = name,
            Self::Pattern(v) => v.name = name,
            Self::Application(v) => v.name = name,
            Self::Directive(v) => v.name = name,
        }
    }
    pub(super) fn references(&self) -> Vec<(String, String, bool)> {
        let mut refs = Vec::new();
        let mut add = |name: &str, kind: &str, many| refs.push((name.into(), kind.into(), many));
        match self {
            Self::Evidence(v) => {
                add(&v.tool, "tool", false);
                add(&v.environment, "environment", false);
            }
            Self::Claim(v) => add(&v.environment, "environment", false),
            Self::Assumption(v) => {
                add(&v.environment, "environment", false);
                add(&v.validation, "evidence", false);
            }
            Self::Reasoning(v) => {
                for n in &v.backing {
                    add(n, "evidence", true);
                }
                if let Some(t) = &v.transfer {
                    add(&t.source, "environment", false);
                    add(&t.target, "environment", false);
                    add(&t.assumption, "assumption", false);
                }
            }
            Self::Argument(v) => {
                add(&v.conclusion, "claim", false);
                add(&v.reasoning, "reasoning", false);
                for n in &v.evidence {
                    add(n, "evidence", true);
                }
                for n in &v.assumptions {
                    add(n, "assumption", true);
                }
                for n in &v.premises {
                    add(n, "claim", true);
                }
                if let Some(n) = &v.binding {
                    add(n, "evidence", false);
                }
            }
            Self::Pattern(v) => {
                add(&v.conclusion, "claim", false);
                add(&v.reasoning, "reasoning", false);
                for n in &v.evidence {
                    add(n, "evidence", true);
                }
                for n in &v.assumptions {
                    add(n, "assumption", true);
                }
                for n in &v.premises {
                    add(n, "claim", true);
                }
                if let Some(n) = &v.binding {
                    add(n, "evidence", false);
                }
            }
            Self::Objection(v) => {
                add(&v.target, &v.target_kind, false);
                for n in &v.evidence {
                    add(n, "evidence", true);
                }
                for n in &v.premises {
                    add(n, "claim", true);
                }
            }
            _ => {}
        }
        refs
    }
}
#[derive(Clone)]
pub(super) struct Record {
    pub(super) value: Entity,
    pub(super) scope: Scope,
}
#[derive(Clone)]
pub(super) struct Call {
    pub(super) caller: String,
    pub(super) called: String,
    pub(super) supplied: BTreeMap<String, s::BindingValue>,
    pub(super) guards: BTreeSet<String>,
}
