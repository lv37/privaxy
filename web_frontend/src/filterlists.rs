use crate::filters::{AddFilterRequest, Filter, FilterConfiguration, FilterGroup};
use crate::save_button::BASE_BUTTON_CSS;
use crate::{get_api_host, save_button, submit_banner};
use reqwasm::http::Request;
use url::Url;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::InputEvent;
use yew::{html, Component, Context, Html};

pub enum SearchFilterMessage {
    Open,
    Close,
    FilterChanged(String),
    AddFilter(filterlists_api::Filter),
    RemoveFilter(filterlists_api::Filter),
    LoadFilters,
    FiltersLoaded(Vec<filterlists_api::Filter>),
    Error(String),
    NextPage,
    PreviousPage,
    LanguagesLoaded(Vec<filterlists_api::FilterLanguage>),
    LicensesLoaded(Vec<filterlists_api::FilterLicense>),
    TagsLoaded(Vec<filterlists_api::FilterTag>),
}

pub struct SearchFilterList {
    link: yew::html::Scope<Self>,
    is_open: bool,
    filters: Vec<filterlists_api::Filter>,
    filter_query: String,
    loading: bool,
    languages: Vec<filterlists_api::FilterLanguage>,
    licenses: Vec<filterlists_api::FilterLicense>,
    tags: Vec<filterlists_api::FilterTag>,
    current_page: usize,
    results_per_page: usize,
    active_filters: FilterConfiguration,
}

use filterlists_api;

const FILTER_TAG_GROUPS: [&'static str; 4] = ["ads", "privacy", "malware", "social"];

#[derive(Properties, PartialEq)]
pub struct Props {
    pub filter_configuration: FilterConfiguration,
}

impl Component for SearchFilterList {
    type Message = SearchFilterMessage;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            link: _ctx.link().clone(),
            is_open: false,
            filters: Vec::<filterlists_api::Filter>::new(),
            languages: Vec::<filterlists_api::FilterLanguage>::new(),
            licenses: Vec::<filterlists_api::FilterLicense>::new(),
            tags: Vec::<filterlists_api::FilterTag>::new(),
            filter_query: String::new(),
            loading: true,
            current_page: 1,
            results_per_page: 10,
            active_filters: _ctx.props().filter_configuration.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: SearchFilterMessage) -> bool {
        match msg {
            SearchFilterMessage::Open => {
                self.is_open = true;
                self.link.send_message(SearchFilterMessage::LoadFilters);
            }
            SearchFilterMessage::Close => self.is_open = false,
            SearchFilterMessage::FilterChanged(query) => self.filter_query = query,
            SearchFilterMessage::AddFilter(filter) => {
                let parsed_url = match Url::parse(&filter.primary_view_url.clone().unwrap()) {
                    Ok(url) => url,
                    Err(err) => {
                        log::error!("Failed to parse URL: {}", err);
                        return false;
                    }
                };
                let group: FilterGroup = self
                    .tags
                    .clone()
                    .into_iter()
                    .filter(|tag| {
                        filter.tag_ids.contains(&tag.id)
                            && FILTER_TAG_GROUPS.contains(&tag.name.as_str())
                    })
                    .map(|tag| match tag.name.as_str() {
                        "ads" => FilterGroup::Ads,
                        "privacy" => FilterGroup::Privacy,
                        "malware" => FilterGroup::Malware,
                        "social" => FilterGroup::Social,
                        _ => FilterGroup::Regional,
                    })
                    .next()
                    .unwrap_or(FilterGroup::Regional);

                let request_body: AddFilterRequest =
                    AddFilterRequest::new(filter.name.clone(), group, parsed_url);
                let request = Request::post(&format!("http://{}/filters", get_api_host()))
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request_body).unwrap());
                self.active_filters.push(Filter::new(
                    filter.name.clone(),
                    FilterGroup::Malware,
                    "".to_string(),
                ));
                spawn_local(async move {
                    match request.send().await {
                        Ok(response) => {
                            if response.ok() {
                                log::info!("Filter added successfully");
                            } else {
                                log::error!("Failed to add filter: {:?}", response.status());
                            }
                        }
                        Err(err) => {
                            log::error!("Request error: {:?}", err);
                        }
                    }
                })
            }
            SearchFilterMessage::RemoveFilter(filter) => {
                let parsed_url = match Url::parse(&filter.primary_view_url.clone().unwrap()) {
                    Ok(url) => url,
                    Err(err) => {
                        log::error!("Failed to parse URL: {}", err);
                        return false;
                    }
                };
                let request_body: AddFilterRequest =
                    AddFilterRequest::new(filter.name.clone(), FilterGroup::Malware, parsed_url);
                let request = Request::delete(&format!("http://{}/filters", get_api_host()))
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request_body).unwrap());
                spawn_local(async move {
                    match request.send().await {
                        Ok(response) => {
                            if response.ok() {
                                log::info!("Filter removed successfully");
                            } else {
                                log::error!("Failed to remove filter: {:?}", response.status());
                            }
                        }
                        Err(err) => {
                            log::error!("Request error: {:?}", err);
                        }
                    }
                })
            }
            SearchFilterMessage::LoadFilters => {
                if self.loading {
                    let link = self.link.clone();
                    spawn_local(async move {
                        match filterlists_api::get_languages().await {
                            Ok(langs) => {
                                link.send_message(SearchFilterMessage::LanguagesLoaded(langs))
                            }
                            Err(err) => {
                                link.send_message(SearchFilterMessage::Error(err.to_string()))
                            }
                        };
                        match filterlists_api::get_licenses().await {
                            Ok(licenses) => {
                                link.send_message(SearchFilterMessage::LicensesLoaded(licenses))
                            }
                            Err(err) => {
                                link.send_message(SearchFilterMessage::Error(err.to_string()))
                            }
                        };
                        match filterlists_api::get_tags().await {
                            Ok(tags) => link.send_message(SearchFilterMessage::TagsLoaded(tags)),
                            Err(err) => {
                                link.send_message(SearchFilterMessage::Error(err.to_string()))
                            }
                        };
                        match filterlists_api::get_filters().await {
                            Ok(filters) => {
                                link.send_message(SearchFilterMessage::FiltersLoaded(filters))
                            }
                            Err(err) => {
                                link.send_message(SearchFilterMessage::Error(err.to_string()))
                            }
                        };
                    });
                }
            }
            SearchFilterMessage::FiltersLoaded(filters) => {
                log::info!("Filters loaded successfully");
                self.filters = filters.clone();
                self.loading = false;
            }
            SearchFilterMessage::LanguagesLoaded(langs) => {
                log::info!("Languages loaded successfully");
                self.languages = langs.clone();
            }
            SearchFilterMessage::LicensesLoaded(licenses) => {
                log::info!("Licenses loaded successfully");
                self.licenses = licenses.clone();
            }
            SearchFilterMessage::TagsLoaded(tags) => {
                log::info!("Tags loaded successfully");
                self.tags = tags.clone();
            }
            SearchFilterMessage::Error(error) => {
                log::error!("Error loading filters: {}", error);
                self.loading = false;
            }
            SearchFilterMessage::NextPage => {
                if self.current_page
                    < (self.filters.len() as f64 / self.results_per_page as f64).ceil() as usize
                {
                    self.current_page += 1;
                }
            }
            SearchFilterMessage::PreviousPage => {
                if self.current_page > 1 {
                    self.current_page -= 1;
                }
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let save_button_classes = classes!(
            BASE_BUTTON_CSS.clone().to_vec(),
            "focus:ring-green-500",
            "bg-green-600",
            "hover:bg-green-700",
        );
        let search_button_classes = classes!(
            BASE_BUTTON_CSS.clone().to_vec(),
            "bg-blue-600",
            "hover:bg-blue-700",
        );

        let filtered_filters: Vec<&filterlists_api::Filter> = self
            .filters
            .iter()
            .filter(|filter| {
                filter
                    .name
                    .to_lowercase()
                    .contains(&self.filter_query.to_lowercase())
            })
            .collect();
        let total_pages =
            (filtered_filters.len() as f64 / self.results_per_page as f64).ceil() as usize;
        let start_index = (self.current_page - 1) * self.results_per_page;
        let paginated_filters = filtered_filters
            .into_iter()
            .skip(start_index)
            .take(self.results_per_page);
        let cancel_button_classes = classes!(
            BASE_BUTTON_CSS.clone().to_vec(),
            "focus:ring-red-500",
            "bg-red-600",
            "hover:bg-red-700",
        );
        let prev_button_classes = classes!(
            "bg-gray-500",
            "hover:bg-gray-700",
            "text-white",
            "font-bold",
            "py-2",
            "px-4",
            "rounded",
            if self.current_page == 1 {
                "opacity-50 cursor-not-allowed"
            } else {
                ""
            },
        );

        let next_button_classes = classes!(
            "bg-gray-500",
            "hover:bg-gray-700",
            "text-white",
            "font-bold",
            "py-2",
            "px-4",
            "rounded",
            if self.current_page == total_pages {
                "opacity-50 cursor-not-allowed"
            } else {
                ""
            },
        );
        let document = gloo_utils::document();
        if let Some(body) = document.body() {
            body.set_class_name(if self.is_open { "modal-open" } else { "" });
        }

        html! {
            <>
            <button onclick={self.link.callback(|_| SearchFilterMessage::Open)} class={classes!(search_button_classes, "mt-5")}>
                <svg xmlns="http://www.w3.org/2000/svg" class="-ml-0.5 mr-2 h-5 w-5" fill="none" viewBox="0 0 490.4 490.4" stroke="currentColor">
                    <path fill="white" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M484.1,454.796l-110.5-110.6c29.8-36.3,47.6-82.8,47.6-133.4c0-116.3-94.3-210.6-210.6-210.6S0,94.496,0,210.796 s94.3,210.6,210.6,210.6c50.8,0,97.4-18,133.8-48l110.5,110.5c12.9,11.8,25,4.2,29.2,0C492.5,475.596,492.5,463.096,484.1,454.796z M41.1,210.796c0-93.6,75.9-169.5,169.5-169.5s169.6,75.9,169.6,169.5s-75.9,169.5-169.5,169.5S41.1,304.396,41.1,210.796z"/>
                </svg>
            {"Search filterlists.com"}
            </button>
                { if self.is_open {
                    html! {
                        <div class="fixed inset-0 bg-gray-600 bg-opacity-75 flex items-center justify-center z-50">
                            <div class="bg-white p-6 rounded-lg shadow-lg z-60" style="width: 50vw; height: 80vh; overflow: hidden;">
                                <div class="flex flex-col space-y-4" style="height: 100%;">
                                    <input type="text" placeholder="Search by name" class="border border-gray-300 p-2 rounded"
                                        value={self.filter_query.clone()}
                                        oninput={_ctx.link().callback(|e: InputEvent| {
                                            let input = e.target_dyn_into::<HtmlInputElement>().expect("input element");
                                            SearchFilterMessage::FilterChanged(input.value())
                                        })}
                                    />
                                    <div style="flex-grow: 1; overflow: auto;">
                                        <table class="table-fixed bg-white">
                                            <thead>
                                                <tr style="height: 5vh;">
                                                    <th class="py-2" style="width: 5vw;">{"Name"}</th>
                                                    <th class="py-2" style="width: 10vw;">{"Description"}</th>
                                                    <th class="py-2" style="width: 8vw;">{"Language"}</th>
                                                    <th class="py-2" style="width: 8vw;">{"License"}</th>
                                                    <th class="py-2" style="width: 2vw;">{"Select"}</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                { for paginated_filters.map(|filter| self.view_filter_row(filter, _ctx)) }
                                            </tbody>
                                        </table>
                                    </div>
                                    <div class="flex justify-between mt-4">
                                        <button
                                            onclick={self.link.callback(|_| SearchFilterMessage::PreviousPage)}
                                            class={prev_button_classes}
                                            disabled={self.current_page == 1}>
                                            {"Previous"}
                                        </button>
                                        <span>{"Page "} {self.current_page} {" of "} {total_pages}</span>
                                        <button
                                            onclick={self.link.callback(|_| SearchFilterMessage::NextPage)}
                                            class={next_button_classes}
                                            disabled={self.current_page == total_pages}>
                                            {"Next"}
                                        </button>
                                    </div>
                                    <div class="flex space-x-4">
                                        <button onclick={_ctx.link().callback(|_| SearchFilterMessage::Close)} class={cancel_button_classes}>
                                            {"close"}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </>
        }
    }
}

impl SearchFilterList {
    fn view_filter_row(&self, filter: &filterlists_api::Filter, ctx: &Context<Self>) -> Html {
        let filter_clone = filter.clone();
        let cancel_button_class = classes!(
            BASE_BUTTON_CSS.clone().to_vec(),
            "focus:ring-red-500",
            "bg-red-600",
            "hover:bg-red-700",
        );
        let save_button_classes = classes!(
            BASE_BUTTON_CSS.clone().to_vec(),
            "focus:ring-green-500",
            "bg-green-600",
            "hover:bg-green-700",
        );
        let existing_filter = self
            .active_filters
            .clone()
            .into_iter()
            .any(|f| f.title == filter.name);
        html! {
            <tr>
                <td class="border px-4 py-2 overflow-hidden" style="height: 5vh; white-space: normal; text-overflow: ellipsis;">
                    { if let Some(url) = &filter.primary_view_url {
                        html! { <a href={url.clone()} target="_blank" class="text-blue-600 underline"> { &filter.name } </a> }
                    } else {
                        html! { &filter.name }
                    }}
                </td>
                <td class="border px-4 py-2 overflow-auto" style="height: 5vh; max-width: 10vw; white-space: normal;">
                    { &filter.description }
                </td>
                <td class="border px-4 py-2 overflow-hidden" style="height: 5vh; white-space: nowrap; text-overflow: ellipsis;">
                    { self.get_language_name(filter.language_ids.clone()) }
                </td>
                <td class="border px-4 py-2 overflow-hidden" style="height: 5vh; white-space: nowrap; text-overflow: ellipsis;">
                    { self.get_license_name(filter.license_id) }
                </td>
                <td class="border px-4 py-2 text-center overflow-hidden" style="height: 5vh; white-space: nowrap; text-overflow: ellipsis;">
                <button
                onclick={ if existing_filter
                    { ctx.link().callback(move |_| SearchFilterMessage::RemoveFilter(filter_clone.clone())) }
                    else
                    { ctx.link().callback(move |_| SearchFilterMessage::AddFilter(filter_clone.clone())) }
                }
                class={ if existing_filter {cancel_button_class} else {save_button_classes} }
            >
                { if existing_filter { "Remove" } else { "Add" } }
            </button>
                </td>
            </tr>
        }
    }

    fn get_language_name(&self, language_ids: Vec<u64>) -> String {
        self.languages
            .iter()
            .filter(|lang| language_ids.contains(&lang.id))
            .map(|lang| lang.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn get_license_name(&self, license_id: u64) -> String {
        self.licenses
            .iter()
            .filter(|license| license.id == license_id)
            .map(|license| license.name.clone())
            .collect::<Vec<String>>()
            .join(", ")
    }
}