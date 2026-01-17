use gloo_timers::callback::Timeout;
use leptos::*;
use serde::Serialize;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, Url};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Toml,
    Excel,
    Pdf,
}

#[component]
pub fn ExportButton<T>(data: T, filename: String) -> impl IntoView
where
    T: Serialize + Clone + 'static,
{
    let (show_menu, set_show_menu) = create_signal(false);

    let download = move |format: ExportFormat| {
        set_show_menu.set(false);
        match format {
            ExportFormat::Pdf => {
                // Delay print to ensure the menu is closed and hidden from the DOM check
                Timeout::new(100, move || {
                    let _ = window().print();
                })
                .forget();
            }
            _ => {
                let (content, mime, ext) = match format {
                    ExportFormat::Json => (
                        serde_json::to_string_pretty(&data).unwrap_or_default(),
                        "application/json",
                        "json",
                    ),
                    ExportFormat::Csv => {
                        let mut csv = String::new();
                        if let Ok(json_val) = serde_json::to_value(data.clone()) {
                            if let Some(arr) = json_val.as_array() {
                                if let Some(first) = arr.first() {
                                    if let Some(obj) = first.as_object() {
                                        let keys: Vec<_> = obj.keys().collect();
                                        csv.push_str(
                                            &keys
                                                .iter()
                                                .map(|k| k.as_str())
                                                .collect::<Vec<_>>()
                                                .join(","),
                                        );
                                        csv.push('\n');
                                        for item in arr {
                                            if let Some(item_obj) = item.as_object() {
                                                csv.push_str(
                                                    &keys
                                                        .iter()
                                                        .map(|k| {
                                                            item_obj
                                                                .get(*k)
                                                                .unwrap_or(&serde_json::Value::Null)
                                                                .to_string()
                                                                .replace(",", " ")
                                                        })
                                                        .collect::<Vec<_>>()
                                                        .join(","),
                                                );
                                                csv.push('\n');
                                            }
                                        }
                                    }
                                }
                            } else if let Some(obj) = json_val.as_object() {
                                let keys: Vec<_> = obj.keys().collect();
                                csv.push_str(
                                    &keys
                                        .iter()
                                        .map(|k| k.as_str())
                                        .collect::<Vec<_>>()
                                        .join(","),
                                );
                                csv.push('\n');
                                csv.push_str(
                                    &keys
                                        .iter()
                                        .map(|k| obj.get(*k).unwrap().to_string().replace(",", " "))
                                        .collect::<Vec<_>>()
                                        .join(","),
                                );
                            }
                        }
                        (csv, "text/csv", "csv")
                    }
                    ExportFormat::Toml => (
                        toml::to_string(&data).unwrap_or_default(),
                        "text/x-toml",
                        "toml",
                    ),
                    ExportFormat::Excel => {
                        // CSV with BOM for Excel compatibility
                        let mut csv = String::from("\u{FEFF}");
                        if let Ok(json_val) = serde_json::to_value(data.clone()) {
                            if let Some(arr) = json_val.as_array() {
                                if let Some(first) = arr.first() {
                                    if let Some(obj) = first.as_object() {
                                        let keys: Vec<_> = obj.keys().collect();
                                        csv.push_str(
                                            &keys
                                                .iter()
                                                .map(|k| k.as_str())
                                                .collect::<Vec<_>>()
                                                .join(","),
                                        );
                                        csv.push('\n');
                                        for item in arr {
                                            if let Some(item_obj) = item.as_object() {
                                                csv.push_str(
                                                    &keys
                                                        .iter()
                                                        .map(|k| {
                                                            item_obj
                                                                .get(*k)
                                                                .unwrap_or(&serde_json::Value::Null)
                                                                .to_string()
                                                                .replace(",", " ")
                                                        })
                                                        .collect::<Vec<_>>()
                                                        .join(","),
                                                );
                                                csv.push('\n');
                                            }
                                        }
                                    }
                                }
                            } else if let Some(obj) = json_val.as_object() {
                                let keys: Vec<_> = obj.keys().collect();
                                csv.push_str(
                                    &keys
                                        .iter()
                                        .map(|k| k.as_str())
                                        .collect::<Vec<_>>()
                                        .join(","),
                                );
                                csv.push('\n');
                                csv.push_str(
                                    &keys
                                        .iter()
                                        .map(|k| obj.get(*k).unwrap().to_string().replace(",", " "))
                                        .collect::<Vec<_>>()
                                        .join(","),
                                );
                            }
                        }
                        (csv, "text/csv;charset=utf-8", "csv") // Excel opens CSV with BOM perfectly
                    }
                    ExportFormat::Pdf => unreachable!(),
                };

                let mut bag = BlobPropertyBag::new();
                bag.set_type(mime);

                let blob = Blob::new_with_str_sequence_and_options(
                    &js_sys::Array::of1(&content.into()),
                    &bag,
                )
                .unwrap();

                let url = Url::create_object_url_with_blob(&blob).unwrap();
                let document = window().document().unwrap();
                let a = document
                    .create_element("a")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlAnchorElement>()
                    .unwrap();
                a.set_href(&url);
                a.set_download(&format!("{}.{}", filename, ext));
                a.click();
                let _ = Url::revoke_object_url(&url);
            }
        }
    };

    view! {
        <div class="relative inline-block text-left no-print">
            <div>
                <button
                    type="button"
                    on:click=move |_| set_show_menu.update(|v| *v = !*v)
                    class="inline-flex items-center justify-center w-full rounded-xl border border-indigo-200 shadow-sm px-4 py-2 bg-white text-sm font-bold text-indigo-700 hover:bg-indigo-50 focus:outline-none transition-all duration-200"
                >
                    <i class="fas fa-download mr-2"></i>
                    "Export"
                    <i class="fas fa-chevron-down ml-2 text-xs opacity-50"></i>
                </button>
            </div>

            {move || show_menu.get().then(|| {
                let download_pdf = { let d = download.clone(); move |_| d(ExportFormat::Pdf) };
                let download_excel = { let d = download.clone(); move |_| d(ExportFormat::Excel) };
                let download_json = { let d = download.clone(); move |_| d(ExportFormat::Json) };
                let download_toml = { let d = download.clone(); move |_| d(ExportFormat::Toml) };
                let download_csv = { let d = download.clone(); move |_| d(ExportFormat::Csv) };

                view! {
                <div class="origin-top-right absolute right-0 mt-2 w-56 rounded-2xl shadow-2xl bg-white ring-1 ring-black ring-opacity-5 z-50 overflow-hidden divide-y divide-gray-100 animate-in fade-in zoom-in duration-200">
                    <div class="py-1">
                        <button
                            on:click=download_pdf
                            class="group flex items-center w-full px-4 py-3 text-sm text-gray-700 hover:bg-red-50 hover:text-red-700 transition-colors"
                        >
                            <i class="fas fa-file-pdf mr-3 text-red-500 group-hover:scale-110 transition-transform"></i>
                            <div class="flex flex-col items-start">
                                <span class="font-bold">"PDF"</span>
                                <span class="text-xs text-gray-400">"Printable Report"</span>
                            </div>
                        </button>
                        <button
                            on:click=download_excel
                            class="group flex items-center w-full px-4 py-3 text-sm text-gray-700 hover:bg-green-50 hover:text-green-700 transition-colors"
                        >
                            <i class="fas fa-file-excel mr-3 text-green-500 group-hover:scale-110 transition-transform"></i>
                            <div class="flex flex-col items-start">
                                <span class="font-bold">"Excel"</span>
                                <span class="text-xs text-gray-400">"Spreadsheet (CSV/BOM)"</span>
                            </div>
                        </button>
                    </div>
                    <div class="py-1">
                        <button
                            on:click=download_json
                            class="group flex items-center w-full px-4 py-3 text-sm text-gray-700 hover:bg-amber-50 hover:text-amber-700 transition-colors"
                        >
                            <i class="fas fa-file-code mr-3 text-amber-500 group-hover:scale-110 transition-transform"></i>
                             <div class="flex flex-col items-start">
                                <span class="font-bold">"JSON"</span>
                                <span class="text-xs text-gray-400">"Raw Data Object"</span>
                            </div>
                        </button>
                        <button
                            on:click=download_toml
                            class="group flex items-center w-full px-4 py-3 text-sm text-gray-700 hover:bg-teal-50 hover:text-teal-700 transition-colors"
                        >
                            <i class="fas fa-file-invoice mr-3 text-teal-500 group-hover:scale-110 transition-transform"></i>
                            <div class="flex flex-col items-start">
                                <span class="font-bold">"TOML"</span>
                                <span class="text-xs text-gray-400">"Config Format"</span>
                            </div>
                        </button>
                        <button
                            on:click=download_csv
                            class="group flex items-center w-full px-4 py-3 text-sm text-gray-700 hover:bg-blue-50 hover:text-blue-700 transition-colors"
                        >
                            <i class="fas fa-file-csv mr-3 text-blue-500 group-hover:scale-110 transition-transform"></i>
                            <div class="flex flex-col items-start">
                                <span class="font-bold">"CSV"</span>
                                <span class="text-xs text-gray-400">"Comma Separated Values"</span>
                            </div>
                        </button>
                    </div>
                </div>
            }
            })}
        </div>
    }
}
