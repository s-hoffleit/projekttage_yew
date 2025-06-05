use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default, PartialOrd, Ord,
)]
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

impl From<String> for ProjektId {
    fn from(value: String) -> Self {
        let id: i32 = str::parse::<i32>(&value).expect("Falsche ProjektId");
        let id = if id < 0 { u32::MAX } else { id as u32 };
        Self(id)
    }
}

impl std::ops::Sub<u32> for ProjektId {
    type Output = ProjektId;

    fn sub(self, rhs: u32) -> Self::Output {
        ProjektId(self.0 - rhs)
    }
}
