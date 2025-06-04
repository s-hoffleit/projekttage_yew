use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use gloo_console::log;
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
use crate::types::SaveFileProjekt;
use crate::types::SaveFileSchueler;
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
}

impl Data {
    pub fn get_schueler(&self, schueler_id: &SchuelerId) -> Option<&SaveFileSchueler> {
        self.schueler.get(schueler_id)
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

#[function_component(App)]
fn app() -> Html {
    let route = use_route::<Route>().unwrap_or_default();

    match route {
        Route::Home => log!("Home"),
        Route::Projekte => log!("Projekte"),
        Route::Schueler => log!("Schueler"),
        Route::Einteilung => log!("Einteilung"),
    }

    let ctx = use_state(|| {
        if let Ok(save_file) = SaveFile::load_from_local_storage() {
            Data {
                projekte: save_file.projekte,
                schueler: save_file.schueler,
            }
        } else {
            Data {
                projekte: BTreeMap::new(),
                schueler: BTreeMap::new(),
            }
        }
    });

    html! {
    <ContextProvider<Data> context={(*ctx).clone()}>
        <HashRouter>
            <nav>
                <Link<Route> to={Route::Home} classes={if route == Route::Home {"current"} else { "" }}>{ "Home" }</Link<Route>>
                <Link<Route> to={Route::Projekte} classes={if route == Route::Projekte {"current"} else { "" }}>{ "Projekte" }</Link<Route>>
                <Link<Route> to={Route::Schueler} classes={if route == Route::Schueler {"current"} else { "" }}>{ "Schueler" }</Link<Route>>
                <Link<Route> to={Route::Einteilung} classes={if route == Route::Einteilung {"current"} else { "" }}>{ "Einteilung" }</Link<Route>>
            </nav>
            <Switch<Route> render={switch} />
        </HashRouter>
    </ContextProvider<Data>>
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
