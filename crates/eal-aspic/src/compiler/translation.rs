use super::*;
pub(super) struct Annotations<'a> {
    pub entries: BTreeMap<(&'a str, &'a str), &'a s::ArgumentationDirective>,
    pub target_kinds: BTreeMap<&'a str, &'static str>,
}
impl Annotations<'_> {
    pub fn rank(&self, name: &str) -> i64 {
        self.entries
            .get(&("rank", name))
            .and_then(|d| d.rank)
            .unwrap_or(RANK)
    }
    pub fn directive(&self, item: &s::ArgumentationDirective) -> Result<Value, String> {
        let mut value = serde_json::to_value(item).map_err(|error| error.to_string())?;
        value["target_kind"] = json!(self.target_kinds[&item.name.as_str()]);
        Ok(value)
    }
    pub fn annotated(&self, kind: &str, name: &str) -> Result<Value, String> {
        self.entries
            .get(&(kind, name))
            .map_or(Ok(Value::Null), |item| self.directive(item))
    }
}
pub(super) struct Translation<'a> {
    pub program: &'a s::Program,
    pub records: &'a Value,
    pub assessment: &'a Value,
    pub annotations: Annotations<'a>,
    pub claims: BTreeMap<&'a str, String>,
    pub evidence: BTreeMap<&'a str, String>,
    pub assumptions: BTreeMap<&'a str, String>,
    pub objections: BTreeMap<&'a str, String>,
    pub premises: Vec<Value>,
    pub rules: Vec<Value>,
    pub contraries: Vec<Value>,
    pub mapping: Value,
    pub structural: String,
    pub basis_needed: bool,
}
impl<'a> Translation<'a> {
    pub(super) fn new(
        program: &'a s::Program,
        records: &'a Value,
        assessment: &'a Value,
        goal: &str,
    ) -> Result<Self, AspicError> {
        let limits = &program.limits;
        let annotations: BTreeMap<_, _> = program
            .argumentation_directives
            .iter()
            .filter(|d| matches!(d.kind.as_str(), "rank" | "strict"))
            .map(|d| ((d.kind.as_str(), d.name.as_str()), d))
            .collect();
        let mut target_kinds = BTreeMap::new();
        for (kind, names) in [
            ("evidence", program.evidence.keys().collect::<Vec<_>>()),
            ("assumption", program.assumptions.keys().collect()),
            ("argument", program.arguments.keys().collect()),
            ("objection", program.objections.keys().collect()),
            ("claim", program.claims.keys().collect()),
        ] {
            for name in names {
                target_kinds.insert(name.as_str(), kind);
            }
        }
        let annotations = Annotations {
            entries: annotations,
            target_kinds,
        };
        let claims: BTreeMap<_, _> = program
            .claims
            .keys()
            .map(|name| (name.as_str(), symbol("c", name)))
            .collect();
        let evidence: BTreeMap<_, _> = program
            .evidence
            .keys()
            .map(|name| (name.as_str(), symbol("e", name)))
            .collect();
        let assumptions: BTreeMap<_, _> = program
            .assumptions
            .keys()
            .map(|name| (name.as_str(), symbol("s", name)))
            .collect();
        let objections: BTreeMap<_, _> = program
            .objections
            .keys()
            .map(|name| (name.as_str(), symbol("o", name)))
            .collect();
        let directives = program
            .argumentation_directives
            .iter()
            .map(|item| annotations.directive(item))
            .collect::<Result<Vec<_>, _>>()?;
        let mapping = json!({"profile":PROFILE,"source_digest":program.source_digest,"assessed_at":assessment["assessed_at"],"context_fingerprint":assessment["context_fingerprint"],"method_registry_fingerprint":assessment["method_registry_fingerprint"],"backend":backend(limits)?,"execution_limits":limits.describe(),"goal":{"claim":goal,"atom":claims[goal]},"claims":{},"evidence":{},"assumptions":{},"arguments":{},"objections":{},"contraries":[],"formal_directives":directives});
        Ok(Self {
            program,
            records,
            assessment,
            annotations,
            claims,
            evidence,
            assumptions,
            objections,
            premises: Vec::new(),
            rules: Vec::new(),
            contraries: Vec::new(),
            mapping,
            structural: symbol("b", "compiler_basis"),
            basis_needed: false,
        })
    }
}
pub(super) fn translate(
    program: &s::Program,
    records: &Value,
    assessment: &Value,
    options: &CompileOptions<'_>,
) -> Result<(Value, Value), AspicError> {
    let mut state = Translation::new(program, records, assessment, options.goal)?;
    state.observations()?;
    state.arguments()?;
    state.objections()?;
    state.finish(options)
}
