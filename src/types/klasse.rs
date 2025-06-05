use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct Klasse(String);

impl Klasse {
    pub fn klasse(&self) -> String {
        self.0.clone()
    }

    pub fn new(klasse: String) -> Self {
        Self(klasse)
    }

    pub fn stufe(&self) -> Option<u32> {
        if self.0 == "KS1" {
            Some(12)
        } else if self.0 == "KS2" {
            Some(13)
        } else {
            // REGEX
            let regex = Regex::new(r"[0-9]+").unwrap();

            let res = regex.find(self.0.as_str());

            res.map(|matching| str::parse(matching.as_str()).unwrap())
        }
    }
}
