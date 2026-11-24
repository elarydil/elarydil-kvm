pub mod cli;
mod cli_help;
pub mod commands;
pub mod engine;
pub mod ipc;
pub mod model;
pub mod monitor_control;
pub mod platform;
pub mod service;
pub mod store;
use serde::{Deserialize, Serialize};
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Error {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub field: Option<String>,
}
impl Error {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.into(),
            message: t(&format!("error.{code}")),
            detail: None,
            field: None,
        }
    }
    pub fn detail(mut self, v: impl ToString) -> Self {
        self.detail = Some(v.to_string());
        self
    }
    pub fn field(mut self, field: impl ToString) -> Self {
        self.field = Some(field.to_string());
        self
    }
    pub fn context(mut self, operation: &str) -> Self {
        self.detail = Some(match self.detail {
            Some(detail) => format!("{operation}: {detail}"),
            None => operation.into(),
        });
        self
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(d) = &self.detail {
            write!(f, " ({d})")?
        }
        Ok(())
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(if e.kind() == std::io::ErrorKind::PermissionDenied {
            "permission_denied"
        } else {
            "io"
        })
        .detail(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::new("invalid_input").detail(e)
    }
}
pub fn catalog() -> &'static serde_json::Value {
    static C: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        serde_json::from_str(include_str!("../../../locales/en-US.json")).expect("catalog")
    })
}
pub fn t(key: &str) -> String {
    catalog()[key].as_str().unwrap_or(key).to_string()
}

#[cfg(windows)]
mod windows_ddc;

pub mod wizard_cli;
