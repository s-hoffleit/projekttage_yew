use std::collections::{BTreeMap, HashMap};

use gloo_console::log;
use serde::Serialize;
use yew::{Component, Context, ContextHandle, Html, html};
use yew_custom_components::table::types::{ColumnBuilder, TableData};

use crate::{
    Data, Projekt,
    components::Tabelle,
    solver::solve_good_lp,
    types::{ProjektId, SchuelerId},
};

pub enum Msg {
    DataUpdate(Data),
    Solve(Data),
}

pub struct Einteilung {
    data: Data,
    verteilung: HashMap<SchuelerId, ProjektId>,
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

        ctx.link().send_message(Msg::Solve(data.clone()));

        Self {
            data,
            verteilung: HashMap::new(),
            _context_listener: context_listener,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // Column definition
        let columns = vec![
            ColumnBuilder::new("schueler_id")
                .orderable(true)
                .short_name("Schueler ID")
                .data_property("schueler_id")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("schueler_name")
                .orderable(true)
                .short_name("Schueler Name")
                .data_property("schueler_name")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("projekt_id")
                .orderable(true)
                .short_name("Projekt ID")
                .data_property("projekt_id")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("projekt_name")
                .orderable(true)
                .short_name("Projekt-Name")
                .data_property("projekt_name")
                .header_class("user-select-none")
                .build(),
        ];

        let mut table_data = Vec::new();
        for (idx, (&schueler_id, &projekt_id)) in self.verteilung.iter().enumerate() {
            table_data.push(EinteilungTableLine::from_data(
                &self.data,
                idx,
                schueler_id,
                projekt_id,
            ));
        }

        html! {
            <div class="seite">
                <Tabelle<EinteilungTableLine> columns={columns} table_data={table_data} />
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
            Msg::Solve(data) => {
                let result = self.solve(&data);

                if let Some(verteilung) = result {
                    self.verteilung = verteilung;
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Einteilung {
    pub fn solve(&self, data: &Data) -> Option<HashMap<SchuelerId, ProjektId>> {
        let feste_zuordnung = BTreeMap::new();

        let projekte = &data
            .projekte
            .iter()
            .map(|(&p_id, project)| (p_id, project.clone().into()))
            .collect::<BTreeMap<ProjektId, Projekt>>();

        let schueler = &data.schueler;

        let result = solve_good_lp(projekte, schueler, &feste_zuordnung);

        if let Ok(result) = result {
            let solver_projekte_id_to_projekte_id = projekte
                .iter()
                .enumerate()
                .map(|(id, (&p_id, _))| (id, p_id))
                .collect::<HashMap<usize, ProjektId>>();

            let solver_schueler_id_to_schueler_id = schueler
                .iter()
                .enumerate()
                .map(|(id, (&s_id, _))| (id, s_id))
                .collect::<HashMap<usize, SchuelerId>>();

            let mut verteilung: HashMap<SchuelerId, ProjektId> = HashMap::new();

            for (schueler_id, schueler_result) in result.iter().enumerate() {
                let schueler_id = solver_schueler_id_to_schueler_id.get(&schueler_id);
                let projekt_index = schueler_result
                    .iter()
                    .position(|wert| (wert - 1.0).abs() <= f64::EPSILON);
                let projekt_id =
                    projekt_index.and_then(|p_idx| solver_projekte_id_to_projekte_id.get(&p_idx));

                if let (Some(schueler_id), Some(projekt_id)) = (schueler_id, projekt_id) {
                    verteilung.insert(*schueler_id, *projekt_id);
                }
            }

            log!("RES!");

            Some(verteilung)
        } else {
            log!("Couldn't solve!");
            None
        }
    }
}

#[derive(Clone, Serialize, Debug, Default)]
pub struct EinteilungTableLine {
    pub original_index: usize,
    pub schueler_id: SchuelerId,
    pub schueler_name: String,
    pub projekt_id: ProjektId,
    pub projekt_name: String,
}

impl EinteilungTableLine {
    pub fn from_data(
        data: &Data,
        idx: usize,
        schueler_id: SchuelerId,
        projekt_id: ProjektId,
    ) -> Self {
        let schueler_name = data.get_schueler(&schueler_id).unwrap().name.clone();
        let projekt_name = data.get_projekt(&projekt_id).unwrap().name.clone();

        Self {
            original_index: idx,
            schueler_id,
            projekt_id,
            schueler_name,
            projekt_name,
        }
    }
}

impl PartialEq<Self> for EinteilungTableLine {
    fn eq(&self, other: &Self) -> bool {
        self.schueler_id == other.schueler_id
    }
}

impl PartialOrd for EinteilungTableLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.schueler_id.id().partial_cmp(&other.schueler_id.id())
    }
}

impl TableData for EinteilungTableLine {
    fn get_field_as_html(
        &self,
        field_name: &str,
    ) -> yew_custom_components::table::error::Result<Html> {
        match field_name {
            "schueler_id" => Ok(html! (<span>{format!("{}", self.schueler_id)}</span>)),
            "schueler_name" => Ok(html! (<span>{format!("{}", self.schueler_name)}</span>)),
            "projekt_id" => Ok(html! (<span>{format!("{}", self.projekt_id)}</span>)),
            "projekt_name" => Ok(html! (<span>{format!("{}", self.projekt_name)}</span>)),
            _ => Ok(html! {}),
        }
    }

    fn get_field_as_value(
        &self,
        field_name: &str,
    ) -> yew_custom_components::table::error::Result<serde_value::Value> {
        match field_name {
            "schueler_id" => Ok(serde_value::Value::String(format!(
                "{}",
                self.schueler_id.id()
            ))),
            "schueler_name" => Ok(serde_value::Value::String(self.schueler_name.clone())),
            "projekt_id" => Ok(serde_value::Value::U32(self.projekt_id.id())),
            "projekt_name" => Ok(serde_value::Value::String(self.projekt_name.clone())),
            _ => Ok(serde_value::to_value(()).unwrap()),
        }
    }

    fn matches_search(&self, needle: Option<String>) -> bool {
        match needle {
            Some(needle) => {
                self.projekt_name
                    .to_lowercase()
                    .contains(&needle.to_lowercase())
                    || self
                        .schueler_name
                        .to_lowercase()
                        .contains(&needle.to_lowercase())
            }
            None => true,
        }
    }
}
