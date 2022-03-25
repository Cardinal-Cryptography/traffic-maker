use serde::{Deserialize, Serialize};

/// A wrapper type for scenario identification.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Ident(pub String);

impl From<String> for Ident {
    fn from(inner: String) -> Self {
        Ident(inner)
    }
}

impl From<&str> for Ident {
    fn from(inner: &str) -> Self {
        Ident(inner.to_string())
    }
}
