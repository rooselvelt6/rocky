use crate::frontend::i18n::{t, use_i18n};
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

    // ... (logic remains same)
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
        <div class="min-h-screen bg-[#0f172a] text-gray-100 p-8 font-sans relative overflow-hidden">
            // Decorative background elements
            <div class="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-indigo-600/10 rounded-full blur-[120px]"></div>
            <div class="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-blue-600/10 rounded-full blur-[120px]"></div>

            // Header
            <div class="relative z-10 flex flex-col md:flex-row justify-between items-center mb-12 gap-6 bg-white/5 backdrop-blur-md p-6 rounded-3xl border border-white/10 shadow-2xl">
                <div class="flex items-center gap-6">
                   <div class="relative">
                       <div class="w-4 h-4 bg-emerald-500 rounded-full animate-pulse shadow-[0_0_20px_#10b981]"></div>
                       <div class="absolute inset-0 bg-emerald-500 rounded-full animate-ping opacity-20"></div>
                   </div>
                   <div>
                       <h1 class="text-3xl font-black tracking-tight text-white uppercase bg-gradient-to-r from-white to-gray-400 bg-clip-text text-transparent">
                           {move || t(lang.get(), "uci_monitor_title")}
                       </h1>
                       <div class="text-xs text-indigo-400 font-bold tracking-[0.2em]">{move || t(lang.get(), "updated_today")}</div>
                   </div>
                </div>

                <div class="flex items-center gap-8">
                    <div class="text-center">
                        <div class="text-[10px] text-slate-500 uppercase font-black tracking-widest mb-1">{move || t(lang.get(), "patients_active")}</div>
                        <div class="text-3xl font-black text-white font-mono">
                           {move || format!("{:02}", patients.get().len())}
                        </div>
                    </div>
                    <div class="h-10 w-[1px] bg-white/10"></div>
                    <div class="bg-indigo-600/20 px-4 py-2 rounded-xl border border-indigo-500/30">
                        <i class="fas fa-clock text-indigo-400 mr-2"></i>
                        <span class="font-mono text-sm font-bold text-indigo-200">"08:45:12 AM"</span>
                    </div>
                </div>
            </div>

            // Grid
            <div class="relative z-10 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-8">
                <For
                    each=move || patients.get()
                    key=|p| p.id.clone()
                    children=move |patient| {
                        let id_str = patient.id.as_ref().map(|id| id.to_string()).unwrap_or_default();
                        // Deterministic but dynamic-looking "status" for demo
                        let is_critical = id_str.len() % 3 == 0;
                        let is_warning = id_str.len() % 3 == 1;

                        let status_class = if is_critical {
                            "border-rose-500/50 from-rose-500/10 to-transparent shadow-[0_0_40px_rgba(244,63,94,0.15)]"
                        } else if is_warning {
                            "border-amber-500/50 from-amber-500/10 to-transparent shadow-[0_0_40px_rgba(245,158,11,0.15)]"
                        } else {
                            "border-emerald-500/50 from-emerald-500/10 to-transparent shadow-[0_0_40px_rgba(16,185,129,0.15)]"
                        };

                        let status_tag = if is_critical { "CRITICAL" } else if is_warning { "WARNING" } else { "STABLE" };
                        let tag_color = if is_critical { "bg-rose-500" } else if is_warning { "bg-amber-500" } else { "bg-emerald-500" };

                        view! {
                            <div class=format!("group relative bg-white/5 backdrop-blur-xl rounded-[2rem] border-t-2 p-1 transition-all duration-500 hover:scale-[1.02] hover:bg-white/10 {}", status_class)>
                                <div class="p-6">
                                    <div class="flex justify-between items-start mb-6">
                                        <div class="flex-1 min-w-0">
                                            <div class=format!("inline-block px-2 py-0.5 rounded text-[9px] font-black text-white mb-2 {}", tag_color)>
                                                {status_tag}
                                            </div>
                                            <h3 class="text-xl font-bold text-white truncate group-hover:text-indigo-300 transition-colors">
                                                {format!("{} {}", patient.first_name, patient.last_name)}
                                            </h3>
                                            <div class="font-mono text-[10px] text-slate-500 mt-1 flex items-center gap-2">
                                                <i class="fas fa-fingerprint opacity-50"></i>
                                                {format!("ID: {}", id_str.split(':').last().unwrap_or("?"))}
                                            </div>
                                        </div>
                                        <div class="flex flex-col items-center gap-1">
                                            <div class="w-10 h-10 rounded-2xl bg-white/5 flex items-center justify-center border border-white/10 group-hover:border-indigo-500/50 transition-colors">
                                                <i class=format!("fas fa-bed text-sm {}", if is_critical { "text-rose-400" } else { "text-slate-400" })></i>
                                            </div>
                                            <span class="text-[10px] font-bold text-slate-600">"B-04"</span>
                                        </div>
                                    </div>

                                    // Vitals Visualization
                                    <div class="grid grid-cols-2 gap-3 mb-6">
                                        <div class="bg-black/20 p-4 rounded-2xl border border-white/5 group-hover:border-white/10 transition-colors relative overflow-hidden">
                                            <div class="text-[10px] text-slate-500 uppercase font-black mb-1">{move || t(lang.get(), "sofa_score")}</div>
                                            <div class=format!("text-3xl font-black font-mono {}", if is_critical { "text-rose-400" } else if is_warning { "text-amber-400" } else { "text-white" })>
                                                {if is_critical { "14" } else if is_warning { "7" } else { "2" }}
                                            </div>
                                            // Mini sparkline decoration
                                            <div class="absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-transparent via-indigo-500/20 to-transparent"></div>
                                        </div>
                                        <div class="bg-black/20 p-4 rounded-2xl border border-white/5 group-hover:border-white/10 transition-colors">
                                            <div class="text-[10px] text-slate-500 uppercase font-black mb-1">{move || t(lang.get(), "glasgow_scale")}</div>
                                            <div class="text-3xl font-black font-mono text-white">
                                                {if is_critical { "07" } else if is_warning { "11" } else { "15" }}
                                            </div>
                                        </div>
                                    </div>

                                    // NEWS2 Quick Indicator
                                    <div class="flex items-center gap-3 mb-6 bg-white/5 p-3 rounded-2xl border border-white/5">
                                        <div class="w-8 h-8 rounded-full bg-indigo-500/20 flex items-center justify-center text-indigo-400">
                                            <i class="fas fa-stethoscope text-xs"></i>
                                        </div>
                                        <div class="flex-1">
                                            <div class="text-[9px] text-slate-500 font-bold uppercase tracking-wider">"NEWS2 Deterioration"</div>
                                            <div class="w-full bg-white/10 h-1 rounded-full mt-1 overflow-hidden">
                                                <div class=format!("h-full rounded-full {}", if is_critical { "w-[85%] bg-rose-500 shadow-[0_0_10px_#f43f5e]" } else if is_warning { "w-[45%] bg-amber-500" } else { "w-[15%] bg-emerald-500" })></div>
                                            </div>
                                        </div>
                                    </div>

                                    // Actions
                                    <div class="flex items-center justify-between">
                                        <div class="text-[10px] items-center gap-2 text-slate-400 font-mono hidden group-hover:flex animate-fade-in">
                                            <span class="relative flex h-2 w-2">
                                              <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-indigo-400 opacity-75"></span>
                                              <span class="relative inline-flex rounded-full h-2 w-2 bg-indigo-500"></span>
                                            </span>
                                            "REAL-TIME"
                                        </div>
                                        <div class="flex-1"></div>
                                        <a href=format!("/patients/{}", id_str) class="group/btn relative px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-[10px] font-black uppercase tracking-widest rounded-xl transition-all shadow-lg shadow-indigo-900/40 hover:shadow-indigo-500/40 flex items-center gap-2 overflow-hidden">
                                            <span class="relative z-10">{move || t(lang.get(), "view_details")}</span>
                                            <i class="fas fa-chevron-right text-[8px] relative z-10 group-hover/btn:translate-x-1 transition-transform"></i>
                                            <div class="absolute inset-0 bg-gradient-to-r from-white/0 via-white/20 to-white/0 translate-x-[-100%] group-hover/btn:translate-x-[100%] transition-transform duration-700"></div>
                                        </a>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
