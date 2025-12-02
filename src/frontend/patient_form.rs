use crate::models::patient::Patient;
use leptos::*;

#[component]
pub fn PatientForm() -> impl IntoView {
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (dob, set_dob) = create_signal(String::new());
    let (gender, set_gender) = create_signal("Male".to_string());
    let (diagnosis, set_diagnosis) = create_signal(String::new());
    let (submit_status, set_submit_status) = create_signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let patient = Patient::new(
            first_name.get(),
            last_name.get(),
            dob.get(),
            gender.get(),
            diagnosis.get(),
        );

        spawn_local(async move {
            let client = reqwasm::http::Request::post("/api/patients")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&patient).unwrap())
                .send()
                .await;

            match client {
                Ok(resp) => {
                    if resp.ok() {
                        set_submit_status.set(Some("Patient registered successfully!".to_string()));
                        // Reset form?
                        set_first_name.set(String::new());
                        set_last_name.set(String::new());
                        set_dob.set(String::new());
                        set_diagnosis.set(String::new());
                    } else {
                        set_submit_status.set(Some(format!("Error: {}", resp.status_text())));
                    }
                }
                Err(e) => {
                    set_submit_status.set(Some(format!("Network Error: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="bg-white p-6 rounded-lg shadow-md max-w-2xl mx-auto">
            <h2 class="text-2xl font-bold mb-6 text-gray-800">"Patient Registration"</h2>

            <form on:submit=on_submit class="space-y-4">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700">"First Name"</label>
                        <input
                            type="text"
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            prop:value=first_name
                            on:input=move |ev| set_first_name.set(event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700">"Last Name"</label>
                        <input
                            type="text"
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            prop:value=last_name
                            on:input=move |ev| set_last_name.set(event_target_value(&ev))
                            required
                        />
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700">"Date of Birth"</label>
                        <input
                            type="date"
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            prop:value=dob
                            on:input=move |ev| set_dob.set(event_target_value(&ev))
                            required
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700">"Gender"</label>
                        <select
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                            prop:value=gender
                            on:change=move |ev| set_gender.set(event_target_value(&ev))
                        >
                            <option value="Male">"Male"</option>
                            <option value="Female">"Female"</option>
                            <option value="Other">"Other"</option>
                        </select>
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700">"Admission Diagnosis"</label>
                    <textarea
                        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm p-2 border"
                        rows="3"
                        prop:value=diagnosis
                        on:input=move |ev| set_diagnosis.set(event_target_value(&ev))
                        required
                    ></textarea>
                </div>

                <div class="pt-4">
                    <button
                        type="submit"
                        class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                    >
                        "Register Patient"
                    </button>
                </div>
            </form>

            {move || submit_status.get().map(|status| view! {
                <div class="mt-4 p-4 rounded-md bg-green-50 text-green-700">
                    {status}
                </div>
            })}
        </div>
    }
}
