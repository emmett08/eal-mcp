//! Typed authored syntax, before name resolution, imports or pattern expansion.
use eal_parser::SourceSpan;
use serde::{Deserialize, Serialize};

/// An EAL decimal number kept exactly as authored, including arbitrary-size integers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumberLiteral(pub String);

/// JSON values retain decimal lexemes until an evaluator selects its numerical domain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(NumberLiteral),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum Reference {
    Name(String),
    Head(String),
    Tail(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnaryOperator {
    Not,
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Expression {
    Literal {
        value: JsonValue,
    },
    Field {
        name: String,
    },
    Call {
        name: String,
        arguments: Vec<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextProperty {
    Version,
    Statement,
    ValidFrom,
    ValidUntil,
    Method,
    Rationale,
    Subject,
    Quantity,
    Unit,
    Scope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NameProperty {
    Tool,
    Environment,
    Validate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonProperty {
    Input,
    Query,
}

/// The grammar constrains which typed fields each declaration accepts.
/// Context inheritance is checked without replacing the authored fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Field {
    Text {
        property: TextProperty,
        value: String,
    },
    Name {
        property: NameProperty,
        value: String,
    },
    Kind {
        value: String,
    },
    MaxAge {
        value: NumberLiteral,
    },
    Json {
        property: JsonProperty,
        value: JsonValue,
    },
    Derive {
        expression: Expression,
    },
    InputBinding {
        target: String,
        evidence: String,
        source: String,
    },
    Require {
        expression: Expression,
    },
    Result {
        expression: Expression,
    },
    Backing {
        references: Vec<Reference>,
    },
    Transfer {
        source: String,
        target: String,
        assumption: String,
        review: String,
    },
    Proposition {
        fields: Vec<Field>,
    },
}

impl Field {
    pub fn name(&self) -> &'static str {
        match self {
            Self::InputBinding { .. } => "bind",
            Self::Derive { .. } => "derive",
            Self::Text { property, .. } => match property {
                TextProperty::Version => "version",
                TextProperty::Statement => "statement",
                TextProperty::ValidFrom => "valid_from",
                TextProperty::ValidUntil => "valid_until",
                TextProperty::Method => "method",
                TextProperty::Rationale => "rationale",
                TextProperty::Subject => "subject",
                TextProperty::Quantity => "quantity",
                TextProperty::Unit => "unit",
                TextProperty::Scope => "scope",
            },
            Self::Name { property, .. } => match property {
                NameProperty::Tool => "tool",
                NameProperty::Environment => "environment",
                NameProperty::Validate => "validate",
            },
            Self::Kind { .. } => "kind",
            Self::MaxAge { .. } => "max_age",
            Self::Json { property, .. } => match property {
                JsonProperty::Input => "input",
                JsonProperty::Query => "query",
            },
            Self::Require { .. } => "require",
            Self::Result { .. } => "result",
            Self::Backing { .. } => "backing",
            Self::Transfer { .. } => "transfer",
            Self::Proposition { .. } => "proposition",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportKind {
    Evidence,
    Assumptions,
    Premises,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportGroup {
    pub kind: SupportKind,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgumentFlow {
    pub support: Vec<SupportGroup>,
    pub reasoning: Reference,
    pub conclusion: Reference,
    pub binding: Option<Reference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternParameter {
    pub name: String,
    pub kind: String,
    pub list: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum BindingValue {
    Single(Reference),
    List(Vec<Reference>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternBinding {
    pub name: String,
    pub value: BindingValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Claim,
    Reasoning,
    Assumption,
    Argument,
    Objection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Directive {
    Strict {
        name: String,
        review: String,
    },
    Rank {
        name: String,
        value: NumberLiteral,
        review: String,
    },
    Contrary {
        name: String,
        other: String,
        review: String,
    },
    Prefer {
        name: String,
        other: String,
        review: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PatternBody {
    Flow(ArgumentFlow),
    Block(Vec<Declaration>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeclarationKind {
    Module {
        name: String,
        body: Vec<Declaration>,
    },
    Import {
        path: String,
        alias: String,
    },
    Context {
        defaults: Vec<Field>,
        body: Vec<Declaration>,
    },
    Environment {
        name: String,
        predicates: Vec<Expression>,
    },
    Tool {
        name: String,
        fields: Vec<Field>,
    },
    Evidence {
        name: String,
        fields: Vec<Field>,
    },
    Assumption {
        name: String,
        fields: Vec<Field>,
    },
    Reasoning {
        name: String,
        fields: Vec<Field>,
    },
    Claim {
        name: String,
        fields: Vec<Field>,
    },
    Argument {
        name: String,
        body: Option<Vec<Declaration>>,
        flow: ArgumentFlow,
    },
    Pattern {
        name: String,
        parameters: Vec<PatternParameter>,
        decreases: Option<String>,
        body: PatternBody,
    },
    Application {
        name: String,
        pattern: String,
        bindings: Vec<PatternBinding>,
    },
    Guard {
        parameter: String,
        body: Vec<Declaration>,
        otherwise: Option<Vec<Declaration>>,
    },
    Objection {
        name: String,
        support: Vec<SupportGroup>,
        target_kind: TargetKind,
        target: Reference,
    },
    Argumentation {
        directive: Directive,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    pub span: SourceSpan,
    pub value: DeclarationKind,
}

/// Authored EAL/3 syntax with inherited-field checks, before any source IO or expansion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub language: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    pub declarations: Vec<Declaration>,
}
