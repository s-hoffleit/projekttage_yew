use std::fmt::Debug;

use serde::Serialize;
use web_sys::HtmlInputElement;
use yew::{
    Callback, Html, InputEvent, Properties, TargetCast, classes, function_component, html,
    use_state,
};
use yew_custom_components::table::{
    Options, Table,
    types::{Column, TableData},
};

#[derive(Properties, PartialEq)]
pub struct TableProps<TableLine: Clone + PartialEq + 'static> {
    pub columns: Vec<Column>,
    pub table_data: Vec<TableLine>,
}

#[function_component(Tabelle)]
pub fn tabelle<T>(props: &TableProps<T>) -> Html
where
    T: Clone + Serialize + Debug + Default + PartialEq + PartialOrd + TableData,
{
    let search_term = use_state(|| Some::<String>("".to_string()));

    let search = (*search_term).as_ref().cloned();

    // Handle search input
    let oninput_search = {
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if input.value().is_empty() {
                search_term.set(None);
            } else {
                search_term.set(Some(input.value()));
            }
        })
    };

    let options = Options {
        unordered_class: Some("table-sort".to_string()),
        ascending_class: Some("table-sort-up".to_string()),
        descending_class: Some("table-sort-down".to_string()),
        orderable_classes: vec!["table-sortable".to_string()],
    };

    // let pagination_options = yew_custom_components::pagination::Options::default()
    //     .show_prev_next(true)
    //     .show_first_last(true)
    //     .list_classes(vec![String::from("pagination")])
    //     .item_classes(vec![String::from("page-item")])
    //     .link_classes(vec![String::from("page-link")])
    //     .active_item_classes(vec![String::from("active")])
    //     .disabled_item_classes(vec![String::from("disabled")]);

    // let page = use_state(|| 0usize);
    // let current_page = (*page).clone();

    // let handle_page = {
    //     let page = page.clone();
    //     Callback::from(move |id: usize| {
    //         page.set(id);
    //     })
    // };

    html! {
        <>
        // <div class="flex-grow-1 p-2 input-group mb-2">
        //       <span class="input-group-text">{"id"}</span>
        //       <input type="text" oninput={oninput_id} class="form-control" value={format!("{}", id)}/>
        //       <span class="input-group-text">{"name"}</span>
        //       <input type="text" oninput={oninput_name} class="form-control" value={name.unwrap_or_default()} />
        //       <span class="input-group-text">{"value"}</span>
        //       <input type="text" oninput={oninput_value} class="form-control" value={format!("{}", value)}/>
        //       <button type="button" {onclick} class="btn btn-primary">{"Add"}</button>
        //     </div>
            <div class="flex-grow-1 p-2 input-group mb-2">
                <span class="input-group-text">
                    <i class="fas fa-search"></i>
                </span>
                <input class="form-control" type="text" id="search" placeholder="Search" oninput={oninput_search} />
            </div>
            <Table<T> options={options.clone()} search={search.clone()} classes={classes!("table", "table-hover")} columns={props.columns.clone()} data={props.table_data.clone()} orderable={true} />
            // <Pagination total={table_data.len()} limit={15} max_pages={5} options={pagination_options} on_page={Some(handle_page)}/>
        </>
    }
}
