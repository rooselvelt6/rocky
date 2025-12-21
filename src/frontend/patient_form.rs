use crate::frontend::i18n::{t, use_i18n};
use crate::models::patient::{AdmissionType, Patient, SkinColor};
use leptos::*;

#[component]
pub fn PatientForm() -> impl IntoView {
    let lang = use_i18n();
    let navigate = leptos_router::use_navigate();
    // navigate is a closure, it might not be Clone, so we might need to rely on it being Copy or wrap it?
    // Actually, let's try just moving a clone if possible, or assume it's not Clone and use a trick.
    // If we cannot clone 'navigate' (some impl Fn are not Clone), we can't use it multiple times in spawn_local.
    // However, we can use `Batch` or `Signal`?
    // Let's try wrapping it in Rc to be safe.
    let navigate = std::rc::Rc::new(navigate);
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (dob, set_dob) = create_signal(String::new());
    let (gender, set_gender) = create_signal("Male".to_string());

    // New fields
    let (hospital_admission, set_hospital_admission) = create_signal(String::new());
    let (uci_admission, set_uci_admission) = create_signal(String::new());
    let (skin_color, set_skin_color) = create_signal("White".to_string());
    let (diagnosis, set_diagnosis) = create_signal(String::new());
    let (mech_vent, set_mech_vent) = create_signal(false);
    let (uci_hist, set_uci_hist) = create_signal(false);
    let (transfer, set_transfer) = create_signal(false);
    let (admission_type, set_admission_type) = create_signal("Urgent".to_string());
    let (invasive, set_invasive) = create_signal(false);

    let (submit_status, set_submit_status) = create_signal(Option::<String>::None);

    // Derived signal for days in hospital
    let days_in_hospital = move || {
        let hosp = hospital_admission.get();
        let uci = uci_admission.get();
        if hosp.is_empty() || uci.is_empty() {
            return "N/A".to_string();
        }

        let h_date = chrono::NaiveDate::parse_from_str(&hosp, "%Y-%m-%d").ok();
        let u_date = chrono::NaiveDate::parse_from_str(&uci, "%Y-%m-%d").ok();

        if let (Some(h), Some(u)) = (h_date, u_date) {
            let diff = u.signed_duration_since(h).num_days();
            format!("{} {}", diff.max(0), t(lang.get(), "days"))
        } else {
            t(lang.get(), "invalid_dates")
        }
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let skin_enum = match skin_color.get().as_str() {
            "Mixed" => SkinColor::Mixed,
            "Black" => SkinColor::Black,
            _ => SkinColor::White,
        };

        let adm_enum = match admission_type.get().as_str() {
            "Programmed" => AdmissionType::Programmed,
            "Transfer" => AdmissionType::Transfer,
            _ => AdmissionType::Urgent,
        };

        // Ensure dates are in ISO 8601 format (simplified for this context, assuming standard date input YYYY-MM-DD)
        // We append T00:00:00Z for simplicity as the input type="date" returns YYYY-MM-DD
        let format_date = |d: String| format!("{}T00:00:00Z", d);

        let patient = Patient::new(
            first_name.get(),
            last_name.get(),
            dob.get(),
            gender.get(),
            format_date(hospital_admission.get()),
            format_date(uci_admission.get()),
            skin_enum,
            diagnosis.get(),
            mech_vent.get(),
            uci_hist.get(),
            transfer.get(),
            adm_enum,
            invasive.get(),
        );

        let navigate = navigate.clone();
        spawn_local(async move {
            let client = reqwasm::http::Request::post("/api/patients")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&patient).unwrap())
                .send()
                .await;

            match client {
                Ok(resp) => {
                    if resp.ok() {
                        set_submit_status.set(Some(t(lang.get(), "success_register")));
                        // Delay redirect for user to see success
                        set_timeout(
                            move || {
                                navigate("/patients", Default::default());
                            },
                            std::time::Duration::from_millis(1500),
                        );
                    } else {
                        set_submit_status.set(Some(format!(
                            "{}: {}",
                            t(lang.get(), "network_error"),
                            resp.status_text()
                        )));
                    }
                }
                Err(e) => {
                    set_submit_status.set(Some(format!(
                        "{}: {}",
                        t(lang.get(), "network_error"),
                        e
                    )));
                }
            }
        });
    };

    view! {
        <div class="bg-white p-8 rounded-2xl shadow-xl max-w-5xl mx-auto border-t-8 border-indigo-600">
            <div class="text-center mb-8">
                <h2 class="text-3xl font-bold text-gray-800 flex items-center justify-center gap-3">
                    <i class="fas fa-user-plus text-indigo-600 text-4xl"></i>
                    {move || t(lang.get(), "patient_registration")}
                </h2>
                <p class="text-gray-500 mt-2 text-lg">{move || t(lang.get(), "enter_clinical_details")}</p>
            </div>

            <form on:submit=on_submit class="space-y-8">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                    // Personal Information Section
                    <div class="bg-indigo-50 p-6 rounded-xl border border-indigo-100 shadow-sm relative overflow-hidden">
                        <div class="absolute top-0 right-0 p-4 opacity-10 pointer-events-none">
                            <i class="fas fa-id-card text-9xl text-indigo-900"></i>
                        </div>
                        <h3 class="text-xl font-bold text-indigo-800 mb-6 flex items-center border-b border-indigo-200 pb-2">
                            <i class="fas fa-user-circle mr-3"></i> {move || t(lang.get(), "personal_information")}
                        </h3>

                        <div class="space-y-5 relative z-10">
                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-1">
                                    <i class="fas fa-user mr-2 text-indigo-500 w-5 text-center"></i> {move || t(lang.get(), "first_name")}
                                </label>
                                <input type="text" class="w-full rounded-lg border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 py-2 px-3 transition-colors"
                                    prop:value=first_name on:input=move |ev| set_first_name.set(event_target_value(&ev)) required />
                            </div>
                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-1">
                                    <i class="fas fa-user mr-2 text-indigo-500 w-5 text-center"></i> {move || t(lang.get(), "last_name")}
                                </label>
                                <input type="text" class="w-full rounded-lg border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 py-2 px-3 transition-colors"
                                    prop:value=last_name on:input=move |ev| set_last_name.set(event_target_value(&ev)) required />
                            </div>
                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-1">
                                    <i class="fas fa-birthday-cake mr-2 text-indigo-500 w-5 text-center"></i> {move || t(lang.get(), "dob")}
                                </label>
                                <input type="date" class="w-full rounded-lg border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 py-2 px-3 transition-colors"
                                    prop:value=dob on:input=move |ev| set_dob.set(event_target_value(&ev)) required />
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-1">
                                        <i class="fas fa-venus-mars mr-2 text-indigo-500 w-5 text-center"></i> {move || t(lang.get(), "gender")}
                                    </label>
                                    <select class="w-full rounded-lg border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 py-2 px-3 cursor-pointer"
                                        prop:value=gender on:change=move |ev| set_gender.set(event_target_value(&ev))>
                                        <option value="Male">{move || t(lang.get(), "male")}</option>
                                        <option value="Female">{move || t(lang.get(), "female")}</option>
                                        <option value="Other">{move || t(lang.get(), "other")}</option>
                                    </select>
                                </div>
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-1">
                                        <i class="fas fa-palette mr-2 text-indigo-500 w-5 text-center"></i> {move || t(lang.get(), "skin_color")}
                                    </label>
                                    <select class="w-full rounded-lg border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 py-2 px-3 cursor-pointer"
                                        prop:value=skin_color on:change=move |ev| set_skin_color.set(event_target_value(&ev))>
                                        <option value="White">{move || format!("⚪ {}", t(lang.get(), "white"))}</option>
                                        <option value="Mixed">{move || format!("🏽 {}", t(lang.get(), "mixed"))}</option>
                                        <option value="Black">{move || format!("⚫ {}", t(lang.get(), "black"))}</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Clinical Information Section
                    <div class="bg-teal-50 p-6 rounded-xl border border-teal-100 shadow-sm relative overflow-hidden">
                        <div class="absolute top-0 right-0 p-4 opacity-10 pointer-events-none">
                            <i class="fas fa-notes-medical text-9xl text-teal-900"></i>
                        </div>
                        <h3 class="text-xl font-bold text-teal-800 mb-6 flex items-center border-b border-teal-200 pb-2">
                            <i class="fas fa-clinic-medical mr-3"></i> {move || t(lang.get(), "clinical_information")}
                        </h3>

                        <div class="space-y-5 relative z-10">
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-1">
                                        <i class="fas fa-hospital mr-2 text-teal-600 w-5 text-center"></i> {move || t(lang.get(), "hospital_adm")}
                                    </label>
                                    <input type="date" class="w-full rounded-lg border-gray-300 shadow-sm focus:border-teal-500 focus:ring-teal-500 py-2 px-3 transition-colors"
                                        prop:value=hospital_admission on:input=move |ev| set_hospital_admission.set(event_target_value(&ev)) required />
                                </div>
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-1">
                                        <i class="fas fa-procedures mr-2 text-teal-600 w-5 text-center"></i> {move || t(lang.get(), "uci_adm")}
                                    </label>
                                    <input type="date" class="w-full rounded-lg border-gray-300 shadow-sm focus:border-teal-500 focus:ring-teal-500 py-2 px-3 transition-colors"
                                        prop:value=uci_admission on:input=move |ev| set_uci_admission.set(event_target_value(&ev)) required />
                                </div>
                            </div>

                            <div class="bg-white/80 backdrop-blur-sm p-4 rounded-lg border border-teal-200 flex items-center justify-between shadow-sm">
                                <span class="text-sm font-bold text-teal-800 flex items-center">
                                    <i class="fas fa-hourglass-half mr-2 text-xl"></i> {move || t(lang.get(), "days_in_hospital")}
                                </span>
                                <span class="text-2xl font-black text-teal-600 bg-teal-50 px-3 py-1 rounded-md border border-teal-100 min-w-[3rem] text-center">
                                    {days_in_hospital}
                                </span>
                            </div>

                             <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-1">
                                    <i class="fas fa-file-signature mr-2 text-teal-500 w-5 text-center"></i> {move || t(lang.get(), "admission_type")}
                                </label>
                                <select class="w-full rounded-lg border-gray-300 shadow-sm focus:border-teal-500 focus:ring-teal-500 py-2 px-3 cursor-pointer"
                                    prop:value=admission_type on:change=move |ev| set_admission_type.set(event_target_value(&ev))>
                                    <option value="Urgent">{move || format!("🚑 {}", t(lang.get(), "urgent"))}</option>
                                    <option value="Programmed">{move || format!("📅 {}", t(lang.get(), "programmed"))}</option>
                                    <option value="Transfer">{move || format!("🔄 {}", t(lang.get(), "transfer"))}</option>
                                </select>
                            </div>

                             <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-1">
                                    <i class="fas fa-stethoscope mr-2 text-teal-500 w-5 text-center"></i> {move || t(lang.get(), "principal_diagnosis")}
                                </label>
                                <textarea class="w-full rounded-lg border-gray-300 shadow-sm focus:border-teal-500 focus:ring-teal-500 py-2 px-3 transition-colors"
                                    rows="2" prop:value=diagnosis on:input=move |ev| set_diagnosis.set(event_target_value(&ev)) required prop:placeholder=move || t(lang.get(), "enter_diagnosis_placeholder")>
                                </textarea>
                            </div>

                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
                                <div class="flex items-center p-2 rounded-lg hover:bg-teal-100/50 transition-colors">
                                    <input type="checkbox" class="h-5 w-5 text-teal-600 focus:ring-teal-500 border-gray-300 rounded cursor-pointer"
                                        prop:checked=mech_vent on:change=move |ev| set_mech_vent.set(event_target_checked(&ev)) />
                                    <label class="ml-3 block text-sm font-medium text-gray-800 cursor-pointer flex items-center">
                                        <i class="fas fa-lungs text-teal-500 mr-2"></i> {move || t(lang.get(), "mech_ventilation")}
                                    </label>
                                </div>
                                <div class="flex items-center p-2 rounded-lg hover:bg-teal-100/50 transition-colors">
                                    <input type="checkbox" class="h-5 w-5 text-teal-600 focus:ring-teal-500 border-gray-300 rounded cursor-pointer"
                                        prop:checked=uci_hist on:change=move |ev| set_uci_hist.set(event_target_checked(&ev)) />
                                    <label class="ml-3 block text-sm font-medium text-gray-800 cursor-pointer flex items-center">
                                        <i class="fas fa-history text-teal-500 mr-2"></i> {move || t(lang.get(), "history_uci")}
                                    </label>
                                </div>
                                <div class="flex items-center p-2 rounded-lg hover:bg-teal-100/50 transition-colors">
                                    <input type="checkbox" class="h-5 w-5 text-teal-600 focus:ring-teal-500 border-gray-300 rounded cursor-pointer"
                                        prop:checked=transfer on:change=move |ev| set_transfer.set(event_target_checked(&ev)) />
                                    <label class="ml-3 block text-sm font-medium text-gray-800 cursor-pointer flex items-center">
                                        <i class="fas fa-ambulance text-teal-500 mr-2"></i> {move || t(lang.get(), "transfer_other_center")}
                                    </label>
                                </div>
                                <div class="flex items-center p-2 rounded-lg hover:bg-teal-100/50 transition-colors">
                                    <input type="checkbox" class="h-5 w-5 text-teal-600 focus:ring-teal-500 border-gray-300 rounded cursor-pointer"
                                        prop:checked=invasive on:change=move |ev| set_invasive.set(event_target_checked(&ev)) />
                                    <label class="ml-3 block text-sm font-medium text-gray-800 cursor-pointer flex items-center">
                                        <i class="fas fa-syringe text-teal-500 mr-2"></i> {move || t(lang.get(), "invasive_processes")}
                                    </label>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="pt-6">
                    <button type="submit" class="w-full flex items-center justify-center py-4 px-4 border border-transparent rounded-xl shadow-lg text-lg font-bold text-white bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transform hover:scale-[1.02] transition-all duration-200">
                        <i class="fas fa-save mr-3 text-2xl"></i> {move || t(lang.get(), "register_patient_btn")}
                    </button>
                </div>
            </form>

            {move || submit_status.get().map(|status| {
                let is_error = status.starts_with("Error") || status.starts_with("Network");
                let color = if is_error { "red" } else { "green" };
                let icon = if is_error { "exclamation-triangle" } else { "check-circle" };
                view! {
                    <div class=format!("mt-6 p-4 rounded-xl border border-{}-200 bg-{}-50 text-{}-800 flex items-center shadow-sm animate-fade-in", color, color, color)>
                        <i class=format!("fas fa-{} text-2xl mr-3 text-{}-600", icon, color)></i>
                        <span class="font-medium text-lg">{status}</span>
                    </div>
                }
            })}
        </div>
    }
}
