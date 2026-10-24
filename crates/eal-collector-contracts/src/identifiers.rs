//! Distinct checked names for collectors, executable tools and environment variables.
use serde::{Deserialize, Serialize};
macro_rules! identifier_type {
    ($name:ident, $predicate:path, $message:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);
        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, String> {
                let value = value.into();
                if $predicate(&value) {
                    Ok(Self(value))
                } else {
                    Err($message.into())
                }
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = String;
            fn try_from(value: String) -> Result<Self, String> {
                Self::parse(value)
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}
identifier_type!(
    CollectorId,
    crate::identifier,
    "EAL_COLLECTOR_ID: invalid collector identifier"
);
identifier_type!(
    ToolId,
    crate::identifier,
    "EAL_TOOL_ID: invalid tool identifier"
);
identifier_type!(
    EnvironmentName,
    crate::environment_name,
    "EAL_ENVIRONMENT_NAME: invalid environment variable name"
);
