use crate::frontend::i18n::use_i18n;
use crate::models::patient::Patient;
use leptos::*;
use std::time::Duration;

#[component]
pub fn WardView() -> impl IntoView {
    let lang = use_i18n();
    // Use the same patient fetching logic as PatientList for now
    // In a real app, this might use a more optimized endpoint
    // We'll simulate auto-refresh by re-fetching

    let (patients, set_patients) = create_signal(Vec::<Patient>::new());

    // Simulate fetching data (placeholder for actual API call reuse)
    // For now we will rely on a resource or just fetch on mount + interval
    let fetch_patients = move || {
        spawn_local(async move {
            let token: Option<String> = window()
                .local_storage()
                .ok()
                .flatten()
                .and_then(|s| s.get_item("uci_token").ok().flatten());

            let mut req = reqwasm::http::Request::get("/api/patients");
            if let Some(t) = token {
                req = req.header("Authorization", &format!("Bearer {}", t));
            }

            let res = req.send().await;
            if let Ok(resp) = res {
                if let Ok(list) = resp.json::<Vec<Patient>>().await {
                    set_patients.set(list);
                }
            }
        });
    };

    // Initial fetch
    create_effect(move |_| {
        fetch_patients();
    });

    // Auto-refresh every 30 seconds
    set_interval(
        move || {
            fetch_patients();
        },
        Duration::from_secs(30),
    );

    view! {
        <div class="min-h-screen bg-slate-900 text-gray-100 p-6 font-sans">
            // Header
            <div class="flex justify-between items-center mb-8 border-b border-slate-700 pb-4">
                <div class="flex items-center gap-4">
                   <div class="w-3 h-3 bg-green-500 rounded-full animate-pulse shadow-[0_0_10px_#22c55e]"></div>
                   <h1 class="text-3xl font-bold tracking-tight text-white uppercase">
                       "UCI Central Monitor"
                   </h1>
                </div>
                <div class="text-slate-400 font-mono text-sm">
                   {move || {
                       let count = patients.get().len();
                       format!("PATIENTS ACTIVE: {:02}", count)
                   }}
                </div>
            </div>

            // Grid
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                <For
                    each=move || patients.get()
                    key=|p| p.id.clone()
                    children=move |patient| {
                        // Mock data for demo purposes (since we don't have real live scores in basic patient struct yet)
                        // In real impl, this would come from the API
                        let id_str = patient.id.as_ref().map(|id| id.to_string()).unwrap_or_default();
                        let is_critical = id_str.len() % 2 == 0; // Randomly assign critical status for visual demo

                        let card_border = if is_critical { "border-red-500 shadow-[0_0_20px_rgba(239,68,68,0.3)]" } else { "border-slate-700 hover:border-slate-500" };
                        let status_color = if is_critical { "text-red-400" } else { "text-green-400" };
                        let pulse_class = if is_critical { "animate-pulse" } else { "" };

                        view! {
                            <div class=format!("bg-slate-800 rounded-xl border-l-4 p-5 transition-all duration-500 {}", card_border)>
                                <div class="flex justify-between items-start mb-4">
                                    <div>
                                        <h3 class="text-xl font-bold text-white truncate w-48">
                                            {format!("{}, {}", patient.last_name, patient.first_name)}
                                        </h3>
                                        <div class="font-mono text-xs text-slate-400 mt-1">
                                            {format!("ID: {}", id_str.split(':').last().unwrap_or("?"))}
                                        </div>
                                    </div>
                                    <div class=format!("text-2xl {}", pulse_class)>
                                        {if is_critical { "🚨" } else { "✅" }}
                                    </div>
                                </div>

                                // Vitals / Scores Grid
                                <div class="grid grid-cols-2 gap-4 mb-4">
                                    <div class="bg-slate-700/50 p-3 rounded-lg">
                                        <div class="text-xs text-slate-400 uppercase">"SOFA"</div>
                                        <div class=format!("text-2xl font-mono font-bold {}", if is_critical { "text-red-400" } else { "text-white" })>
                                            {if is_critical { "12" } else { "4" }}
                                        </div>
                                    </div>
                                    <div class="bg-slate-700/50 p-3 rounded-lg">
                                        <div class="text-xs text-slate-400 uppercase">"GCS"</div>
                                        <div class="text-2xl font-mono font-bold text-white">
                                            {if is_critical { "9" } else { "14" }}
                                        </div>
                                    </div>
                                </div>

                                // Next Assessment Timer
                                <div class="flex items-center justify-between text-xs font-mono bg-slate-900/50 p-2 rounded">
                                    <span class="text-slate-500">"NEXT CHECK:"</span>
                                    <span class=format!("font-bold {}", if is_critical { "text-red-400" } else { "text-yellow-400" })>
                                        {if is_critical { "-00:15:00" } else { "03:45:00" }}
                                    </span>
                                </div>

                                <div class="mt-4 pt-3 border-t border-slate-700/50 flex justify-end">
                                    <a href=format!("/patients/{}", id_str) class="text-xs text-cyan-400 hover:text-cyan-300 font-bold uppercase tracking-wider">
                                        "View Details >"
                                    </a>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
