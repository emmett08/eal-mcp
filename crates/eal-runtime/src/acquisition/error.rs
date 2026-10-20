use serde::Serialize;
use std::{fmt, io};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum AcquisitionFailureCategory {
    ValueError,
    TypeError,
    OSError,
    FileNotFoundError,
    PermissionError,
    UnicodeDecodeError,
    JsonDecodeError,
    RuntimeError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcquisitionError {
    pub category: AcquisitionFailureCategory,
    pub message: String,
}

impl AcquisitionError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            category: AcquisitionFailureCategory::ValueError,
            message: message.into(),
        }
    }
    pub fn type_name(&self) -> &'static str {
        match self.category {
            AcquisitionFailureCategory::ValueError => "ValueError",
            AcquisitionFailureCategory::TypeError => "TypeError",
            AcquisitionFailureCategory::OSError => "OSError",
            AcquisitionFailureCategory::FileNotFoundError => "FileNotFoundError",
            AcquisitionFailureCategory::PermissionError => "PermissionError",
            AcquisitionFailureCategory::UnicodeDecodeError => "UnicodeDecodeError",
            AcquisitionFailureCategory::RuntimeError => "RuntimeError",
            AcquisitionFailureCategory::JsonDecodeError => "JSONDecodeError",
        }
    }
    pub fn as_record(&self) -> serde_json::Value {
        serde_json::json!({"type":self.type_name(),"message":self.message})
    }
}
impl fmt::Display for AcquisitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for AcquisitionError {}
impl From<io::Error> for AcquisitionError {
    fn from(error: io::Error) -> Self {
        let category = match error.kind() {
            io::ErrorKind::NotFound => AcquisitionFailureCategory::FileNotFoundError,
            io::ErrorKind::PermissionDenied => AcquisitionFailureCategory::PermissionError,
            _ => AcquisitionFailureCategory::OSError,
        };
        Self {
            category,
            message: error.to_string(),
        }
    }
}
impl From<String> for AcquisitionError {
    fn from(v: String) -> Self {
        Self::invalid(v)
    }
}
impl From<&str> for AcquisitionError {
    fn from(v: &str) -> Self {
        Self::invalid(v)
    }
}
