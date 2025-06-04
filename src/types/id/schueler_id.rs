use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct SchuelerId(Uuid);

impl fmt::Display for SchuelerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SchuelerId {
    pub fn id(&self) -> Uuid {
        self.0
    }
}
