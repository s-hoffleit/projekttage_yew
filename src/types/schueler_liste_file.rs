use std::collections::BTreeMap;

use gloo_console::log;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{Klasse, SaveFileSchueler, SchuelerId};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Schueler {
    id: String,
    anmeldename: String,
    vorname: String,
    nachname: String,
    gruppen: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SchuelerListeFile([Vec<Schueler>; 1]);

impl From<SchuelerListeFile> for BTreeMap<SchuelerId, SaveFileSchueler> {
    fn from(val: SchuelerListeFile) -> Self {
        log!("AAAAAAAA");
        let mut schueler_liste = BTreeMap::new();

        for schueler in val.0.first().unwrap() {
            if schueler.anmeldename.is_empty() {
                continue;
            }
            schueler_liste.insert(
                SchuelerId::new(Uuid::parse_str(&schueler.anmeldename).expect("Falsche Uuid")),
                SaveFileSchueler {
                    name: format!("{} {}", schueler.vorname.clone(), schueler.nachname.clone()),
                    uid: str::parse(&schueler.id).expect("Falsche Schueler-UID"),
                    wishes: None,
                    partner_raw: None,
                    ignore: true,
                    klasse: Klasse::new(schueler.gruppen.clone()),
                    partner: None,
                    fest: None,
                },
            );
        }

        schueler_liste
    }
}
