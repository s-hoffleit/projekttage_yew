use std::collections::{BTreeMap, HashMap};

use gloo::utils::document;
use gloo_console::log;
use gloo_file::{Blob, ObjectUrl, callbacks::FileReader};
use web_sys::{FileList, HtmlInputElement, wasm_bindgen::JsCast};
use yew::{Component, Context, ContextHandle, Event, Html, TargetCast, html};

use crate::{
    DataContext,
    types::{SaveFile, SaveFileSchueler, SchuelerId, schueler_file, schueler_liste_file},
};

#[derive(Clone, Copy)]
pub enum FileType {
    Full,
    Projekte,
    Schueler,
    SchuelerListe,
}

pub enum Msg {
    FileLoaded(String, String),
    ProjekteLoaded(String, String),
    SchuelerLoaded(String, String),
    SchuelerListeLoaded(String, String),
    FileLoad(Option<FileList>, FileType),
    DataUpdate(DataContext),
    SaveFile,
    ExportCsv,
}

pub struct Home {
    readers: HashMap<String, FileReader>,
    data: DataContext,
    _context_listener: ContextHandle<DataContext>,
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (data, context_listener) = ctx
            .link()
            .context::<DataContext>(ctx.link().callback(Msg::DataUpdate))
            .expect("Kein Datenkontext");

        Self {
            data,
            _context_listener: context_listener,
            readers: HashMap::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileLoaded(name, text) => {
                self.readers.remove(&name);
                log!(text.clone());

                let save_file_data: Result<SaveFile, serde_json::Error> =
                    serde_json::from_str(&text.clone());

                match save_file_data {
                    Ok(save_file) => {
                        save_file.log();
                        let _ = save_file.save_to_local_storage();
                        self.data.set(save_file.into());
                    }
                    Err(err) => log!(err.to_string()),
                }

                true
            }
            Msg::FileLoad(files, file_type) => {
                log!("LOAD!");
                for file in gloo::file::FileList::from(files.expect("files")).iter() {
                    let link = ctx.link().clone();
                    let name = file.name().clone();
                    let _file_type = file.raw_mime_type();

                    let task = {
                        gloo::file::callbacks::read_as_text(file, move |text| match file_type {
                            FileType::Full => link.send_message(Msg::FileLoaded(
                                name,
                                text.expect("Failed to read file"),
                            )),
                            FileType::Projekte => link.send_message(Msg::ProjekteLoaded(
                                name,
                                text.expect("Failed to read file"),
                            )),
                            FileType::Schueler => link.send_message(Msg::SchuelerLoaded(
                                name,
                                text.expect("Failed to read file"),
                            )),
                            FileType::SchuelerListe => link.send_message(Msg::SchuelerListeLoaded(
                                name,
                                text.expect("Failed to read file"),
                            )),
                        })
                    };
                    self.readers.insert(file.name(), task);
                }
                true
            }
            Msg::ProjekteLoaded(name, text) => {
                self.readers.remove(&name);
                log!(text.clone());

                let save_file_data: Result<SaveFile, serde_json::Error> =
                    serde_json::from_str(&text.clone());

                match save_file_data {
                    Ok(save_file) => {
                        save_file.log();
                        let _ = save_file.save_to_local_storage();
                        self.data.set(save_file.into());
                    }
                    Err(err) => log!(err.to_string()),
                }

                true
            }
            Msg::SchuelerLoaded(name, text) => {
                self.readers.remove(&name);
                log!(text.clone());

                let save_file_data: Result<schueler_file::SchuelerFile, serde_json::Error> =
                    serde_json::from_str(&text.clone());

                match save_file_data {
                    Ok(schueler_file) => {
                        // let _ = schueler_file.save_to_local_storage();

                        let mut data = self.data.get();

                        let mut schueler = data.schueler;

                        let schueler_wuensche: BTreeMap<SchuelerId, SaveFileSchueler> =
                            schueler_file.into();

                        for (schueler_id, schueler_data) in schueler_wuensche {
                            schueler.insert(schueler_id, schueler_data);
                        }

                        data.schueler = schueler;

                        self.data.set(data);
                    }
                    Err(err) => log!(err.to_string()),
                }

                true
            }
            Msg::SchuelerListeLoaded(name, text) => {
                self.readers.remove(&name);
                log!(text.clone());

                let save_file_data: Result<
                    schueler_liste_file::SchuelerListeFile,
                    serde_json::Error,
                > = serde_json::from_str(&text.clone());

                match save_file_data {
                    Ok(schueler_file) => {
                        // let _ = schueler_file.save_to_local_storage();

                        let mut data = self.data.get();

                        data.schueler = schueler_file.into();

                        self.data.set(data);
                    }
                    Err(err) => log!(err.to_string()),
                }

                true
            }
            Msg::DataUpdate(data) => {
                self.data = data;
                let _ = self.data.save();

                true
            }
            Msg::SaveFile => {
                let data = self.data.get();

                let json_string =
                    serde_json::to_string_pretty(&data).expect("Fehler beim Serialisieren");

                let blob = Blob::new_with_options(json_string.as_str(), Some("application/json"));

                let url = ObjectUrl::from(blob);

                let document = document();
                let a = document
                    .create_element("a")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlAnchorElement>()
                    .unwrap();

                a.set_href(&url);
                a.set_download("projekttage.json");
                a.click();

                false
            }
            Msg::ExportCsv => {
                let data = self.data.get();
                let zuordnungen = data.clone().zuordnung;

                let mut csv_string = "ID;Schueler;Projekt".to_string();

                for zuordnung in zuordnungen {
                    let schueler = data.get_schueler(&zuordnung.schueler).unwrap();

                    let projekt = data.get_projekt(&zuordnung.projekt.unwrap()).unwrap();

                    csv_string += format!(
                        "\n{};{} ({});{}: {} ({}-{})",
                        zuordnung.id,
                        schueler.name,
                        schueler.klasse.klasse(),
                        zuordnung
                            .projekt
                            .map(|p_id| p_id.id().to_string())
                            .unwrap_or("--".to_string()),
                        projekt.name,
                        projekt.min_stufe,
                        projekt.max_stufe
                    )
                    .as_str();
                }

                let blob = Blob::new_with_options(csv_string.as_str(), Some("application/csv"));

                let url = ObjectUrl::from(blob);

                let document = document();
                let a = document
                    .create_element("a")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlAnchorElement>()
                    .unwrap();

                a.set_href(&url);
                a.set_download("projekttage.csv");
                a.click();

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <div>
                    <label for="file-upload">{"Speicherstand hochladen"}</label>
                    <input
                            id="file-upload"
                            type="file"
                            accept="*.json"
                            multiple={false}
                            onchange={ctx.link().callback(move |e: Event| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::FileLoad(input.files(), FileType::Full)
                            })}
                        />
                    <button onclick={ctx.link().callback(move |_| Msg::SaveFile)}>{"Speichern"}</button>
                </div>
                <div>
                    <label for="projekte-upload">{"Projekte hochladen"}</label>
                    <input
                            id="projekte-upload"
                            type="file"
                            accept="*.json"
                            multiple={false}
                            onchange={ctx.link().callback(move |e: Event| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::FileLoad(input.files(), FileType::Projekte)
                            })}
                        />
                </div>
                <div>
                    <label for="schuelerliste-upload">{"Schülerliste hochladen"}</label>
                    <input
                            id="schuelerliste-upload"
                            type="file"
                            accept="*.json"
                            multiple={false}
                            onchange={ctx.link().callback(move |e: Event| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::FileLoad(input.files(), FileType::SchuelerListe)
                            })}
                        />
                    <label for="schueler-upload">{"Schülerwahl hochladen"}</label>
                    <input
                            id="schueler-upload"
                            type="file"
                            accept="*.json"
                            multiple={false}
                            onchange={ctx.link().callback(move |e: Event| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::FileLoad(input.files(), FileType::Schueler)
                            })}
                        />
                </div>
                <div>
                    <button onclick={ctx.link().callback(move |_| Msg::ExportCsv)}>{"Einteilung als CSV exportieren"}</button>
                </div>
            </div>
        }
    }
}
