use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ProjektId(u32);

impl fmt::Display for ProjektId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

impl ProjektId {
    pub fn id(&self) -> u32 {
        self.0
    }
}
