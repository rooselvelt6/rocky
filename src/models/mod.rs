pub mod apache;
pub mod config;
pub mod glasgow;
pub mod history;
pub mod news2;
pub mod patient;
pub mod saps;
pub mod sofa;
pub mod user;

#[cfg(not(feature = "ssr"))]
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Id {
    #[serde(rename = "String")]
    pub string: String,
}

#[cfg(not(feature = "ssr"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Thing {
    pub tb: String,
    pub id: Id,
}

#[cfg(not(feature = "ssr"))]
impl ToString for Thing {
    fn to_string(&self) -> String {
        format!("{}:{}", self.tb, self.id.string)
    }
}
