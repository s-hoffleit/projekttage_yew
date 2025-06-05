use std::collections::{BTreeMap, HashMap};

use gloo_console::log;
use serde::Serialize;
use web_sys::{HtmlInputElement, wasm_bindgen::JsCast};
use yew::{
    Callback, Component, Context, ContextHandle, ContextProvider, Html, Properties,
    function_component, html, html::onchange, platform::spawn_local, use_context,
};
use yew_custom_components::table::types::{ColumnBuilder, TableData};

use crate::{
    Data, DataContext, Projekt,
    components::Tabelle,
    solver::solve_good_lp,
    types::{Klasse, ProjektId, SaveFileZuordnung, SchuelerId},
};

pub enum Msg {
    DataUpdate(DataContext),
    DataSet(Data),
    SolveButton,
    Solve(Data),
    Edit(SchuelerId, Edit),
}

pub struct Einteilung {
    data: DataContext,
    onchange: Callback<(SchuelerId, Edit)>,
    _context_listener: ContextHandle<DataContext>,
    verteilung: HashMap<SchuelerId, Option<ProjektId>>,
}

fn get_verteilung(data: &Data) -> HashMap<SchuelerId, Option<ProjektId>> {
    data.zuordnung
        .iter()
        .map(|zuordnung| (zuordnung.schueler, zuordnung.projekt))
        .collect::<HashMap<SchuelerId, Option<ProjektId>>>()
}

impl Component for Einteilung {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (data, context_listener) = ctx
            .link()
            .context::<DataContext>(ctx.link().callback(Msg::DataUpdate))
            .expect("Kein Datenkontext");

        log!("CREATE");

        if data.zuordnung.is_empty() {
            ctx.link().send_message(Msg::Solve(data.get()));
        }

        Self {
            verteilung: get_verteilung(&data),
            onchange: ctx
                .link()
                .callback(|(schueler_id, edit)| Msg::Edit(schueler_id, edit)),
            data,
            _context_listener: context_listener,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Column definition
        let columns = vec![
            ColumnBuilder::new("schueler_id")
                .orderable(true)
                .short_name("Schueler ID")
                .data_property("schueler_id")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("schueler_klasse")
                .orderable(true)
                .short_name("Klasse")
                .data_property("schueler_klasse")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("schueler_name")
                .orderable(true)
                .short_name("Schueler Name")
                .data_property("schueler_name")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("projekt")
                .orderable(true)
                .short_name("Projekt")
                .data_property("projekt")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("wuensche")
                .orderable(false)
                .short_name("Wünsche")
                .data_property("wuensche")
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
            <ContextProvider<Callback<(SchuelerId,Edit)>> context={ self.onchange.clone() }>
                <div class="seite">
                    <button onclick={ctx.link().callback(move |_| Msg::SolveButton)}>{"Lösen"}</button>
                    <Tabelle<EinteilungTableLine> columns={columns} table_data={table_data} />
                </div>
            </ContextProvider<Callback<(SchuelerId,Edit)>>>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataUpdate(data) => {
                self.data = data;
                let _ = self.data.save();

                true
            }
            Msg::DataSet(data) => {
                self.data.set(data);
                let _ = self.data.save();

                true
            }
            Msg::SolveButton => {
                ctx.link().send_message(Msg::Solve(self.data.get()));
                false
            }
            Msg::Solve(data) => {
                if data.zuordnung == self.data.zuordnung {
                    log!("Solve111!");

                    let link = ctx.link().clone();

                    let data2 = data.clone();

                    let callback = Callback::from(move |result: Option<Vec<SaveFileZuordnung>>| {
                        if let Some(result) = result {
                            let mut data = data2.clone();

                            for zuordnung in result.iter() {
                                if let Some(projekt_id) = zuordnung.projekt {
                                    let projekt = data.projekte.get_mut(&projekt_id);

                                    if let Some(projekt) = projekt {
                                        if let Some(num) = projekt.num_einteilung {
                                            projekt.num_einteilung = Some(num + 1);
                                        } else {
                                            projekt.num_einteilung = Some(1)
                                        }
                                    }
                                }
                            }

                            data.zuordnung = result;

                            link.send_message(Msg::DataSet(data));
                        }
                    });

                    spawn_local(async move {
                        let data = data.clone();

                        let a = solve_task(data);

                        let result = a.await;

                        callback.emit(result);
                    });

                    // let result = self.solve(&data);

                    // if let Some(verteilung) = result {
                    //     let mut data = self.data.get();

                    //     data.zuordnung = verteilung
                    //         .iter()
                    //         .enumerate()
                    //         .map(|(idx, (schueler_id, projekt_id))| SaveFileZuordnung {
                    //             id: idx as u32,
                    //             schueler: *schueler_id,
                    //             projekt: *projekt_id,
                    //         })
                    //         .collect::<Vec<SaveFileZuordnung>>();

                    //     ctx.link().send_message(Msg::DataSet(data));

                    //     self.verteilung = verteilung;

                    //     true
                    // } else {
                    //     false
                    // }

                    false
                } else {
                    self.verteilung = get_verteilung(&data);
                    true
                }
            }
            Msg::Edit(schueler_id, edit) => {
                log!("Edit");

                let mut data = self.data.get();

                let zuteilung = data.zuordnung.iter().find(|z| z.schueler == schueler_id);

                if let Some(_zuteilung) = zuteilung {
                    let mut zuordnungen = data.zuordnung;

                    match edit {
                        Edit::Projekt { projekt_id } => zuordnungen.iter_mut().for_each(|z| {
                            if z.schueler == schueler_id {
                                z.projekt = Some(projekt_id)
                            }
                        }),
                    }

                    data.zuordnung = zuordnungen;

                    ctx.link().send_message(Msg::DataSet(data));
                }

                true
            }
        }
    }
}

pub async fn solve_task(data: Data) -> Option<Vec<SaveFileZuordnung>> {
    log!("Start solve!");

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

        let mut verteilung: Vec<SaveFileZuordnung> = Vec::new();

        log!("Verteilung done!");

        for (schueler_idx, schueler_result) in result.iter().enumerate() {
            let schueler_id = solver_schueler_id_to_schueler_id.get(&schueler_idx);
            let projekt_index = schueler_result.iter().position(|wert| wert >= &0.5);
            let projekt_id =
                projekt_index.and_then(|p_idx| solver_projekte_id_to_projekte_id.get(&p_idx));

            if let Some(schueler_id) = schueler_id {
                verteilung.push(SaveFileZuordnung {
                    id: schueler_idx as u32,
                    schueler: *schueler_id,
                    projekt: projekt_id.cloned(),
                });
            }
        }

        log!("Solved!");

        Some(verteilung)
    } else {
        log!("Couldn't solve!");
        None
    }
}

#[derive(Clone, Serialize, Debug, Default)]
pub struct EinteilungTableLine {
    pub original_index: usize,
    pub schueler_id: SchuelerId,
    pub schueler_klasse: Klasse,
    pub schueler_name: String,
    pub projekt_id: Option<ProjektId>,
    pub projekt_name: String,
    pub wuensche: Option<[ProjektId; 5]>,
}

impl EinteilungTableLine {
    pub fn from_data(
        data: &Data,
        idx: usize,
        schueler_id: SchuelerId,
        projekt_id: Option<ProjektId>,
    ) -> Self {
        let schueler = data
            .get_schueler(&schueler_id)
            .expect("Schueler nicht gefunden");
        let projekt_name = if let Some(projekt_id) = projekt_id {
            data.get_projekt(&projekt_id).unwrap().name.clone()
        } else {
            String::new()
        };

        Self {
            original_index: idx,
            schueler_id,
            schueler_klasse: schueler.klasse.clone(),
            projekt_id,
            schueler_name: schueler.name.clone(),
            projekt_name,
            wuensche: schueler.wishes,
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

pub enum Edit {
    Projekt { projekt_id: ProjektId },
}

#[derive(Properties, PartialEq)]
struct ProjektSelectProps {
    schueler_id: SchuelerId,
    selected: Option<ProjektId>,
    class: String,
}

#[function_component(ProjektSelect)]
fn project_selection(props: &ProjektSelectProps) -> Html {
    let data = use_context::<DataContext>();
    let on_change = use_context::<Callback<(SchuelerId, Edit)>>();

    if on_change.is_none() {
        return html!(<></>);
    }

    let on_change = on_change.unwrap();

    let schueler_id: SchuelerId = props.schueler_id;

    let onchange = Callback::from(move |event: onchange::Event| {
        let event = event.target();
        if let Some(event) = event {
            let projekt_id = event.unchecked_into::<HtmlInputElement>().value().into();

            on_change.emit((schueler_id, Edit::Projekt { projekt_id }))
        }
        // on_change.emit(schueler_id, Edit::Wunsch { idx: wunsch_idx, value: () });
    });

    if let Some(data) = data {
        html! {
            <select id="wish_select" { onchange } class={ props.class.clone() } >
                { for data.projekte.iter().map(|(p_id, projekt)| html! {
                    <option value={ format!("{}", p_id.id()) } selected={ props.selected == Some(*p_id) }> {format!("{p_id}: {}", projekt.name.clone())} </option>
                })}
                <option value="-1"> { "Kein Wunsch" } </option>
            </select>
        }
    } else {
        html!( <></> )
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
            "projekt" => {
                let Some(wuensche) = self.wuensche else {
                    return Ok(
                        html! (<ProjektSelect selected={ self.projekt_id } schueler_id={ self.schueler_id } class="" />),
                    );
                };

                let wunsch_idx = wuensche.iter().position(|w| Some(*w) == self.projekt_id);

                Ok(
                    html! (<ProjektSelect selected={ self.projekt_id } schueler_id={ self.schueler_id } class={("wunsch_".to_owned() + &wunsch_idx.unwrap_or(5).to_string()).to_string()} />),
                )
            }
            "schueler_klasse" => {
                Ok(html!(<span>{format!("{}", self.schueler_klasse.klasse())}</span>))
            }
            "wuensche" => {
                let Some(wuensche) = self.wuensche else {
                    return Ok(html!(<span>{"---"}</span>));
                };

                Ok(html!(<span>{wuensche.map(|w| w.to_string()).join(", ")}</span>))
            }
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
            "projekt" => Ok(serde_value::Value::Option(
                self.projekt_id
                    .map(|p_id| Box::new(serde_value::Value::U32(p_id.id()))),
            )),
            "schueler_klasse" => Ok(serde_value::Value::U32(
                self.schueler_klasse.stufe().unwrap_or(0),
            )),
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
