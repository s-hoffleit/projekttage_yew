use std::collections::BTreeMap;

use gloo_console::log;
use serde::Serialize;
use yew::{Component, Context, ContextHandle, Html, html};
use yew_custom_components::table::types::{ColumnBuilder, TableData};

use crate::{
    Data, Projekt,
    components::Tabelle,
    solver::solve_good_lp,
    types::{Klasse, ProjektId, SaveFileSchueler, SchuelerId},
};

pub enum Msg {
    DataUpdate(Data),
}

pub struct Einteilung {
    data: Data,
    _context_listener: ContextHandle<Data>,
}

impl Component for Einteilung {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (data, context_listener) = ctx
            .link()
            .context::<Data>(ctx.link().callback(Msg::DataUpdate))
            .expect("Kein Datenkontext");

        log!("CREATE");

        let feste_zuordnung = BTreeMap::new();

        let projekte = &data
            .projekte
            .iter()
            .map(|(&p_id, project)| (p_id, project.clone().into()))
            .collect::<BTreeMap<ProjektId, Projekt>>();

        let schueler = &data.schueler;

        let result = solve_good_lp(projekte, schueler, &feste_zuordnung);

        if let Ok((_solution, _result)) = result {
            log!("RES!");
        } else {
            log!("Couldn't solve!");
        }

        Self {
            data,
            _context_listener: context_listener,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // Column definition
        let columns = vec![
            ColumnBuilder::new("id")
                .orderable(true)
                .short_name("ID")
                .data_property("id")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("klasse")
                .orderable(true)
                .short_name("Klasse")
                .data_property("klasse")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("name")
                .orderable(true)
                .short_name("Name")
                .data_property("name")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("wunsch1")
                .orderable(true)
                .short_name("Wunsch 1")
                .data_property("wunsch1")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("wunsch2")
                .orderable(true)
                .short_name("Wunsch 2")
                .data_property("wunsch2")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("wunsch3")
                .orderable(true)
                .short_name("Wunsch 3")
                .data_property("wunsch3")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("wunsch4")
                .orderable(true)
                .short_name("Wunsch 4")
                .data_property("wunsch4")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("wunsch5")
                .orderable(true)
                .short_name("Wunsch 5")
                .data_property("wunsch5")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("partner")
                .orderable(true)
                .short_name("Wunschpartner")
                .data_property("partner")
                .header_class("user-select-none")
                .build(),
        ];

        fn get_wuensche(
            schueler: &SaveFileSchueler,
            data: &Data,
        ) -> [Option<(ProjektId, String)>; 5] {
            if let Some(projekte) = schueler.wishes {
                projekte
                    .map(|p_id| data.projekte.get(&p_id).map(|projekt| (p_id, projekt)))
                    .map(|option| {
                        option.map(|(projekt_id, projekt)| (projekt_id, projekt.name.clone()))
                    })
            } else {
                [const { None }; 5]
            }
        }

        let mut table_data = Vec::new();
        for (index, (schueler_id, schueler)) in self.data.schueler.iter().enumerate() {
            table_data.push(SchuelerTableLine {
                original_index: index,
                id: *schueler_id,
                klasse: schueler.klasse.clone(),
                name: schueler.name.clone(),
                wuensche: get_wuensche(schueler, &self.data),
                partner: schueler
                    .partner
                    .map(|p| (p, self.data.get_schueler(&p).unwrap().clone().name)),
            });
        }

        html! {
            <div class="seite">
                <Tabelle<SchuelerTableLine> columns={columns} table_data={table_data} />
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataUpdate(data) => {
                self.data = data;
                log!("data");
                true
            }
        }
    }
}

#[derive(Clone, Serialize, Debug, Default)]
pub struct SchuelerTableLine {
    pub original_index: usize,
    pub id: SchuelerId,
    pub klasse: Klasse,
    pub name: String,
    pub wuensche: [Option<(ProjektId, String)>; 5],
    pub partner: Option<(SchuelerId, String)>,
}

impl PartialEq<Self> for SchuelerTableLine {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for SchuelerTableLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.id().partial_cmp(&other.id.id())
    }
}

impl TableData for SchuelerTableLine {
    fn get_field_as_html(
        &self,
        field_name: &str,
    ) -> yew_custom_components::table::error::Result<Html> {
        match field_name {
            "id" => Ok(html! (<span>{format!("{}", self.id)}</span>)),
            "name" => Ok(html! (<span>{self.name.clone()}</span>)),
            "klasse" => Ok(html! (<span>{self.klasse.klasse()}</span>)),
            w if w.starts_with("wunsch") => {
                let wunsch = self.wuensche[w
                    .chars()
                    .last()
                    .expect("Falscher field_name")
                    .to_digit(10)
                    .expect("Falscher field_name")
                    as usize
                    - 1]
                .as_ref();

                Ok(html! (<span>{wunsch.map(|w| format!("{}: {}", w.0, w.1.clone()))}</span>))
            }
            "partner" => Ok(html! (<span>{self.partner.as_ref().map(|p| p.1.clone())}</span>)),
            _ => Ok(html! {}),
        }
    }

    fn get_field_as_value(
        &self,
        field_name: &str,
    ) -> yew_custom_components::table::error::Result<serde_value::Value> {
        match field_name {
            "id" => Ok(serde_value::Value::String(
                self.id.id().as_hyphenated().to_string(),
            )),
            "klasse" => Ok(serde_value::Value::String(self.klasse.klasse())),
            "name" => Ok(serde_value::Value::String(self.name.clone())),
            w if w.starts_with("wunsch") => Ok(serde_value::Value::Option(
                self.wuensche[w
                    .chars()
                    .last()
                    .expect("Falscher field_name")
                    .to_digit(10)
                    .expect("Falscher field_name") as usize
                    - 1]
                .as_ref()
                .map(|w| w.0)
                .map(|project_id| Box::new(serde_value::Value::U32(project_id.id()))),
            )),
            "partner" => Ok(serde_value::Value::Option(
                self.partner
                    .as_ref()
                    .map(|p| p.1.clone())
                    .map(|project| Box::new(serde_value::Value::String(project))),
            )),
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
