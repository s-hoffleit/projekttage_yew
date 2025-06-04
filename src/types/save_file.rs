use std::collections::HashMap;

use gloo_console::log;
use gloo_storage::{LocalStorage, Storage, errors::StorageError};
use serde::{Deserialize, Serialize};

use crate::{
    Projekt,
    types::{Klasse, ProjektId, SchuelerId},
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct SaveFileStufe(u32);

impl SaveFileStufe {
    pub fn stufe(&self) -> u32 {
        self.0
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SaveFileKlasse {
    anzahl: u32,
    klassen: Vec<Klasse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SaveFileProjekt {
    pub name: String,
    #[serde(alias = "min")]
    pub min_teilnehmer: i32,
    #[serde(alias = "max")]
    pub max_teilnehmer: i32,
    pub min_stufe: u32,
    pub max_stufe: u32,
    pub ignore: bool,
}

impl From<SaveFileProjekt> for Projekt {
    fn from(val: SaveFileProjekt) -> Self {
        Projekt {
            name: val.name,
            stufen: (val.min_stufe..=val.max_stufe),
            teilnehmer: (val.min_teilnehmer..=val.max_teilnehmer),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SaveFileSchueler {
    pub uid: u32,
    pub name: String,
    pub wishes: Option<[ProjektId; 5]>,
    pub partner_raw: Option<String>,
    pub ignore: bool,
    pub klasse: Klasse,
    pub partner: Option<SchuelerId>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SaveFileZuordnung {
    id: u32,
    schueler: SchuelerId,
    projekt: Option<ProjektId>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SaveFile {
    pub klassen: HashMap<SaveFileStufe, SaveFileKlasse>,
    pub projekte: HashMap<ProjektId, SaveFileProjekt>,
    pub schueler: HashMap<SchuelerId, SaveFileSchueler>,
    pub zuordnung: Vec<SaveFileZuordnung>,
}

impl SaveFile {
    pub fn log(&self) {
        log!(self.klassen.len());
        log!(self.projekte.len());
        log!(self.schueler.len());
        log!(self.zuordnung.len());
    }

    pub fn save_to_local_storage(&self) -> Result<(), StorageError> {
        LocalStorage::set("klassen", self.klassen.clone())?;
        LocalStorage::set("projekte", self.projekte.clone())?;
        LocalStorage::set("schueler", self.schueler.clone())?;
        LocalStorage::set("zuordnung", self.zuordnung.clone())?;

        // todo!("Use use_state instead");

        Ok(())
    }

    pub fn load_from_local_storage() -> Result<Self, StorageError> {
        Ok(Self {
            klassen: LocalStorage::get("klassen")?,
            projekte: LocalStorage::get("projekte")?,
            schueler: LocalStorage::get("schueler")?,
            zuordnung: LocalStorage::get("zuordnung")?,
        })
    }
}
