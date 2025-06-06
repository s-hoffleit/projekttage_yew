use std::collections::BTreeMap;

use gloo_console::log;
use gloo_storage::{LocalStorage, Storage, errors::StorageError};
use serde::{Deserialize, Serialize};

use crate::{
    Data, Projekt,
    types::{Klasse, ProjektId, SchuelerId},
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct SaveFileStufe(u32);

impl SaveFileStufe {
    pub fn stufe(&self) -> u32 {
        self.0
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
    pub num_einteilung: Option<u32>,
}

impl From<SaveFileProjekt> for Projekt {
    fn from(val: SaveFileProjekt) -> Self {
        Projekt {
            name: val.name,
            stufen: (val.min_stufe..=val.max_stufe),
            teilnehmer: (val.min_teilnehmer..=val.max_teilnehmer),
            num_einteilung: val.num_einteilung,
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
    pub fest: Option<bool>,
    pub klasse: Klasse,
    pub partner: Option<SchuelerId>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct SaveFileZuordnung {
    pub id: u32,
    pub schueler: SchuelerId,
    pub projekt: Option<ProjektId>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SaveFile {
    pub klassen: BTreeMap<SaveFileStufe, SaveFileKlasse>,
    pub projekte: BTreeMap<ProjektId, SaveFileProjekt>,
    pub schueler: BTreeMap<SchuelerId, SaveFileSchueler>,
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
            klassen: LocalStorage::get("klassen").unwrap_or(BTreeMap::new()),
            projekte: LocalStorage::get("projekte").unwrap_or(BTreeMap::new()),
            schueler: LocalStorage::get("schueler").unwrap_or(BTreeMap::new()),
            zuordnung: LocalStorage::get("zuordnung").unwrap_or(Vec::new()),
        })
    }
}

impl From<SaveFile> for Data {
    fn from(val: SaveFile) -> Self {
        Data {
            projekte: val.projekte,
            schueler: val.schueler,
            zuordnung: val.zuordnung,
            klassen: val.klassen,
        }
    }
}
