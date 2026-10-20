//! Explicit host budgets captured by a compiled programme and pure operations.
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

macro_rules! execution_limits {
    ($($field:ident: $default:expr),+ $(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
        pub struct ExecutionLimits { $(pub $field: usize,)+ }

        impl Default for ExecutionLimits {
            fn default() -> Self { Self { $($field: $default,)+ } }
        }

        impl ExecutionLimits {
            pub fn validate(&self) -> Result<(), String> {
                $(if self.$field == 0 {
                    return Err(format!("Budget {} must be a positive integer", stringify!($field)));
                })+
                Ok(())
            }

            /// Describe only operational budgets, independently of language versions.
            pub fn describe(&self) -> serde_json::Value {
                serde_json::to_value(self).expect("bounded integer budgets serialise")
            }
        }

        impl<'de> Deserialize<'de> for ExecutionLimits {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let supplied = BTreeMap::<String, usize>::deserialize(deserializer)?;
                let mut limits = Self::default();
                for (name, value) in supplied {
                    match name.as_str() {
                        $(stringify!($field) => limits.$field = value,)+
                        _ => return Err(serde::de::Error::custom(format!("Unknown execution budget {name:?}"))),
                    }
                }
                limits.validate().map_err(serde::de::Error::custom)?;
                Ok(limits)
            }
        }
    };
}

execution_limits! {
    source_bytes: 1024 * 1024,
    source_tokens: 100_000,
    structural_values: 100_000,
    declarations: 4096,
    premise_depth: 128,
    expansion_depth: 128,
    applications: 1000,
    expanded_references: 100_000,
    expression_nodes: 10_000,
    expression_depth: 128,
    json_depth: 64,
    json_nodes: 100_000,
    record_bytes: 4 * 1024 * 1024,
    collection_evidence: 128,
    composed_edges: 131_072,
    abstract_attacks: 65_536,
    formal_premises: 64,
    formal_rules: 64,
    formal_antecedents: 8,
    formal_atoms: 128,
    formal_contraries: 128,
    formal_arguments: 128,
    formal_defeats: 4096,
    formal_construction: 100_000,
    extension_search: 100_000,
}
