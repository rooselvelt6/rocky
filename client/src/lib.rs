use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

// ============================================
// MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub identity_card: String,
    pub principal_diagnosis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub username: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpResponse {
    pub success: bool,
    pub session_id: Option<String>,
    pub message: String,
    pub requires_otp: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct God {
    pub name: String,
    pub domain: String,
    pub active: bool,
    pub status: String,
}

// Modelos para Aphrodite (UI/Temas)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub background: String,
    pub surface: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub accent: String,
    pub border_radius: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemesResponse {
    pub themes: Vec<String>,
    pub current: String,
    pub designed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentThemeResponse {
    pub theme: Theme,
    pub controlled_by: String,
}

// ============================================
// MAIN APP
// ============================================

#[component]
pub fn App() -> impl IntoView {
    let page = RwSignal::new("/".to_string());
    let is_logged_in = RwSignal::new(false);
    let current_user = RwSignal::new(String::new());
    let current_theme = RwSignal::new("Olympus Dark".to_string());
    
    // Cargar tema actual al iniciar
    spawn_local(async move {
        if let Ok(resp) = reqwasm::http::Request::get("/api/aphrodite/theme").send().await {
            if let Ok(data) = resp.json::<CurrentThemeResponse>().await {
                let theme_name = data.theme.name.clone();
                // Aplicar CSS variables al documento
                apply_theme_to_document(&data.theme);
                current_theme.set(theme_name);
            }
        }
    });

    view! {
        <div class="min-h-screen bg-slate-900" id="app-container">
            {move || {
                if !is_logged_in.get() {
                    view! { 
                        <LoginPage on_login=move |u: String, _t: String| { is_logged_in.set(true); current_user.set(u); }/> 
                    }.into_any()
                } else {
                    view! {
                        <div>
                            <nav class="bg-slate-800 p-4 text-white flex justify-between items-center border-b border-pink-500/30">
                                <div class="flex items-center gap-3">
                                    <span class="text-2xl font-bold text-indigo-400">OLYMPUS UCI</span>
                                    <span class="text-xs text-pink-400 flex items-center gap-1">
                                        <span>"🎨 "</span>
                                        {current_theme.get()}
                                    </span>
                                </div>
                                <div class="flex gap-2">
                                    <button on:click=move |_| page.set("/".to_string()) class="px-3 py-1 bg-slate-700 rounded hover:bg-slate-600">Inicio</button>
                                    <button on:click=move |_| page.set("/patients".to_string()) class="px-3 py-1 bg-slate-700 rounded hover:bg-slate-600">Pacientes</button>
                                    <button on:click=move |_| page.set("/scales".to_string()) class="px-3 py-1 bg-slate-700 rounded hover:bg-slate-600">Escalas</button>
                                    <button on:click=move |_| page.set("/gods".to_string()) class="px-3 py-1 bg-slate-700 rounded hover:bg-slate-600">Dioses</button>
                                    <button on:click=move |_| page.set("/aphrodite".to_string()) class="px-3 py-1 bg-pink-600 rounded hover:bg-pink-500 flex items-center gap-1">
                                        <span>"✨"</span>
                                        <span>"Aphrodite"</span>
                                    </button>
                                    <button on:click=move |_| is_logged_in.set(false) class="px-3 py-1 bg-red-600 rounded hover:bg-red-500">Salir</button>
                                </div>
                            </nav>
                            <main class="p-6 max-w-7xl mx-auto">
                                {move || {
                                    match page.get().as_str() {
                                        "/patients" => view! { <PatientPage/> }.into_any(),
                                        "/scales" => view! { <ScalesPage/> }.into_any(),
                                        "/aphrodite" => view! { <AphroditePage current_theme={current_theme}/> }.into_any(),
                                        "/gods" => view! { <OlympusMonitor/> }.into_any(),
                                        _ => view! { <Dashboard/> }.into_any(),
                                    }
                                }}
                            </main>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

// ============================================
// LOGIN PAGE
// ============================================

#[component]
fn LoginPage<F>(on_login: F) -> impl IntoView 
where F: Fn(String, String) + Clone + Send + Sync + 'static
{
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let otp = RwSignal::new(String::new());
    let step = RwSignal::new(1i32);
    let message = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

    let do_login = move |_| {
        loading.set(true);
        let user = username.get();
        
        spawn_local(async move {
            let res = reqwasm::http::Request::post("/api/login_step1")
                .header("Content-Type", "application/json")
                .body(serde_json::json!({"username": user, "password": "admin123"}).to_string())
                .send().await;
            
            loading.set(false);
            
            if let Ok(resp) = res {
                if let Ok(data) = resp.json::<OtpResponse>().await {
                    if data.success {
                        step.set(2);
                        message.set("Codigo OTP: 123456".to_string());
                    }
                }
            }
        });
    };

    let do_verify = move |_| {
        loading.set(true);
        let callback = on_login.clone();
        
        spawn_local(async move {
            let res = reqwasm::http::Request::post("/api/login_step2")
                .header("Content-Type", "application/json")
                .body(serde_json::json!({"session_id": "session_123", "otp_code": otp.get()}).to_string())
                .send().await;
            
            loading.set(false);
            
            if let Ok(resp) = res {
                if let Ok(data) = resp.json::<AuthResponse>().await {
                    if data.success {
                        callback("admin".to_string(), "token".to_string());
                    } else {
                        message.set(data.message);
                    }
                }
            }
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-slate-900">
            <div class="bg-slate-800 p-8 rounded-xl border border-slate-700 w-96">
                <h1 class="text-3xl font-bold text-indigo-400 text-center mb-6">OLYMPUS</h1>
                <div class="space-y-4">
                    {move || {
                        if step.get() == 1 {
                            view! {
                                <>
                                    <input type="text" placeholder="Usuario" 
                                        on:input=move |e| username.set(event_target_value(&e))
                                        class="w-full p-3 bg-slate-700 border border-slate-600 rounded text-white"/>
                                    <input type="password" placeholder="Contraseña" 
                                        on:input=move |e| password.set(event_target_value(&e))
                                        class="w-full p-3 bg-slate-700 border border-slate-600 rounded text-white"/>
                                </>
                            }.into_any()
                        } else {
                            view! {
                                <div>
                                    <p class="text-indigo-300 text-sm mb-2 text-center">"Codigo OTP: 123456"</p>
                                    <input type="text" placeholder="Codigo OTP" maxlength="6"
                                        on:input=move |e| otp.set(event_target_value(&e))
                                        class="w-full p-3 bg-slate-700 border border-slate-600 rounded text-white text-center text-2xl tracking-widest"/>
                                </div>
                            }.into_any()
                        }
                    }}
                    
                    {move || {
                        if !message.get().is_empty() {
                            view! { <p class="text-amber-400 text-sm text-center">{message.get()}</p> }.into_any()
                        } else { view! { <div></div> }.into_any() }
                    }}
                    
                    <button on:click=move |ev| { if step.get() == 1 { do_login(ev); } else { do_verify(ev); } } 
                        disabled={loading.get()}
                        class="w-full py-3 bg-indigo-600 text-white rounded disabled:opacity-50">
                        {move || if loading.get() { "Procesando..." } else { if step.get() == 1 { "Continuar" } else { "Verificar" } }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ============================================
// DASHBOARD
// ============================================

#[component]
fn Dashboard() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h2 class="text-3xl font-bold text-white text-center">Panel de Control UCI</h2>
            <div class="grid grid-cols-4 gap-4">
                <div class="bg-slate-800 p-6 rounded-xl border border-slate-700 text-center">
                    <p class="text-indigo-300">Pacientes</p>
                    <p class="text-4xl text-white font-bold">0</p>
                </div>
                <div class="bg-slate-800 p-6 rounded-xl border border-slate-700 text-center">
                    <p class="text-blue-300">En UCI</p>
                    <p class="text-4xl text-white font-bold">0</p>
                </div>
                <div class="bg-slate-800 p-6 rounded-xl border border-red-500/30 text-center">
                    <p class="text-red-300">Criticos</p>
                    <p class="text-4xl text-red-400 font-bold">0</p>
                </div>
                <div class="bg-slate-800 p-6 rounded-xl border border-green-500/30 text-center">
                    <p class="text-green-300">Estables</p>
                    <p class="text-4xl text-green-400 font-bold">0</p>
                </div>
            </div>
            
            <div class="bg-slate-800 p-6 rounded-xl border border-slate-700 mt-8">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-white font-bold text-xl">Estado del Olimpo</p>
                        <p class="text-slate-400 text-sm">20 Dioses Activos</p>
                    </div>
                    <div class="text-right">
                        <p class="text-green-400 font-semibold">Sistema Operativo</p>
                        <p class="text-slate-500 text-sm">Zeus Orquestando</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================
// PATIENT PAGE
// ============================================

#[component]
fn PatientPage() -> impl IntoView {
    let patients = RwSignal::new(Vec::<Patient>::new());
    let show_form = RwSignal::new(false);
    let message = RwSignal::new(String::new());
    
    let load_patients = move || {
        spawn_local(async move {
            if let Ok(resp) = reqwasm::http::Request::get("/api/patients").send().await {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(list) = data.get("patients").and_then(|v| v.as_array()) {
                        let parsed: Vec<Patient> = list.iter().filter_map(|x| serde_json::from_value(x.clone()).ok()).collect();
                        patients.set(parsed);
                    }
                }
            }
        });
    };
    
    load_patients();

    view! {
        <div class="space-y-6">
            <div class="flex justify-between items-center">
                <h2 class="text-2xl text-white font-bold">Pacientes</h2>
                <button on:click=move |_| show_form.set(true)
                    class="px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-500">
                    Nuevo Paciente
                </button>
            </div>
            
            {move || {
                if show_form.get() {
                    view! { 
                        <PatientForm 
                            on_save=move || { show_form.set(false); load_patients(); message.set("Paciente guardado".to_string()); }
                            on_cancel=move || show_form.set(false)
                        /> 
                    }.into_any()
                } else {
                    view! {
                        <>
                            {move || {
                                if !message.get().is_empty() {
                                    view! { <p class="text-green-400 mb-4">{message.get()}</p> }.into_any()
                                } else { view! { <div></div> }.into_any() }
                            }}
                            <PatientList patients={patients.get()} on_reload={load_patients}/>
                        </>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn PatientList(patients: Vec<Patient>, on_reload: impl Fn() + 'static + Clone) -> impl IntoView {
    let reload = on_reload.clone();
    
    view! {
        <div class="bg-slate-800 rounded-xl border border-slate-700">
            {if patients.is_empty() {
                view! {
                    <div class="p-8 text-center text-slate-500">
                        No hay pacientes registrados
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="divide-y divide-slate-700">
                        {patients.into_iter().map(|p| {
                            let id = p.id.clone().unwrap_or_default();
                            let reload = reload.clone();
                            view! {
                                <div class="p-4 flex justify-between items-center">
                                    <div>
                                        <p class="text-white font-medium">{format!("{} {}", p.first_name, p.last_name)}</p>
                                        <p class="text-slate-500 text-sm">{p.identity_card.clone()}</p>
                                        <p class="text-slate-400 text-sm">{p.principal_diagnosis.clone()}</p>
                                    </div>
                                    <button on:click=move |_| {
                                        let id = id.clone();
                                        let reload = reload.clone();
                                        spawn_local(async move {
                                            let _ = reqwasm::http::Request::delete(&format!("/api/patients/{}", id)).send().await;
                                            reload();
                                        });
                                    }
                                        class="px-3 py-1 bg-red-600/20 text-red-400 rounded hover:bg-red-600/30">
                                        Eliminar
                                    </button>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn PatientForm(on_save: impl Fn() + 'static + Clone, on_cancel: impl Fn() + 'static + Clone) -> impl IntoView {
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let identity_card = RwSignal::new(String::new());
    let diagnosis = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    
    let save = move |_| {
        saving.set(true);
        let callback = on_save.clone();
        
        spawn_local(async move {
            let patient = Patient {
                id: None,
                first_name: first_name.get(),
                last_name: last_name.get(),
                identity_card: identity_card.get(),
                principal_diagnosis: diagnosis.get(),
            };
            
            let _ = reqwasm::http::Request::post("/api/patients")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&patient).unwrap_or_default())
                .send().await;
            
            saving.set(false);
            callback();
        });
    };
    
    view! {
        <div class="bg-slate-800 p-6 rounded-xl border border-slate-700">
            <h3 class="text-xl font-bold text-white mb-4">Nuevo Paciente</h3>
            
            <div class="space-y-4">
                <input type="text" placeholder="Nombre" 
                    on:input=move |e| first_name.set(event_target_value(&e))
                    class="w-full p-3 bg-slate-700 border border-slate-600 rounded text-white"/>
                <input type="text" placeholder="Apellido" 
                    on:input=move |e| last_name.set(event_target_value(&e))
                    class="w-full p-3 bg-slate-700 border border-slate-600 rounded text-white"/>
                <input type="text" placeholder="Cedula" 
                    on:input=move |e| identity_card.set(event_target_value(&e))
                    class="w-full p-3 bg-slate-700 border border-slate-600 rounded text-white"/>
                <input type="text" placeholder="Diagnostico" 
                    on:input=move |e| diagnosis.set(event_target_value(&e))
                    class="w-full p-3 bg-slate-700 border border-slate-600 rounded text-white"/>
            </div>
            
            <div class="flex justify-end gap-3 mt-6">
                <button on:click=move |_| on_cancel()
                    class="px-4 py-2 bg-slate-700 text-white rounded hover:bg-slate-600">
                    Cancelar
                </button>
                <button on:click=save disabled={saving.get()}
                    class="px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-500 disabled:opacity-50">
                    {move || if saving.get() { "Guardando..." } else { "Guardar" }}
                </button>
            </div>
        </div>
    }
}

// ============================================
// SCALES PAGE
// ============================================

#[component]
fn ScalesPage() -> impl IntoView {
    let scale = RwSignal::new("glasgow".to_string());

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl text-white font-bold text-center">Escalas Medicas</h2>
            
            <div class="flex flex-wrap gap-2 justify-center">
                <button on:click=move |_| scale.set("glasgow".to_string()) 
                    class={move || format!("px-4 py-2 rounded {}", if scale.get() == "glasgow" { "bg-purple-600 ring-2 ring-white" } else { "bg-purple-600/50 hover:bg-purple-600" })}>
                    Glasgow
                </button>
                <button on:click=move |_| scale.set("sofa".to_string())
                    class={move || format!("px-4 py-2 rounded {}", if scale.get() == "sofa" { "bg-blue-600 ring-2 ring-white" } else { "bg-blue-600/50 hover:bg-blue-600" })}>
                    SOFA
                </button>
                <button on:click=move |_| scale.set("apache".to_string())
                    class={move || format!("px-4 py-2 rounded {}", if scale.get() == "apache" { "bg-red-600 ring-2 ring-white" } else { "bg-red-600/50 hover:bg-red-600" })}>
                    APACHE II
                </button>
                <button on:click=move |_| scale.set("saps".to_string())
                    class={move || format!("px-4 py-2 rounded {}", if scale.get() == "saps" { "bg-orange-600 ring-2 ring-white" } else { "bg-orange-600/50 hover:bg-orange-600" })}>
                    SAPS II
                </button>
                <button on:click=move |_| scale.set("news2".to_string())
                    class={move || format!("px-4 py-2 rounded {}", if scale.get() == "news2" { "bg-green-600 ring-2 ring-white" } else { "bg-green-600/50 hover:bg-green-600" })}>
                    NEWS2
                </button>
            </div>
            
            <div class="bg-slate-800 p-6 rounded-xl border border-slate-700 max-w-2xl mx-auto">
                {move || {
                    match scale.get().as_str() {
                        "sofa" => view! { <SofaForm/> }.into_any(),
                        "apache" => view! { <ApacheForm/> }.into_any(),
                        "saps" => view! { <SapsForm/> }.into_any(),
                        "news2" => view! { <News2Form/> }.into_any(),
                        _ => view! { <GlasgowForm/> }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn GlasgowForm() -> impl IntoView {
    let eye = RwSignal::new(4i32);
    let verbal = RwSignal::new(4i32);
    let motor = RwSignal::new(5i32);
    let total = RwSignal::new(13i32);
    
    Effect::new(move |_| { 
        total.set(eye.get() + verbal.get() + motor.get()); 
    });

    view! {
        <div class="space-y-6">
            <h3 class="text-xl text-white font-bold text-center">Glasgow Coma Scale</h3>
            
            <ScaleSlider label="Apertura Ocular" value={eye} min=1 max=4/>
            <ScaleSlider label="Respuesta Verbal" value={verbal} min=1 max=5/>
            <ScaleSlider label="Respuesta Motora" value={motor} min=1 max=6/>
            
            <div class="text-center p-6 bg-slate-700 rounded-xl">
                <p class="text-slate-400 text-sm mb-2">Puntuacion Total</p>
                <p class="text-6xl font-bold text-white">{total.get()}</p>
                <p class="text-indigo-400 text-sm mt-2">/ 15</p>
            </div>
            
            <button on:click=move |_| {}
                class="w-full py-3 bg-purple-600 hover:bg-purple-500 text-white rounded transition">
                Guardar Escala
            </button>
        </div>
    }
}

#[component]
fn SofaForm() -> impl IntoView {
    let resp = RwSignal::new(0i32);
    let coag = RwSignal::new(0i32);
    let liver = RwSignal::new(0i32);
    let cardio = RwSignal::new(0i32);
    let cns = RwSignal::new(0i32);
    let renal = RwSignal::new(0i32);
    let total = RwSignal::new(0i32);
    
    Effect::new(move |_| { 
        total.set(resp.get() + coag.get() + liver.get() + cardio.get() + cns.get() + renal.get()); 
    });

    view! {
        <div class="space-y-4">
            <h3 class="text-xl text-white font-bold text-center">SOFA Score</h3>
            
            <ScaleSlider label="Respiratorio" value={resp} min=0 max=4/>
            <ScaleSlider label="Coagulacion" value={coag} min=0 max=4/>
            <ScaleSlider label="Higado" value={liver} min=0 max=4/>
            <ScaleSlider label="Cardiovascular" value={cardio} min=0 max=4/>
            <ScaleSlider label="SNC" value={cns} min=0 max=4/>
            <ScaleSlider label="Renal" value={renal} min=0 max=4/>
            
            <div class="text-center p-6 bg-slate-700 rounded-xl">
                <p class="text-slate-400 text-sm mb-2">Puntuacion Total</p>
                <p class="text-6xl font-bold text-white">{total.get()}</p>
                <p class="text-blue-400 text-sm mt-2">/ 24</p>
            </div>
            
            <button on:click=move |_| {}
                class="w-full py-3 bg-blue-600 hover:bg-blue-500 text-white rounded transition">
                Guardar Escala
            </button>
        </div>
    }
}

#[component]
fn ApacheForm() -> impl IntoView {
    let temp = RwSignal::new(37.0f32);
    let hr = RwSignal::new(80i32);

    view! {
        <div class="space-y-4">
            <h3 class="text-xl text-white font-bold text-center">APACHE II</h3>
            <p class="text-slate-400 text-center text-sm">Escala de gravedad fisiologica</p>
            
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="text-slate-400 text-sm">Temperatura (C)</label>
                    <input type="number" value={temp.get()} 
                        on:input=move |e| temp.set(event_target_value(&e).parse().unwrap_or(37.0))
                        class="w-full p-2 bg-slate-700 border border-slate-600 rounded text-white"/>
                </div>
                <div>
                    <label class="text-slate-400 text-sm">FC (lpm)</label>
                    <input type="number" value={hr.get()} 
                        on:input=move |e| hr.set(event_target_value(&e).parse().unwrap_or(80))
                        class="w-full p-2 bg-slate-700 border border-slate-600 rounded text-white"/>
                </div>
            </div>
            
            <button on:click=move |_| {}
                class="w-full py-3 bg-red-600 hover:bg-red-500 text-white rounded transition">
                Guardar APACHE
            </button>
        </div>
    }
}

#[component]
fn SapsForm() -> impl IntoView {
    let age = RwSignal::new(50i32);
    let hr = RwSignal::new(80i32);

    view! {
        <div class="space-y-4">
            <h3 class="text-xl text-white font-bold text-center">SAPS II</h3>
            
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="text-slate-400 text-sm">Edad</label>
                    <input type="number" value={age.get()} 
                        on:input=move |e| age.set(event_target_value(&e).parse().unwrap_or(50))
                        class="w-full p-2 bg-slate-700 border border-slate-600 rounded text-white"/>
                </div>
                <div>
                    <label class="text-slate-400 text-sm">FC (lpm)</label>
                    <input type="number" value={hr.get()} 
                        on:input=move |e| hr.set(event_target_value(&e).parse().unwrap_or(80))
                        class="w-full p-2 bg-slate-700 border border-slate-600 rounded text-white"/>
                </div>
            </div>
            
            <button on:click=move |_| {}
                class="w-full py-3 bg-orange-600 hover:bg-orange-500 text-white rounded transition">
                Guardar SAPS
            </button>
        </div>
    }
}

#[component]
fn News2Form() -> impl IntoView {
    let resp_rate = RwSignal::new(16i32);
    let hr = RwSignal::new(80i32);
    let total = RwSignal::new(0i32);

    view! {
        <div class="space-y-4">
            <h3 class="text-xl text-white font-bold text-center">NEWS2</h3>
            <p class="text-slate-400 text-center text-sm">National Early Warning Score</p>
            
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="text-slate-400 text-sm">FR (rpm)</label>
                    <input type="number" value={resp_rate.get()} 
                        on:input=move |e| resp_rate.set(event_target_value(&e).parse().unwrap_or(16))
                        class="w-full p-2 bg-slate-700 border border-slate-600 rounded text-white"/>
                </div>
                <div>
                    <label class="text-slate-400 text-sm">FC (lpm)</label>
                    <input type="number" value={hr.get()} 
                        on:input=move |e| hr.set(event_target_value(&e).parse().unwrap_or(80))
                        class="w-full p-2 bg-slate-700 border border-slate-600 rounded text-white"/>
                </div>
            </div>
            
            <div class="text-center p-6 bg-slate-700 rounded-xl">
                <p class="text-slate-400 text-sm mb-2">Puntuacion</p>
                <p class="text-5xl font-bold text-white">{total.get()}</p>
            </div>
            
            <button on:click=move |_| {}
                class="w-full py-3 bg-green-600 hover:bg-green-500 text-white rounded transition">
                Guardar NEWS2
            </button>
        </div>
    }
}

#[component]
fn ScaleSlider(label: &'static str, value: RwSignal<i32>, min: i32, max: i32) -> impl IntoView {
    view! {
        <div class="space-y-2">
            <div class="flex justify-between items-center">
                <span class="text-slate-300 text-sm">{label}</span>
                <span class="text-white font-bold">{value.get()}</span>
            </div>
            <input type="range" min={min} max={max} value={value.get()} 
                on:input=move |e| value.set(event_target_value(&e).parse().unwrap_or(min))
                class="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer accent-indigo-500"/>
        </div>
    }
}

// ============================================
// OLYMPUS MONITOR
// ============================================

#[component]
fn OlympusMonitor() -> impl IntoView {
    let gods = RwSignal::new(vec![
        ("Zeus", "Governance", "yellow"),
        ("Hades", "Security", "purple"),
        ("Poseidon", "DataFlow", "blue"),
        ("Athena", "Clinical", "indigo"),
        ("Apollo", "Events", "orange"),
        ("Artemis", "Search", "cyan"),
        ("Hermes", "Messaging", "pink"),
        ("Hera", "Validation", "red"),
        ("Ares", "Conflict", "red"),
        ("Hefesto", "Config", "orange"),
        ("Chronos", "Scheduling", "blue"),
        ("Moirai", "Predictions", "purple"),
        ("Chaos", "Testing", "slate"),
        ("Aurora", "NewBeginnings", "yellow"),
        ("Aphrodite", "UI/UX", "pink"),
        ("Iris", "Communications", "indigo"),
        ("Demeter", "Resources", "green"),
        ("Dionysus", "Analysis", "purple"),
        ("Hestia", "Persistence", "emerald"),
        ("Nemesis", "Compliance", "red"),
        ("Erinyes", "Integrity", "amber"),
    ]);

    view! {
        <div class="space-y-6">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Monitor del Olimpo</h2>
                <p class="text-slate-400">20 Dioses del Sistema Olympus</p>
            </div>
            
            <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4">
                {gods.get().into_iter().map(|(name, domain, color)| {
                    let bg_color = match color {
                        "yellow" => "bg-yellow-900/30 border-yellow-500/30",
                        "purple" => "bg-purple-900/30 border-purple-500/30",
                        "blue" => "bg-blue-900/30 border-blue-500/30",
                        "indigo" => "bg-indigo-900/30 border-indigo-500/30",
                        "orange" => "bg-orange-900/30 border-orange-500/30",
                        "cyan" => "bg-cyan-900/30 border-cyan-500/30",
                        "pink" => "bg-pink-900/30 border-pink-500/30",
                        "red" => "bg-red-900/30 border-red-500/30",
                        "slate" => "bg-slate-700 border-slate-600",
                        "green" => "bg-green-900/30 border-green-500/30",
                        "emerald" => "bg-emerald-900/30 border-emerald-500/30",
                        "amber" => "bg-amber-900/30 border-amber-500/30",
                        _ => "bg-slate-700 border-slate-600",
                    };
                    let text_color = match color {
                        "yellow" => "text-yellow-400",
                        "purple" => "text-purple-400",
                        "blue" => "text-blue-400",
                        "indigo" => "text-indigo-400",
                        "orange" => "text-orange-400",
                        "cyan" => "text-cyan-400",
                        "pink" => "text-pink-400",
                        "red" => "text-red-400",
                        "slate" => "text-slate-400",
                        "green" => "text-green-400",
                        "emerald" => "text-emerald-400",
                        "amber" => "text-amber-400",
                        _ => "text-slate-400",
                    };
                    
                    view! {
                        <div class={format!("p-4 rounded-xl border {}", bg_color)}>
                            <div class="flex items-center gap-2 mb-1">
                                <span class="w-2 h-2 rounded-full bg-green-400 animate-pulse"></span>
                                <span class={format!("font-bold {}", text_color)}>{name}</span>
                            </div>
                            <p class="text-slate-400 text-xs">{domain}</p>
                            <p class="text-slate-500 text-xs mt-1">Activo</p>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
            
            <div class="bg-slate-800 p-6 rounded-xl border border-slate-700 mt-8">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-white font-bold text-xl">Zeus - Orquestador Principal</p>
                        <p class="text-slate-400 text-sm">Supervisando 20 dioses - Uptime: 24h+</p>
                    </div>
                    <div class="text-right">
                        <p class="text-green-400 font-bold">Sistema Estable</p>
                        <p class="text-slate-500 text-sm">Ultimo heartbeat: Ahora</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ============================================
// UTILS
// ============================================

fn event_target_value(ev: &leptos::ev::Event) -> String {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|i| i.value())
        .unwrap_or_default()
}

// Aplicar tema de Aphrodite al documento
fn apply_theme_to_document(theme: &Theme) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        if let Some(root) = document.document_element() {
            let style = &root.dyn_into::<web_sys::HtmlElement>().unwrap().style();
            let _ = style.set_property("--color-primary", &theme.primary_color);
            let _ = style.set_property("--color-secondary", &theme.secondary_color);
            let _ = style.set_property("--color-background", &theme.background);
            let _ = style.set_property("--color-surface", &theme.surface);
            let _ = style.set_property("--color-text-primary", &theme.text_primary);
            let _ = style.set_property("--color-text-secondary", &theme.text_secondary);
            let _ = style.set_property("--color-accent", &theme.accent);
            let _ = style.set_property("--border-radius", &theme.border_radius);
        }
    }
}

// ============================================
// APHRODITE PAGE - Diosa de la Belleza
// ============================================

#[component]
fn AphroditePage(current_theme: RwSignal<String>) -> impl IntoView {
    let themes = RwSignal::new(Vec::<String>::new());
    let selected_theme = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

    // Cargar temas disponibles
    let load_themes = move || {
        spawn_local(async move {
            if let Ok(resp) = reqwasm::http::Request::get("/api/aphrodite/themes").send().await {
                if let Ok(data) = resp.json::<ThemesResponse>().await {
                    themes.set(data.themes);
                    current_theme.set(data.current);
                }
            }
        });
    };
    
    load_themes();

    let switch_theme = move |_| {
        if selected_theme.get().is_empty() {
            message.set("Selecciona un tema".to_string());
            return;
        }
        
        loading.set(true);
        let theme_name = selected_theme.get();
        
        spawn_local(async move {
            let res = reqwasm::http::Request::post("/api/aphrodite/theme")
                .header("Content-Type", "application/json")
                .body(serde_json::json!({"theme_name": theme_name}).to_string())
                .send().await;
            
            loading.set(false);
            
            if let Ok(_) = res {
                current_theme.set(theme_name.clone());
                message.set(format!("✨ Tema cambiado a: {}", theme_name));
                
                // Recargar tema actual para aplicar CSS
                if let Ok(resp) = reqwasm::http::Request::get("/api/aphrodite/theme").send().await {
                    if let Ok(data) = resp.json::<CurrentThemeResponse>().await {
                        apply_theme_to_document(&data.theme);
                    }
                }
            }
        });
    };

    view! {
        <div class="space-y-6">
            <div class="text-center mb-8">
                <h2 class="text-3xl font-bold text-white mb-2 flex items-center justify-center gap-2">
                    <span>"✨"</span>
                    <span>"Aphrodite - Diosa de la Belleza"</span>
                </h2>
                <p class="text-pink-400">"Gestiona la apariencia del Olimpo"</p>
            </div>
            
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Selector de Temas
                <div class="bg-slate-800 p-6 rounded-xl border border-pink-500/30">
                    <h3 class="text-xl font-bold text-white mb-4 flex items-center gap-2">
                        <span>"🎨"</span>
                        <span>"Temas Disponibles"</span>
                    </h3>
                    
                    <div class="space-y-3 mb-6">
                        {move || themes.get().into_iter().map(|theme_name| {
                            let is_selected = selected_theme.get() == theme_name;
                            let is_current = current_theme.get() == theme_name;
                            
                            view! {
                                <button
                                    on:click=move |_| selected_theme.set(theme_name.clone())
                                    class={format!(
                                        "w-full p-4 rounded-lg text-left transition flex justify-between items-center {}",
                                        if is_selected {
                                            "bg-pink-600 text-white ring-2 ring-pink-400"
                                        } else if is_current {
                                            "bg-pink-900/30 border border-pink-500/50 text-pink-300"
                                        } else {
                                            "bg-slate-700 text-slate-300 hover:bg-slate-600"
                                        }
                                    )}
                                >
                                    <span class="font-semibold">{theme_name.clone()}</span>
                                    {if is_current {
                                        view! { <span class="text-xs bg-pink-500 px-2 py-1 rounded">"Actual"</span> }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }}
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    
                    <button 
                        on:click=switch_theme
                        disabled={loading.get()}
                        class="w-full py-3 bg-gradient-to-r from-pink-600 to-purple-600 hover:from-pink-500 hover:to-purple-500 text-white rounded-lg font-semibold transition disabled:opacity-50"
                    >
                        {move || if loading.get() { "Aplicando..." } else { "Aplicar Tema" }}
                    </button>
                    
                    {move || {
                        if !message.get().is_empty() {
                            view! {
                                <div class="mt-4 p-3 bg-pink-900/30 border border-pink-500/30 rounded-lg">
                                    <p class="text-pink-300 text-center">{message.get()}</p>
                                </div>
                            }.into_any()
                        } else { view! { <div></div> }.into_any() }
                    }}
                </div>
                
                // Preview del Tema
                <div class="bg-slate-800 p-6 rounded-xl border border-pink-500/30">
                    <h3 class="text-xl font-bold text-white mb-4 flex items-center gap-2">
                        <span>"👁️"</span>
                        <span>"Vista Previa"</span>
                    </h3>
                    
                    <div class="space-y-4">
                        <div class="p-4 bg-slate-700 rounded-lg">
                            <p class="text-slate-400 text-sm mb-2">Tema Actual</p>
                            <p class="text-2xl font-bold text-pink-400">{current_theme.get()}</p>
                        </div>
                        
                        <div class="grid grid-cols-2 gap-3">
                            <div class="p-3 bg-indigo-600 rounded-lg text-center">
                                <p class="text-xs text-indigo-200">Primario</p>
                            </div>
                            <div class="p-3 bg-purple-600 rounded-lg text-center">
                                <p class="text-xs text-purple-200">Secundario</p>
                            </div>
                            <div class="p-3 bg-pink-600 rounded-lg text-center">
                                <p class="text-xs text-pink-200">Acento</p>
                            </div>
                            <div class="p-3 bg-emerald-600 rounded-lg text-center">
                                <p class="text-xs text-emerald-200">Éxito</p>
                            </div>
                        </div>
                        
                        <div class="p-4 bg-slate-700 rounded-lg border border-slate-600">
                            <p class="text-white font-semibold mb-2">Componentes</p>
                            <div class="flex gap-2 flex-wrap">
                                <span class="px-3 py-1 bg-pink-600 text-white rounded text-sm">Botones</span>
                                <span class="px-3 py-1 bg-indigo-600 text-white rounded text-sm">Tarjetas</span>
                                <span class="px-3 py-1 bg-purple-600 text-white rounded text-sm">Navegación</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            
            // Información de Aphrodite
            <div class="bg-gradient-to-r from-pink-900/30 to-purple-900/30 p-6 rounded-xl border border-pink-500/30">
                <div class="flex items-start gap-4">
                    <div class="text-4xl">"🎨"</div>
                    <div>
                        <h4 class="text-lg font-bold text-white mb-2">"Aphrodite - UI/UX Goddess"</h4>
                        <p class="text-slate-300 text-sm">
                            "Aphrodite gestiona la belleza visual del Olimpo. Ella controla los temas, 
                            colores y componentes de la interfaz. Cada cambio que realices es procesado 
                            por ella y aplicado al sistema en tiempo real."
                        </p>
                        <div class="mt-3 flex gap-4 text-xs text-slate-400">
                            <span>"✨ 4 temas predefinidos"</span>
                            <span>"🎨 CSS variables dinámicas"</span>
                            <span>"💅 Componentes gestionados"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
