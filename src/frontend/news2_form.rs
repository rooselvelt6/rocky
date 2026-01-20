use crate::frontend::i18n::{t, use_i18n};
use crate::models::news2::{ConsciousnessLevel, News2Assessment, News2RiskLevel};
use leptos::*;
use leptos_router::{use_navigate, use_params_map};

#[component]
pub fn News2Form() -> impl IntoView {
    let lang = use_i18n();
    let params = use_params_map();
    let patient_id = move || params.get().get("id").cloned();

    // Physiological Parameters Signals
    let (respiration_rate, set_respiration_rate) = create_signal(16u8);
    let (spo2_scale, set_spo2_scale) = create_signal(1u8);
    let (spo2, set_spo2) = create_signal(98u8);
    let (air_or_oxygen, set_air_or_oxygen) = create_signal(false);
    let (systolic_bp, set_systolic_bp) = create_signal(120u16);
    let (heart_rate, set_heart_rate) = create_signal(75u16);
    let (consciousness, set_consciousness) = create_signal(ConsciousnessLevel::Alert);
    let (temperature, set_temperature) = create_signal(36.6f32);

    // Derived signal for real-time score
    let assessment = create_memo(move |_| {
        let mut a = News2Assessment {
            id: None,
            patient_id: patient_id().unwrap_or_default(),
            assessed_at: "".to_string(),
            respiration_rate: respiration_rate.get(),
            spo2_scale: spo2_scale.get(),
            spo2: spo2.get(),
            air_or_oxygen: air_or_oxygen.get(),
            systolic_bp: systolic_bp.get(),
            heart_rate: heart_rate.get(),
            consciousness: consciousness.get(),
            temperature: temperature.get(),
            score: 0,
            risk_level: News2RiskLevel::Low,
        };
        a.calculate_score();
        a
    });

    let risk_color = move || match assessment.get().risk_level {
        News2RiskLevel::Low => "bg-green-100 text-green-800 border-green-200",
        News2RiskLevel::LowMedium => "bg-yellow-100 text-yellow-800 border-yellow-200",
        News2RiskLevel::Medium => "bg-orange-100 text-orange-800 border-orange-200",
        News2RiskLevel::High => "bg-red-100 text-red-800 border-red-200",
    };

    let on_save = move |_| {
        let assessment_data = assessment.get();
        let navigate = use_navigate();

        spawn_local(async move {
            let token: Option<String> = window()
                .local_storage()
                .ok()
                .flatten()
                .and_then(|s| s.get_item("uci_token").ok().flatten());

            let mut req = reqwasm::http::Request::post("/api/news2")
                .header("Content-Type", "application/json");

            if let Some(t) = token {
                req = req.header("Authorization", &format!("Bearer {}", t));
            }

            let res = req
                .body(serde_json::to_string(&assessment_data).unwrap())
                .send()
                .await;

            if res.is_ok() {
                let pid = assessment_data.patient_id.clone();
                navigate(&format!("/patients/{}", pid), Default::default());
            }
        });
    };

    view! {
        <div class="w-full max-w-4xl mx-auto px-4 pb-12">
            <div class="text-center mb-8">
                <h2 class="text-3xl font-bold text-indigo-900 flex items-center justify-center gap-3">
                    <i class="fas fa-file-medical-alt text-indigo-600"></i>
                    {move || t(lang.get(), "news2_title")}
                </h2>
                <p class="text-indigo-600 mt-1 uppercase text-sm font-semibold tracking-wider">
                    {move || t(lang.get(), "news2_subtitle")}
                </p>
            </div>

            // Real-time Score Display
            <div class=move || format!("mb-8 p-6 rounded-2xl border-2 transition-all duration-500 shadow-lg flex flex-col md:flex-row items-center justify-between gap-6 {}", risk_color())>
                <div class="text-center md:text-left">
                    <div class="text-xs uppercase font-bold opacity-70 mb-1">{move || t(lang.get(), "score")}</div>
                    <div class="text-6xl font-black">{move || assessment.get().score}</div>
                </div>

                <div class="flex-1 text-center md:text-left">
                    <div class="text-xs uppercase font-bold opacity-70 mb-1">{move || t(lang.get(), "news2_risk")}</div>
                    <div class="text-2xl font-bold">
                        {move || match assessment.get().risk_level {
                            News2RiskLevel::Low => t(lang.get(), "news2_low"),
                            News2RiskLevel::LowMedium => format!("{} (Red Score)", t(lang.get(), "news2_medium")),
                            News2RiskLevel::Medium => t(lang.get(), "news2_medium"),
                            News2RiskLevel::High => t(lang.get(), "news2_high"),
                        }}
                    </div>
                </div>

                <div class="w-full md:w-auto">
                    <button
                        on:click=on_save
                        class="w-full px-6 py-3 bg-white/50 hover:bg-white text-indigo-900 font-bold rounded-xl transition-all shadow-sm flex items-center justify-center gap-2"
                    >
                        <i class="fas fa-save"></i>
                        {move || t(lang.get(), "save_changes")}
                    </button>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                // Respiration Rate
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-indigo-50">
                    <label class="flex justify-between items-center mb-4">
                        <span class="font-bold text-gray-700">{move || t(lang.get(), "respiratory_rate")}</span>
                        <span class="text-2xl font-black text-indigo-600">{move || respiration_rate.get()}</span>
                    </label>
                    <input type="range" min="5" max="40"
                        prop:value=move || respiration_rate.get()
                        on:input=move |ev| set_respiration_rate.set(event_target_value(&ev).parse().unwrap_or(16))
                        class="w-full h-2 bg-indigo-100 rounded-lg appearance-none cursor-pointer accent-indigo-600" />
                </div>

                // SpO2
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-indigo-50">
                    <div class="flex justify-between items-center mb-2">
                        <span class="font-bold text-gray-700">"SpO₂ (%)"</span>
                        <span class="text-2xl font-black text-blue-600">{move || spo2.get()} "%"</span>
                    </div>
                    <div class="flex gap-2 mb-4">
                        <button
                            on:click=move |_| set_spo2_scale.set(1)
                            class=move || format!("flex-1 py-1 text-xs font-bold rounded-lg border {}", if spo2_scale.get() == 1 { "bg-blue-600 text-white border-blue-600" } else { "bg-white text-gray-500 border-gray-200" })
                        >
                            "Scale 1"
                        </button>
                        <button
                            on:click=move |_| set_spo2_scale.set(2)
                            class=move || format!("flex-1 py-1 text-xs font-bold rounded-lg border {}", if spo2_scale.get() == 2 { "bg-blue-600 text-white border-blue-600" } else { "bg-white text-gray-500 border-gray-200" })
                        >
                            "Scale 2 (COPD)"
                        </button>
                    </div>
                    <input type="range" min="70" max="100"
                        prop:value=move || spo2.get()
                        on:input=move |ev| set_spo2.set(event_target_value(&ev).parse().unwrap_or(98))
                        class="w-full h-2 bg-blue-100 rounded-lg appearance-none cursor-pointer accent-blue-600" />
                </div>

                // Air or Oxygen
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-indigo-50">
                    <span class="font-bold text-gray-700 block mb-4">"Air or Supplemental Oxygen"</span>
                    <div class="flex gap-2">
                        <button
                            on:click=move |_| set_air_or_oxygen.set(false)
                            class=move || format!("flex-1 py-3 rounded-xl font-bold flex items-center justify-center gap-2 border transition-all {}", if !air_or_oxygen.get() { "bg-indigo-600 text-white border-indigo-600 shadow-md" } else { "bg-gray-50 text-gray-400 border-gray-200" })
                        >
                            <i class="fas fa-wind"></i> "Air"
                        </button>
                        <button
                            on:click=move |_| set_air_or_oxygen.set(true)
                            class=move || format!("flex-1 py-3 rounded-xl font-bold flex items-center justify-center gap-2 border transition-all {}", if air_or_oxygen.get() { "bg-orange-500 text-white border-orange-500 shadow-md" } else { "bg-gray-50 text-gray-400 border-gray-200" })
                        >
                            <i class="fas fa-burn"></i> "Oxygen"
                        </button>
                    </div>
                </div>

                // Systolic BP
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-indigo-50">
                    <label class="flex justify-between items-center mb-4">
                        <span class="font-bold text-gray-700">{move || t(lang.get(), "systolic_bp")}</span>
                        <span class="text-2xl font-black text-red-600">{move || systolic_bp.get()}</span>
                    </label>
                    <input type="range" min="40" max="250"
                        prop:value=move || systolic_bp.get()
                        on:input=move |ev| set_systolic_bp.set(event_target_value(&ev).parse().unwrap_or(120))
                        class="w-full h-2 bg-red-100 rounded-lg appearance-none cursor-pointer accent-red-600" />
                </div>

                // Heart Rate
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-indigo-50">
                    <label class="flex justify-between items-center mb-4">
                        <span class="font-bold text-gray-700">{move || t(lang.get(), "heart_rate")}</span>
                        <span class="text-2xl font-black text-rose-600">{move || heart_rate.get()}</span>
                    </label>
                    <input type="range" min="20" max="200"
                        prop:value=move || heart_rate.get()
                        on:input=move |ev| set_heart_rate.set(event_target_value(&ev).parse().unwrap_or(75))
                        class="w-full h-2 bg-rose-100 rounded-lg appearance-none cursor-pointer accent-rose-600" />
                </div>

                // Temperature
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-indigo-50">
                    <label class="flex justify-between items-center mb-4">
                        <span class="font-bold text-gray-700">{move || t(lang.get(), "temperature")}</span>
                        <span class="text-2xl font-black text-amber-600">{move || format!("{:.1}°C", temperature.get())}</span>
                    </label>
                    <input type="range" min="32.0" max="42.0" step="0.1"
                        prop:value=move || temperature.get()
                        on:input=move |ev| set_temperature.set(event_target_value(&ev).parse().unwrap_or(36.6))
                        class="w-full h-2 bg-amber-100 rounded-lg appearance-none cursor-pointer accent-amber-600" />
                </div>

                // Consciousness
                <div class="bg-white p-6 rounded-2xl shadow-sm border border-indigo-50 md:col-span-2">
                    <span class="font-bold text-gray-700 block mb-4">"Consciousness Level"</span>
                    <div class="flex gap-4">
                        <button
                            on:click=move |_| set_consciousness.set(ConsciousnessLevel::Alert)
                            class=move || format!("flex-1 py-4 rounded-xl font-bold flex flex-col items-center justify-center gap-1 border transition-all {}", if consciousness.get() == ConsciousnessLevel::Alert { "bg-emerald-600 text-white border-emerald-600 shadow-lg scale-105" } else { "bg-gray-50 text-gray-400 border-gray-200" })
                        >
                            <i class="fas fa-smile text-xl"></i>
                            <span>"Alert"</span>
                        </button>
                        <button
                            on:click=move |_| set_consciousness.set(ConsciousnessLevel::CVPU)
                            class=move || format!("flex-1 py-4 rounded-xl font-bold flex flex-col items-center justify-center gap-1 border transition-all {}", if consciousness.get() == ConsciousnessLevel::CVPU { "bg-red-600 text-white border-red-600 shadow-lg scale-105" } else { "bg-gray-50 text-gray-400 border-gray-200" })
                        >
                            <i class="fas fa-exclamation-triangle text-xl"></i>
                            <span>"New Confusion, Voice, Pain, Unresponsive"</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
