use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{Klasse, ProjektId, SaveFileSchueler, SchuelerId};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Schueler {
    antworten: String,
    anmeldename: Uuid,
    vollstndigername: String,
    gruppe: String,
    q01_erstwunsch: Option<String>,
    q02_zweitwunsch: Option<String>,
    q03_drittwunsch: Option<String>,
    q04_viertwunsch: Option<String>,
    q05_fnftwunsch: Option<String>,
    q06_wunschpartner: Option<String>,
}

impl Schueler {
    fn get_wuensche(&self) -> Option<[ProjektId; 5]> {
        if let (Some(wunsch1), Some(wunsch2), Some(wunsch3), Some(wunsch4), Some(wunsch5)) = (
            self.q01_erstwunsch.clone(),
            self.q02_zweitwunsch.clone(),
            self.q03_drittwunsch.clone(),
            self.q04_viertwunsch.clone(),
            self.q05_fnftwunsch.clone(),
        ) {
            Some([wunsch1, wunsch2, wunsch3, wunsch4, wunsch5].map(|wunsch| {
                std::convert::Into::<ProjektId>::into(
                    (*wunsch.split(" : ").collect::<Vec<&str>>().first().unwrap()).to_string(),
                ) - 1
            }))
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SchuelerFile([Vec<Schueler>; 1]);

impl From<SchuelerFile> for BTreeMap<SchuelerId, SaveFileSchueler> {
    fn from(val: SchuelerFile) -> Self {
        let mut schueler_liste = BTreeMap::new();

        for schueler in val.0.first().unwrap() {
            schueler_liste.insert(
                SchuelerId::new(schueler.anmeldename),
                SaveFileSchueler {
                    name: schueler.vollstndigername.clone(),
                    uid: str::parse(&schueler.antworten).expect("Falsche Schueler-UID"),
                    wishes: schueler.get_wuensche(),
                    partner_raw: schueler.q06_wunschpartner.clone(),
                    ignore: false,
                    klasse: Klasse::new(schueler.gruppe.clone()),
                    partner: None,
                    fest: Some(false),
                },
            );
        }

        schueler_liste
    }
}
