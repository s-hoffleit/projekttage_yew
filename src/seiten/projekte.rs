use std::collections::BTreeMap;

use gloo_console::log;
use gloo_storage::{LocalStorage, Storage};
use serde::Serialize;
use yew::{Component, Context, Html, html, use_effect};
use yew_custom_components::table::types::{ColumnBuilder, TableData};

use crate::{
    Projekt,
    components::Tabelle,
    types::{ProjektId, SaveFileProjekt},
};

pub enum Msg {}

pub struct Projekte {
    projekte: BTreeMap<ProjektId, Projekt>,
}

impl Component for Projekte {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            projekte: LocalStorage::get::<BTreeMap<ProjektId, SaveFileProjekt>>("projekte")
                .unwrap()
                .iter()
                .map(|(&projekt_id, s_f_project)| (projekt_id, s_f_project.clone().into()))
                .collect::<BTreeMap<ProjektId, Projekt>>(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // let projekte = use_state(|| {
        //     vec![
        //         Projekt {
        //             id: 0,
        //             name: "Test".into(),
        //             stufen: (6..=12),
        //             teilnehmer: (10..=50),
        //         },
        //         Projekt {
        //             id: 1,
        //             name: "Test2".into(),
        //             stufen: (5..=8),
        //             teilnehmer: (0..=100),
        //         },
        //     ]
        // });

        {
            let projekte = self.projekte.clone();

            // For storage
            use_effect(move || {
                log!(
                    "projekte changed to: ",
                    serde_wasm_bindgen::to_value(&projekte.clone()).unwrap()
                );
            });
        }

        // Column definition
        let columns = vec![
            ColumnBuilder::new("id")
                .orderable(true)
                .short_name("ID")
                .data_property("id")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("name")
                .orderable(true)
                .short_name("Name")
                .data_property("name")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("min_stufe")
                .orderable(true)
                .short_name("Mindeste Stufe")
                .data_property("min_stufe")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("max_stufe")
                .orderable(true)
                .short_name("Maximale Stufe")
                .data_property("max_stufe")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("min_teilnehmer")
                .orderable(false)
                .short_name("Mindeste Teilnehmeranzahl")
                .data_property("min_teilnehmer")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("max_teilnehmer")
                .orderable(false)
                .short_name("Maximale Teilnehmeranzahl")
                .data_property("max_teilnehmer")
                .header_class("user-select-none")
                .build(),
        ];

        let mut table_data = Vec::new();
        for (index, (projekt_id, projekt)) in self.projekte.iter().enumerate() {
            table_data.push(ProjektTableLine {
                original_index: index,
                id: *projekt_id,
                name: projekt.name.clone(),
                min_stufe: *projekt.stufen.start(),
                max_stufe: *projekt.stufen.end(),
                min_teilnehmer: *projekt.teilnehmer.start(),
                max_teilnehmer: *projekt.teilnehmer.end(),
            });
        }

        html! {
            <div class="seite">
                <Tabelle<ProjektTableLine> columns={columns} table_data={table_data} />
            </div>
        }
    }
}

#[derive(Clone, Serialize, Debug, Default)]
pub struct ProjektTableLine {
    pub original_index: usize,
    pub id: ProjektId,
    pub name: String,
    pub min_stufe: u32,
    pub max_stufe: u32,
    pub min_teilnehmer: i32,
    pub max_teilnehmer: i32,
}

impl PartialEq<Self> for ProjektTableLine {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for ProjektTableLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.id().partial_cmp(&other.id.id())
    }
}

impl TableData for ProjektTableLine {
    fn get_field_as_html(
        &self,
        field_name: &str,
    ) -> yew_custom_components::table::error::Result<Html> {
        match field_name {
            "id" => Ok(html! (<span>{format!("{}", self.id)}</span>)),
            "name" => Ok(html! (<span>{self.name.clone()}</span>)),
            "min_stufe" => Ok(html! (<span>{self.min_stufe}</span>)),
            "max_stufe" => Ok(html! (<span>{self.max_stufe}</span>)),
            "min_teilnehmer" => Ok(html! (<span>{self.min_teilnehmer}</span>)),
            "max_teilnehmer" => Ok(html! (<span>{self.max_teilnehmer}</span>)),
            _ => Ok(html! {}),
        }
    }

    fn get_field_as_value(
        &self,
        field_name: &str,
    ) -> yew_custom_components::table::error::Result<serde_value::Value> {
        match field_name {
            "id" => Ok(serde_value::Value::U32(self.id.id())),
            "name" => Ok(serde_value::Value::String(self.name.clone())),
            "min_stufe" => Ok(serde_value::Value::U32(self.min_stufe)),
            "max_stufe" => Ok(serde_value::Value::U32(self.max_stufe)),
            "min_teilnehmer" => Ok(serde_value::Value::I32(self.min_teilnehmer)),
            "max_teilnehmer" => Ok(serde_value::Value::I32(self.max_teilnehmer)),
            _ => Ok(serde_value::to_value(()).unwrap()),
        }
    }

    fn matches_search(&self, needle: Option<String>) -> bool {
        match needle {
            Some(needle) => self.name.to_lowercase().contains(&needle.to_lowercase()),
            None => true,
        }
    }
}
