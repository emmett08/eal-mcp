//! One declaration owns request records, operation identities and input schemas.
use eal_runtime::packets::PacketLimits;
use rmcp::model::Tool;
use serde_json::{Map, Value, json};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exposure {
    Operator,
    Registered,
}

impl Exposure {
    pub fn name(self) -> &'static str {
        match self {
            Self::Operator => "operator",
            Self::Registered => "registered",
        }
    }
    pub fn resolve(value: Option<&str>, selected: &[String]) -> Result<Self, String> {
        use std::collections::BTreeSet;
        if selected.len() > 128
            || selected.iter().any(|id| id.is_empty() || id.trim() != id)
            || selected.iter().collect::<BTreeSet<_>>().len() != selected.len()
        {
            return Err(
                "known_entries must contain up to 128 unique non-empty registered IDs".into(),
            );
        }
        let exposure = match value {
            Some("operator") => Self::Operator,
            Some("registered") => Self::Registered,
            Some(_) => return Err("exposure must be operator or registered".into()),
            None if selected.is_empty() => Self::Operator,
            None => Self::Registered,
        };
        match exposure {
            Self::Registered if selected.is_empty() => {
                return Err("registered exposure requires at least one known entry".into());
            }
            Self::Operator if !selected.is_empty() => {
                return Err("operator exposure conflicts with known entries".into());
            }
            _ => (),
        }
        Ok(exposure)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OperationFields {
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
}

#[derive(Clone, Copy)]
enum FieldKind {
    Text,
    Identifier,
    Goal,
    NullableText,
    NullableIdentifier,
    Semantics,
    QueryMode,
    Object,
    Strings,
    NullableIdentifiers,
    Attacks,
    Limit,
    Offset,
    Packet,
}

pub const DEFAULT_LIMIT: usize = 50;
pub const DEFAULT_OFFSET: usize = 0;
pub const DEFAULT_SEMANTICS: &str = "grounded";
pub const DEFAULT_QUERY_MODE: &str = "sceptical";

#[derive(Clone, Copy)]
struct IntegerBounds {
    minimum: usize,
    maximum: usize,
}
const LIMIT: IntegerBounds = IntegerBounds {
    minimum: 1,
    maximum: 100,
};
const OFFSET: IntegerBounds = IntegerBounds {
    minimum: 0,
    maximum: 10_000,
};
const PACKET_BYTES: IntegerBounds = IntegerBounds {
    minimum: 4096,
    maximum: 65_536,
};
const STATEMENT_BYTES: IntegerBounds = IntegerBounds {
    minimum: 1,
    maximum: 4096,
};
impl IntegerBounds {
    fn schema(self, default: usize) -> Value {
        json!({"type":"integer","minimum":self.minimum,"maximum":self.maximum,"default":default})
    }
    fn check(self, key: &str, value: &Value) -> Result<usize, String> {
        strict_integer(key, value, self.minimum, self.maximum)
    }
    fn contains(self, value: usize) -> bool {
        (self.minimum..=self.maximum).contains(&value)
    }
}

macro_rules! field_type {
    (Text) => { String };
    (Identifier) => { String };
    (Goal) => { String };
    (NullableText) => { String };
    (NullableIdentifier) => { String };
    (Semantics) => { String };
    (QueryMode) => { String };
    (Object) => { Map<String, Value> };
    (Strings) => { Vec<String> };
    (NullableIdentifiers) => { Vec<String> };
    (Attacks) => { Vec<[String; 2]> };
    (Limit) => { usize };
    (Offset) => { usize };
    (Packet) => { PacketBudget };
}

macro_rules! operations {
    ($($variant:ident($record:ident) => ($name:literal, $exposure:ident, $acquires:literal, $description:literal) {
        required { $($required:ident: $required_kind:ident),* $(,)? }
        optional { $($optional:ident: $optional_kind:ident),* $(,)? }
    }),* $(,)?) => {
        $(
            #[derive(Clone, Debug)]
            pub struct $record {
                $(pub $required: field_type!($required_kind),)*
                $(pub $optional: Option<field_type!($optional_kind)>,)*
            }
        )*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Operation { $($variant,)* }
        #[derive(Clone, Debug)]
        pub enum OperationRequest { $($variant($record),)* }
        pub const OPERATIONS: &[Operation] = &[$(Operation::$variant,)*];
        pub const OPERATION_FIELDS: &[(&str, OperationFields)] = &[$(
            ($name, OperationFields { required: &[$(stringify!($required)),*], optional: &[$(stringify!($optional)),*] }),
        )*];
        impl Operation {
            pub fn name(self) -> &'static str { match self { $(Self::$variant => $name,)* } }
            pub fn tool_name(self) -> String { format!("eal_{}", self.name()) }
            pub fn parse(name: &str) -> Option<Self> { match name.strip_prefix("eal_").unwrap_or(name) { $($name => Some(Self::$variant),)* _ => None } }
            pub fn exposure(self) -> Exposure { match self { $(Self::$variant => Exposure::$exposure,)* } }
            pub fn acquires(self) -> bool { match self { $(Self::$variant => $acquires,)* } }
            pub fn fields(self) -> OperationFields { match self { $(Self::$variant => OperationFields { required: &[$(stringify!($required)),*], optional: &[$(stringify!($optional)),*] },)* } }
            pub fn schema(self) -> Value {
                let fields: &[(&str, FieldKind, bool)] = match self {
                    $(Self::$variant => &[$((stringify!($required), FieldKind::$required_kind, true),)* $((stringify!($optional), FieldKind::$optional_kind, false),)*],)*
                };
                object_schema(fields)
            }
            pub fn tool(self) -> Tool {
                let description = match self { $(Self::$variant => $description,)* };
                let title = format!("Eal {}", self.name().split('_').map(|word| { let mut chars = word.chars(); chars.next().map(|first| first.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default() }).collect::<Vec<_>>().join(" "));
                let mut tool = Tool::new(self.tool_name(), description, Arc::new(self.schema().as_object().unwrap().clone()))
                    .with_raw_output_schema(Arc::new(json!({"additionalProperties":true,"type":"object"}).as_object().unwrap().clone()));
                tool.title = Some(title);
                tool.meta = Some(rmcp::model::MetaObject::from(json!({"fastmcp":{"tags":[]}}).as_object().unwrap().clone()));
                tool
            }
            fn validate(self, arguments: &Value) -> Result<(), String> {
                let fields: &[(&str, FieldKind, bool)] = match self {
                    $(Self::$variant => &[$((stringify!($required), FieldKind::$required_kind, true),)* $((stringify!($optional), FieldKind::$optional_kind, false),)*],)*
                };
                validate_fields(arguments, fields)
            }
        }
        impl OperationRequest {
            pub fn decode(name: &str, arguments: &Value) -> Result<Self, String> {
                crate::guard::bounded_json(arguments)?;
                let operation = Operation::parse(name).ok_or("Unknown EAL operation [EAL_OPERATION_UNKNOWN]")?;
                operation.validate(arguments)?;
                match operation {
                    $(Operation::$variant => Ok(Self::$variant($record {
                        $($required: read_field(&arguments[stringify!($required)])?,)*
                        $($optional: arguments.get(stringify!($optional)).filter(|value| !value.is_null()).map(read_field).transpose()?,)*
                    })),)*
                }
            }
            pub fn operation(&self) -> Operation { match self { $(Self::$variant(_) => Operation::$variant,)* } }
        }
    };
}

operations! {
    Describe(DescribeRequest) => ("describe", Operator, false, "Discover supported language versions, syntax, method contracts and interpretation limits.") { required {} optional {} },
    Format(FormatRequest) => ("format", Operator, false, "Return canonical checked EAL source; changed source bytes require new collection bindings.") { required {source: Text} optional {} },
    Validate(ValidateRequest) => ("validate", Operator, false, "Parse EAL and check names, types and dependency structure without running tools.") { required {source: Text} optional {} },
    Plan(PlanRequest) => ("plan", Operator, false, "Plan all evidence for a claim, including alternative supports and objections.") { required {source: Text, claim: Identifier} optional {} },
    Collect(CollectRequest) => ("collect", Operator, true, "Run the configured collectors for selected evidence; return collection_id.") { required {source: Text, context: Object} optional {evidence_ids: NullableIdentifiers} },
    CollectClaim(CollectClaimRequest) => ("collect_claim", Operator, true, "Collect the complete declared evidence closure for a claim.") { required {source: Text, context: Object, claim: Identifier} optional {} },
    Prepare(PrepareRequest) => ("prepare", Operator, true, "Inspect configured interfaces and return source-bound tool pins without acquiring observations or assessing support.") { required {source: Text, context: Object} optional {claim: NullableIdentifier} },
    Execute(ExecuteRequest) => ("execute", Operator, true, "Execute one EAL program: acquire its evidence dependencies and retain its assessment using operator-selected tools.") { required {source: Text, context: Object} optional {claim: NullableIdentifier, now: NullableText} },
    Reason(ReasonRequest) => ("reason", Operator, false, "Evaluate declared support and objections at an explicit or current UTC time.") { required {source: Text, context: Object} optional {collection_id: NullableIdentifier, now: NullableText} },
    CompileAspic(CompileAspicRequest) => ("compile_aspic", Operator, false, "Compile checked EAL routes to an opt-in bounded ASPIC+ snapshot with a source map.") { required {source: Text, context: Object, collection_id: Identifier, goal: Goal} optional {now: NullableText, semantics: Semantics, query_mode: QueryMode, preference: NullableText} },
    Explain(ExplainRequest) => ("explain", Operator, false, "Retrieve a persisted reasoning result and its dependency explanations.") { required {assessment_id: Identifier} optional {claim: NullableIdentifier} },
    Packet(PacketRequest) => ("packet", Operator, false, "Return compact claim results with a reference to the full explanation.") { required {assessment_id: Identifier} optional {claim: NullableIdentifier, packet_limits: Packet} },
    Sources(SourcesRequest) => ("sources", Registered, false, "List developer-registered EAL sources and their selectable claims.") { required {} optional {limit: Limit, offset: Offset} },
    FindClaims(FindClaimsRequest) => ("find_claims", Registered, false, "Find registered claim metadata; use an exact entry ID and claim to assess.") { required {} optional {query: NullableText, claim: NullableText, limit: Limit} },
    AssessKnown(AssessKnownRequest) => ("assess_known", Registered, true, "Reuse compatible observations or collect missing evidence for a registered claim.") { required {entry_id: Identifier, claim: Identifier} optional {packet_limits: Packet} },
    GetKnownPacket(GetKnownPacketRequest) => ("get_known_packet", Registered, false, "Retrieve a bounded packet for a stored assessment of an allowed registered claim without collecting evidence or reassessing. Original assessment and observation times remain historical.") { required {entry_id: Identifier, assessment_id: Identifier, claim: Identifier} optional {packet_limits: Packet} },
    Grounded(GroundedRequest) => ("grounded", Operator, false, "Compute Dung grounded semantics for an explicitly supplied finite attack graph.") { required {arguments: Strings, attacks: Attacks} optional {} },
}

// Nested budgets follow the same single-owner rule as top-level operations.
macro_rules! packet_budget {
    ($($field:ident: $bounds:ident),* $(,)?) => {
        #[derive(Clone, Debug, Default)]
        pub struct PacketBudget { $(pub $field: Option<usize>,)* }
        impl PacketBudget {
            pub fn resolve(&self) -> Result<PacketLimits, String> {
                let defaults = PacketLimits::default();
                let result = PacketLimits { $($field: self.$field.unwrap_or(defaults.$field),)* ..defaults };
                if $(!$bounds.contains(result.$field))||* {
                    return Err("Packet budget exceeds operation bounds [EAL_OPERATION_BUDGET]".into());
                }
                result.validate()?;
                Ok(result)
            }
        }
        fn packet_schema() -> Value {
            let defaults = PacketLimits::default();
            let properties: Map<String, Value> = [$(
                (stringify!($field).into(), $bounds.schema(defaults.$field)),
            )*].into_iter().collect();
            json!({"type":"object","additionalProperties":false,"description":"Limits for compact UTF-8 packet JSON only; response wrappers are additional. Dependency and traversal ceilings remain unchanged.","properties":properties})
        }
        fn validate_packet(value: &Value) -> Result<(), String> {
            let object = value.as_object().ok_or("packet_limits must be an object")?;
            for (key, value) in object {
                match key.as_str() {
                    $(stringify!($field) => { $bounds.check(concat!("packet_limits.", stringify!($field)), value)?; },)*
                    _ => return Err("Unknown packet budget field [EAL_OPERATION_UNKNOWN_FIELD]".into()),
                }
            }
            Ok(())
        }
        impl FieldValue for PacketBudget {
            fn read(value: &Value) -> Option<Self> {
                let object = value.as_object()?;
                Some(Self { $($field: object.get(stringify!($field)).map(read_field).transpose().ok()?,)* })
            }
        }
    };
}
packet_budget! { max_bytes: PACKET_BYTES, max_statement_bytes: STATEMENT_BYTES }

fn object_schema(fields: &[(&str, FieldKind, bool)]) -> Value {
    let properties: Map<String, Value> = fields
        .iter()
        .map(|(name, kind, _)| ((*name).into(), kind.schema()))
        .collect();
    let mut schema = json!({"type":"object","additionalProperties":false,"properties":properties});
    let required: Vec<_> = fields
        .iter()
        .filter_map(|(name, _, required)| required.then_some(*name))
        .collect();
    if !required.is_empty() {
        schema["required"] = json!(required);
    }
    schema
}
impl FieldKind {
    fn schema(self) -> Value {
        match self {
            Self::Text => json!({"type":"string"}),
            Self::Identifier => json!({"type":"string","minLength":1}),
            Self::Goal => json!({"type":"string","pattern":"\\S"}),
            Self::NullableText => {
                json!({"anyOf":[{"type":"string"},{"type":"null"}],"default":null})
            }
            Self::NullableIdentifier => {
                json!({"anyOf":[{"type":"string","minLength":1},{"type":"null"}],"default":null})
            }
            Self::Semantics => json!({"type":"string","default":DEFAULT_SEMANTICS}),
            Self::QueryMode => json!({"type":"string","default":DEFAULT_QUERY_MODE}),
            Self::Object => json!({"type":"object","additionalProperties":true}),
            Self::Strings => json!({"type":"array","items":{"type":"string"}}),
            Self::NullableIdentifiers => {
                json!({"anyOf":[{"type":"array","items":{"type":"string","minLength":1}},{"type":"null"}],"default":null})
            }
            Self::Attacks => {
                json!({"type":"array","items":{"type":"array","minItems":2,"maxItems":2,"items":{"type":"string"}}})
            }
            Self::Limit => LIMIT.schema(DEFAULT_LIMIT),
            Self::Offset => OFFSET.schema(DEFAULT_OFFSET),
            Self::Packet => packet_schema(),
        }
    }
    fn validate(self, key: &str, value: &Value) -> Result<(), String> {
        let valid = match self {
            Self::Text | Self::Semantics | Self::QueryMode => value.is_string(),
            Self::Identifier => value.as_str().is_some_and(|text| !text.is_empty()),
            Self::Goal => value.as_str().is_some_and(|text| !text.trim().is_empty()),
            Self::NullableText => value.is_null() || value.is_string(),
            Self::NullableIdentifier => {
                value.is_null() || value.as_str().is_some_and(|text| !text.is_empty())
            }
            Self::Object => value.is_object(),
            Self::Strings => value
                .as_array()
                .is_some_and(|items| items.iter().all(Value::is_string)),
            Self::NullableIdentifiers => {
                value.is_null()
                    || value.as_array().is_some_and(|items| {
                        items
                            .iter()
                            .all(|value| value.as_str().is_some_and(|text| !text.is_empty()))
                    })
            }
            Self::Attacks => value.as_array().is_some_and(|items| {
                items.iter().all(|edge| {
                    edge.as_array()
                        .is_some_and(|pair| pair.len() == 2 && pair.iter().all(Value::is_string))
                })
            }),
            Self::Limit => return LIMIT.check(key, value).map(|_| ()),
            Self::Offset => return OFFSET.check(key, value).map(|_| ()),
            Self::Packet => return validate_packet(value),
        };
        valid
            .then_some(())
            .ok_or_else(|| format!("Invalid operation field {key} [EAL_OPERATION_FIELD]"))
    }
}
fn validate_fields(arguments: &Value, fields: &[(&str, FieldKind, bool)]) -> Result<(), String> {
    let object = arguments
        .as_object()
        .ok_or("Operation arguments must be an object [EAL_OPERATION_SHAPE]")?;
    if object
        .keys()
        .any(|name| !fields.iter().any(|(field, _, _)| name == field))
    {
        return Err("Unknown operation field [EAL_OPERATION_UNKNOWN_FIELD]".into());
    }
    for (name, kind, required) in fields {
        match object.get(*name) {
            None if *required => {
                return Err(format!(
                    "Missing operation field {name} [EAL_OPERATION_REQUIRED]"
                ));
            }
            Some(value) => kind.validate(name, value)?,
            None => (),
        }
    }
    Ok(())
}
fn strict_integer(
    key: &str,
    value: &Value,
    minimum: usize,
    maximum: usize,
) -> Result<usize, String> {
    if value
        .as_number()
        .is_some_and(|number| number.as_str().contains(['.', 'e', 'E']))
    {
        return Err(format!("Tool input requires a strict integer for {key}"));
    }
    value
        .as_u64()
        .and_then(|number| usize::try_from(number).ok())
        .filter(|number| (minimum..=maximum).contains(number))
        .ok_or_else(|| {
            format!(
                "{key} must be an integer from {minimum} through {maximum} [EAL_OPERATION_INTEGER]"
            )
        })
}
// Read admitted Values directly. Deserialising arbitrary context Values again
// would reinterpret serde_json's private number keys and lose literal data.
trait FieldValue: Sized {
    fn read(value: &Value) -> Option<Self>;
}
fn read_field<T: FieldValue>(value: &Value) -> Result<T, String> {
    T::read(value).ok_or_else(|| "Invalid typed operation request [EAL_OPERATION_SHAPE]".into())
}
impl FieldValue for String {
    fn read(value: &Value) -> Option<Self> {
        value.as_str().map(str::to_owned)
    }
}
impl FieldValue for Map<String, Value> {
    fn read(value: &Value) -> Option<Self> {
        value.as_object().cloned()
    }
}
impl FieldValue for usize {
    fn read(value: &Value) -> Option<Self> {
        value
            .as_u64()
            .and_then(|number| usize::try_from(number).ok())
    }
}
impl FieldValue for Vec<String> {
    fn read(value: &Value) -> Option<Self> {
        value.as_array()?.iter().map(String::read).collect()
    }
}
impl FieldValue for Vec<[String; 2]> {
    fn read(value: &Value) -> Option<Self> {
        value
            .as_array()?
            .iter()
            .map(|value| {
                let pair = value.as_array()?;
                Some([String::read(pair.first()?)?, String::read(pair.get(1)?)?])
            })
            .collect()
    }
}
