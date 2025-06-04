use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct Klasse(String);

impl Klasse {
    pub fn klasse(&self) -> String {
        self.0.clone()
    }
}
