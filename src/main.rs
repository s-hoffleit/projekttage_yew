use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use gloo_console::log;
use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use gloo_storage::errors::StorageError;
use serde::Deserialize;
use serde::Serialize;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::seiten::Einteilung;
use crate::seiten::Home;
use crate::seiten::Projekte;
use crate::seiten::Schueler;
use crate::types::ProjektId;
use crate::types::SaveFile;
use crate::types::SaveFileKlasse;
use crate::types::SaveFileProjekt;
use crate::types::SaveFileSchueler;
use crate::types::SaveFileStufe;
use crate::types::SaveFileZuordnung;
use crate::types::SchuelerId;

pub mod components;
pub mod seiten;
pub mod solver;
pub mod types;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[not_found]
    #[at("/")]
    Home,
    #[at("/projekte")]
    Projekte,
    #[at("/schueler")]
    Schueler,
    #[at("/einteilung")]
    Einteilung,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projekt {
    name: String,
    stufen: RangeInclusive<u32>,
    teilnehmer: RangeInclusive<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub projekte: BTreeMap<ProjektId, SaveFileProjekt>,
    pub schueler: BTreeMap<SchuelerId, SaveFileSchueler>,
    pub zuordnung: Vec<SaveFileZuordnung>,
    pub klassen: BTreeMap<SaveFileStufe, SaveFileKlasse>,
}

impl Data {
    pub fn get_schueler(&self, schueler_id: &SchuelerId) -> Option<&SaveFileSchueler> {
        self.schueler.get(schueler_id)
    }
    pub fn get_projekt(&self, projekt_id: &ProjektId) -> Option<&SaveFileProjekt> {
        self.projekte.get(projekt_id)
    }

    pub fn save(&self) -> Result<(), StorageError> {
        LocalStorage::set("projekte", self.projekte.clone())?;
        LocalStorage::set("schueler", self.schueler.clone())?;
        LocalStorage::set("zuordnung", self.zuordnung.clone())?;
        LocalStorage::set("klassen", self.klassen.clone())?;

        Ok(())
    }

    pub fn get(&self) -> Data {
        self.clone()
    }
}

#[function_component(Secure)]
fn secure() -> Html {
    let navigator = use_navigator().unwrap();

    let onclick_callback = Callback::from(move |_| navigator.push(&Route::Home));
    html! {
        <div>
            <h1>{ "Secure" }</h1>
            <button onclick={onclick_callback}>{ "Go Home" }</button>
        </div>
    }
}

pub type DataContext = UseStateHandle<Data>;

#[function_component(App)]
fn app() -> Html {
    let route = use_route::<Route>().unwrap_or_default();

    match route {
        Route::Home => log!("Home"),
        Route::Projekte => log!("Projekte"),
        Route::Schueler => log!("Schueler"),
        Route::Einteilung => log!("Einteilung"),
    }

    let data = use_state(|| {
        if let Ok(save_file) = SaveFile::load_from_local_storage() {
            Data {
                projekte: save_file.projekte,
                schueler: save_file.schueler,
                zuordnung: save_file.zuordnung,
                klassen: save_file.klassen,
            }
        } else {
            Data {
                projekte: BTreeMap::new(),
                schueler: BTreeMap::new(),
                zuordnung: Vec::new(),
                klassen: BTreeMap::new(),
            }
        }
    });

    {
        let data = data.clone();
        use_effect_with(data, |data| {
            let _ = data.save();
        })
    }

    html! {
    <ContextProvider<DataContext> context={data}>
        <HashRouter>
            <nav>
                <Link<Route> to={Route::Home} classes={if route == Route::Home {"current"} else { "" }}>{ "Home" }</Link<Route>>
                <Link<Route> to={Route::Projekte} classes={if route == Route::Projekte {"current"} else { "" }}>{ "Projekte" }</Link<Route>>
                <Link<Route> to={Route::Schueler} classes={if route == Route::Schueler {"current"} else { "" }}>{ "Schueler" }</Link<Route>>
                <Link<Route> to={Route::Einteilung} classes={if route == Route::Einteilung {"current"} else { "" }}>{ "Einteilung" }</Link<Route>>
            </nav>
            <Switch<Route> render={switch} />
        </HashRouter>
    </ContextProvider<DataContext>>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Projekte => html! {
            <Projekte />
        },
        Route::Schueler => html! {
            <Schueler />
        },
        Route::Einteilung => html! {
            <Einteilung />
        },
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
    // todo!(
    //     "Volle Implementation der Klassen, Projekte, Schueler und Zuordnungen mit jeweiliger Seite"
    // );
}
