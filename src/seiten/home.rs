use std::collections::HashMap;

use gloo_console::log;
use gloo_file::callbacks::FileReader;
use web_sys::{FileList, HtmlInputElement};
use yew::{Component, Context, Event, Html, TargetCast, html};

use crate::types::SaveFile;

pub enum Msg {
    FileLoaded((String, String)),
    FileLoad(Option<FileList>),
}

pub struct Home {
    readers: HashMap<String, FileReader>,
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileLoaded((name, text)) => {
                self.readers.remove(&name);
                log!(text.clone());

                let save_file_data: Result<SaveFile, serde_json::Error> =
                    serde_json::from_str(&text.clone());

                match save_file_data {
                    Ok(data) => {
                        data.log();
                        let _ = data.save_to_local_storage();
                    }
                    Err(err) => log!(err.to_string()),
                }

                true
            }
            Msg::FileLoad(files) => {
                log!("LOAD!");
                for file in gloo::file::FileList::from(files.expect("files")).iter() {
                    let link = ctx.link().clone();
                    let name = file.name().clone();
                    let _file_type = file.raw_mime_type();

                    let task = {
                        gloo::file::callbacks::read_as_text(file, move |text| {
                            link.send_message(Msg::FileLoaded((
                                name,
                                text.expect("Failed to read file"),
                            )))
                        })
                    };
                    self.readers.insert(file.name(), task);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <input
                        id="file-upload"
                        type="file"
                        accept="*.json"
                        multiple={true}
                        onchange={ctx.link().callback(move |e: Event| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::FileLoad(input.files())
                        })}
                    />
            </div>
        }
    }
}
