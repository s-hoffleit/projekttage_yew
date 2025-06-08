use std::collections::BTreeMap;

use gloo_console::log;
use regex::Regex;
use serde::Serialize;
use unicode_normalization::UnicodeNormalization;
use web_sys::{HtmlInputElement, wasm_bindgen::JsCast};
use yew::{
    AttrValue, Callback, Component, Context, ContextHandle, ContextProvider, Html, Properties,
    UseStateHandle, classes, function_component, html, html::onchange, use_context,
};
use yew_custom_components::table::types::{ColumnBuilder, TableData};

use crate::{
    Data, DataContext,
    components::Tabelle,
    types::{Klasse, ProjektId, SaveFileSchueler, SchuelerId},
};

pub enum Msg {
    DataUpdate(UseStateHandle<Data>),
    DataSet(Data),
    Edit(SchuelerId, Edit),
}

pub enum Edit {
    Wunsch { idx: u8, projekt_id: ProjektId },
    Fest { value: bool },
    Ignorieren { value: bool },
    Partner { value: String },
}

#[derive(Properties, PartialEq)]
struct ProjektSelectProps {
    schueler_id: SchuelerId,
    selected: Option<ProjektId>,
    wunsch_idx: u8,
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
    let wunsch_idx = props.wunsch_idx;

    let onchange = Callback::from(move |event: onchange::Event| {
        let event = event.target();
        if let Some(event) = event {
            let projekt_id = event.unchecked_into::<HtmlInputElement>().value().into();

            on_change.emit((
                schueler_id,
                Edit::Wunsch {
                    idx: wunsch_idx,
                    projekt_id,
                },
            ))
        }
        // on_change.emit(schueler_id, Edit::Wunsch { idx: wunsch_idx, value: () });
    });

    if let Some(data) = data {
        html! {
            <select id="wish_select" { onchange } >
                <option value="-1" selected={ props.selected.is_none() || props.selected.map(|p_id| p_id.id()) == Some(u32::MAX) }> { "Kein Wunsch" } </option>
                { for data.projekte.iter().map(|(p_id, projekt)| html! {
                    <option value={ format!("{}", p_id.id()) } selected={ props.selected == Some(*p_id) }> {format!("{p_id}: {}", projekt.name.clone())} </option>
                })}
            </select>
        }
    } else {
        html!( <></> )
    }
}

pub struct Schueler {
    data: DataContext,
    onchange: Callback<(SchuelerId, Edit)>,
    _context_listener: ContextHandle<DataContext>,
}

#[derive(Properties, PartialEq)]
pub struct SchuelerProps {
    pub data: UseStateHandle<DataContext>,
}

pub fn find_partner(
    data: &BTreeMap<SchuelerId, SaveFileSchueler>,
    partner_raw: String,
) -> Option<SchuelerId> {
    let regex = Regex::new(
        r"(?<vorname>(?:\p{L}|-)+)(?:\s?(?<mittename>(?:\s??(?:\p{L}|-)+)*?) (?<nachname>(?:\p{L}|-)+))?(?:[|,\s(/\\]|(?:Klasse))*(?<klasse>\d{1,2}[a-zA-Z])?[)]?",
    ).unwrap();

    let captures = regex.captures(&partner_raw)?;

    let klasse = captures.name("klasse").map(|klasse| klasse.as_str());

    let mut filtered_schueler = if let Some(klasse) = klasse {
        data.clone()
            .into_iter()
            .filter(|(_, s)| s.klasse.klasse() == klasse)
            .collect::<BTreeMap<SchuelerId, SaveFileSchueler>>()
    } else {
        data.clone()
    };

    if filtered_schueler.is_empty() {
        filtered_schueler = data.clone();
    }

    // Schüler nach Vornamen filtern, wenn nur einmal => fertig

    let vorname = captures
        .name("vorname")
        .unwrap()
        .as_str()
        .nfkd()
        .collect::<String>()
        .to_lowercase();

    let filtered_schueler = filtered_schueler
        .into_iter()
        .filter(|(_, s)| {
            s.name
                .split(" ")
                .collect::<Vec<&str>>()
                .first()
                .map(|name| name.nfkd().collect::<String>().to_lowercase() == vorname)
                == Some(true)
        })
        .collect::<BTreeMap<SchuelerId, SaveFileSchueler>>();

    log!(format!("{}", filtered_schueler.len()));

    if filtered_schueler.len() == 1 {
        return filtered_schueler
            .iter()
            .find(|_| true)
            .map(|(s_id, _)| *s_id);
    } else if filtered_schueler.is_empty() {
        return None;
    }

    let nachname = captures
        .name("nachname")
        .map(|name| name.as_str().nfkd().collect::<String>().to_lowercase());

    let filtered_schueler = if let Some(nachname) = nachname.clone() {
        filtered_schueler
            .into_iter()
            .filter(|(_, s)| {
                s.name
                    .split(" ")
                    .collect::<Vec<&str>>()
                    .last()
                    .map(|name| name.nfkd().collect::<String>().to_lowercase() == nachname)
                    == Some(true)
            })
            .collect::<BTreeMap<SchuelerId, SaveFileSchueler>>()
    } else {
        filtered_schueler
    };

    log!(format!("{}", filtered_schueler.len()));

    if filtered_schueler.len() == 1 {
        return filtered_schueler
            .iter()
            .find(|_| true)
            .map(|(s_id, _)| *s_id);
    } else if filtered_schueler.is_empty() {
        return None;
    }

    let mittename = captures
        .name("mittename")
        .map(|name| name.as_str().nfkd().collect::<String>().to_lowercase());

    let mut voller_name = vorname;

    if let Some(mittename) = mittename {
        voller_name += &(" ".to_owned() + &mittename);
    }

    if let Some(nachname) = nachname {
        voller_name += &(" ".to_owned() + &nachname);
    }

    let filtered_schueler = if let Some(klasse) = klasse {
        data.clone()
            .into_iter()
            .filter(|(_, s)| s.klasse.klasse() == klasse)
            .collect::<BTreeMap<SchuelerId, SaveFileSchueler>>()
    } else {
        data.clone()
    }
    .into_iter()
    .filter(|(_, s)| s.name.nfkd().collect::<String>() == voller_name)
    .collect::<BTreeMap<SchuelerId, SaveFileSchueler>>();

    log!(format!("{}", filtered_schueler.len()));

    if filtered_schueler.len() == 1 {
        return filtered_schueler
            .iter()
            .find(|_| true)
            .map(|(s_id, _)| *s_id);
    } else if filtered_schueler.is_empty() {
        return None;
    }

    // log!(format!(
    //     "Vorname: {:?}, Mittename: {:?}, Nachname: {:?}, Klasse: {:?}",
    //     &captures.name("vorname"),
    //     &captures.name("mittename"),
    //     &captures.name("nachname"),
    //     &captures.name("klasse")
    // ));

    None
}

pub fn match_all_partner(
    data: &BTreeMap<SchuelerId, SaveFileSchueler>,
) -> BTreeMap<SchuelerId, SaveFileSchueler> {
    let mut mut_data = data.clone();
    for (_schueler_id, schueler_data) in mut_data.iter_mut() {
        if let Some(partner_raw) = &schueler_data.partner_raw {
            let partner = find_partner(data, partner_raw.clone());

            if let Some(partner) = partner {
                log!(format!("{} + {}", _schueler_id, partner));
            }

            schueler_data.partner = partner;
        }
    }

    mut_data.clone()
}

#[function_component(Datalist)]
fn datalist() -> Html {
    let data = use_context::<DataContext>();

    let schueler_liste = if let Some(data) = data {
        let data = data.get();

        data.schueler
            .values()
            .map(|s| html!(<option value={format!("{} ({})", s.name, s.klasse.klasse())} />))
            .collect::<Vec<Html>>()
    } else {
        vec![]
    };

    html! {
        <datalist id="schueler_datalist">
            { schueler_liste.into_iter().collect::<Html>() }
        </datalist>
    }
}

impl Component for Schueler {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (data, context_listener) = ctx
            .link()
            .context::<DataContext>(ctx.link().callback(Msg::DataUpdate))
            .expect("Kein Datenkontext");

        log!("CREATE");

        let schueler = match_all_partner(&data.schueler);

        let mut data2 = data.get();

        data2.schueler = schueler;

        data.set(data2);

        // log!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");

        // let partner = find_partner(
        //     &data.schueler,
        //     "Jonathan David Hoffleit (Klasse 10c)".to_string(),
        // );

        // log!(format!("{:?}", partner));

        Self {
            data,
            onchange: ctx
                .link()
                .callback(|(schueler_id, edit)| Msg::Edit(schueler_id, edit)),
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
            ColumnBuilder::new("fest")
                .orderable(true)
                .short_name("Feste Einteilungen")
                .data_property("fest")
                .header_class("user-select-none")
                .build(),
            ColumnBuilder::new("ignorieren")
                .orderable(true)
                .short_name("Ignorieren")
                .data_property("ignorieren")
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
                partner: schueler.partner.map(|p| {
                    let partner = self.data.get_schueler(&p).unwrap().clone();
                    (p, partner.name, partner.klasse)
                }),
                partner_raw: schueler.partner_raw.clone(),
                fest: schueler.fest.unwrap_or(false),
                ignorieren: schueler.ignore,
            });
        }

        html! {
            <ContextProvider<Callback<(SchuelerId,Edit)>> context={ self.onchange.clone() }>
                <Datalist />
                <div class="seite">
                    <Tabelle<SchuelerTableLine> columns={columns} table_data={table_data} />
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
            Msg::Edit(schueler_id, edit) => {
                log!("Edit");

                let mut data = self.data.get();

                let schueler = data.schueler.get(&schueler_id);

                if let Some(schueler) = schueler {
                    let mut schueler = schueler.clone();

                    match edit {
                        Edit::Wunsch { idx, projekt_id } => {
                            if schueler.wishes.is_none() {
                                schueler.wishes = Some([ProjektId::from("-1".to_string()); 5]);
                            }
                            schueler.wishes = schueler.wishes.map(|mut wishes| {
                                wishes[idx as usize] = projekt_id;
                                wishes
                            });
                        }
                        Edit::Fest { value } => schueler.fest = Some(value),
                        Edit::Ignorieren { value } => schueler.ignore = value,
                        Edit::Partner { value } => {
                            schueler.partner = find_partner(&data.schueler, value)
                        }
                    }

                    let mut schueler_map = data.schueler;
                    schueler_map.insert(schueler_id, schueler);

                    data.schueler = schueler_map;

                    ctx.link().send_message(Msg::DataSet(data));
                }

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
    pub partner: Option<(SchuelerId, String, Klasse)>,
    pub partner_raw: Option<String>,
    pub fest: bool,
    pub ignorieren: bool,
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

#[derive(Properties, PartialEq)]
struct CheckboxProps {
    value: bool,
    object_key: AttrValue,
    schueler: SchuelerId,
}

#[function_component(Checkbox)]
fn checkbox(props: &CheckboxProps) -> Html {
    let on_change = use_context::<Callback<(SchuelerId, Edit)>>();

    if on_change.is_none() {
        return html!(<></>);
    }

    let on_change = on_change.unwrap();

    let schueler_id = props.schueler;
    let key = props.object_key.clone();

    let onchange = Callback::from(move |event: onchange::Event| {
        let event = event.target();
        if let Some(event) = event {
            let value = event.unchecked_into::<HtmlInputElement>().checked();

            if key == "fest" {
                on_change.emit((schueler_id, Edit::Fest { value }))
            } else if key == "ignorieren" {
                on_change.emit((schueler_id, Edit::Ignorieren { value }))
            }
        }
        // on_change.emit(schueler_id, Edit::Wunsch { idx: wunsch_idx, value: () });
    });

    html! (<input type="checkbox" checked={props.value} { onchange } />)
}

#[derive(Properties, PartialEq)]
struct PartnerProps {
    schueler: SchuelerId,
    partner: Option<(SchuelerId, String, Klasse)>,
    partner_raw: Option<String>,
}

#[function_component(Partner)]
fn partner(props: &PartnerProps) -> Html {
    let on_change = use_context::<Callback<(SchuelerId, Edit)>>();

    if on_change.is_none() {
        return html!(<></>);
    }

    let on_change = on_change.unwrap();

    let schueler_id = props.schueler;

    let onchange = Callback::from(move |event: onchange::Event| {
        let event = event.target();
        if let Some(event) = event {
            let value = event.unchecked_into::<HtmlInputElement>().value();

            on_change.emit((schueler_id, Edit::Partner { value }))
        }
        // on_change.emit(schueler_id, Edit::Wunsch { idx: wunsch_idx, value: () });
    });

    if let Some((_id, partner_name, partner_klasse)) = props.partner.clone() {
        html! (<span><input type="text" class={classes!("partner")} value={format!("{} ({})", partner_name, partner_klasse.klasse())} { onchange } list="schueler_datalist" /></span>)
    } else {
        html! (<span><input type="text" class={classes!("raw_partner")} value={props.partner_raw.clone()} { onchange } list="schueler_datalist" /></span>)
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
                let wunsch_idx = w
                    .chars()
                    .last()
                    .expect("Falscher field_name")
                    .to_digit(10)
                    .expect("Falscher field_name") as usize
                    - 1;
                let wunsch = self.wuensche[wunsch_idx].as_ref();

                Ok(html! {
                    <span>
                        <ProjektSelect selected={wunsch.map(|w| w.0)} schueler_id={self.id} wunsch_idx = { wunsch_idx as u8 } />
                    </span>
                })

                // Ok(html! (<span>{wunsch.map(|w| format!("{}: {}", w.0, w.1.clone()))}</span>))
            }
            "partner" => Ok(
                html! (<Partner schueler={self.id} partner={self.partner.clone()} partner_raw={self.partner_raw.clone()} />),
            ),
            "fest" => Ok(html! {
                <span><Checkbox value={self.fest} object_key={"fest".to_string()} schueler={self.id} /></span>
            }),
            "ignorieren" => Ok(html! {
                <span><Checkbox value={self.ignorieren} object_key={"ignorieren"} schueler={self.id} /></span>
            }),
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
            "klasse" => Ok(serde_value::Value::U32(self.klasse.stufe().unwrap_or(0))),
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
            "fest" => Ok(serde_value::Value::Bool(self.fest)),
            "ignorieren" => Ok(serde_value::Value::Bool(self.ignorieren)),
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
