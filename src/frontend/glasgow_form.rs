use crate::uci::scale::glasgow::{GlasgowRequest, GlasgowResponse};
use leptos::*;
use reqwasm::http::Request;

/// Glasgow Coma Scale form component
#[component]
pub fn GlasgowForm() -> impl IntoView {
    // Reactive signals for form inputs
    let (eye_value, set_eye_value) = create_signal(4u8);
    let (verbal_value, set_verbal_value) = create_signal(5u8);
    let (motor_value, set_motor_value) = create_signal(6u8);

    // Resource that triggers when any input changes
    let glasgow_resource = create_resource(
        move || (eye_value.get(), verbal_value.get(), motor_value.get()),
        move |(eye, verbal, motor)| async move {
            let request = GlasgowRequest { eye, verbal, motor };

            // Call the API
            let response = Request::post("/api/glasgow")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&request).unwrap())
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.ok() {
                        resp.json::<GlasgowResponse>().await.ok()
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        },
    );

    view! {
        <div class="glasgow-form">
            <h2>"Glasgow Coma Scale"</h2>
            <p class="description">
                "Assess the level of consciousness in ICU patients"
            </p>

            <div class="form-grid">
                // Eye Opening Response
                <div class="form-group">
                    <label for="eye-response">
                        <strong>"Eye Opening Response"</strong>
                        " (1-4 points)"
                    </label>
                    <select
                        id="eye-response"
                        on:change=move |ev| {
                            let value = event_target_value(&ev).parse().unwrap_or(4);
                            set_eye_value.set(value);
                        }
                        prop:value=move || eye_value.get().to_string()
                    >
                        <option value="4">"4 - Spontaneous"</option>
                        <option value="3">"3 - To verbal command"</option>
                        <option value="2">"2 - To pain"</option>
                        <option value="1">"1 - No response"</option>
                    </select>
                </div>

                // Verbal Response
                <div class="form-group">
                    <label for="verbal-response">
                        <strong>"Verbal Response"</strong>
                        " (1-5 points)"
                    </label>
                    <select
                        id="verbal-response"
                        on:change=move |ev| {
                            let value = event_target_value(&ev).parse().unwrap_or(5);
                            set_verbal_value.set(value);
                        }
                        prop:value=move || verbal_value.get().to_string()
                    >
                        <option value="5">"5 - Oriented and conversing"</option>
                        <option value="4">"4 - Disoriented and conversing"</option>
                        <option value="3">"3 - Inappropriate words"</option>
                        <option value="2">"2 - Incomprehensible sounds"</option>
                        <option value="1">"1 - No response"</option>
                    </select>
                </div>

                // Motor Response
                <div class="form-group">
                    <label for="motor-response">
                        <strong>"Motor Response"</strong>
                        " (1-6 points)"
                    </label>
                    <select
                        id="motor-response"
                        on:change=move |ev| {
                            let value = event_target_value(&ev).parse().unwrap_or(6);
                            set_motor_value.set(value);
                        }
                        prop:value=move || motor_value.get().to_string()
                    >
                        <option value="6">"6 - Obeys commands"</option>
                        <option value="5">"5 - Localizes pain"</option>
                        <option value="4">"4 - Withdrawal from pain"</option>
                        <option value="3">"3 - Flexion to pain"</option>
                        <option value="2">"2 - Extension to pain"</option>
                        <option value="1">"1 - No response"</option>
                    </select>
                </div>
            </div>

            // Results display with Suspense for async loading
            <div class="results">
                <Suspense fallback=move || view! { <p>"Calculating..."</p> }>
                    {move || {
                        glasgow_resource.get().flatten().map(|data| {
                            view! {
                                <div class="result-card">
                                    <h3>"Results"</h3>
                                    <div class="score-display">
                                        <span class="score-label">"Total Score:"</span>
                                        <span class="score-value">{data.score}" / 15"</span>
                                    </div>
                                    <div class="diagnosis">
                                        <strong>"Diagnosis: "</strong>
                                        {data.diagnosis}
                                    </div>
                                    <div class="recommendation">
                                        <strong>"Recommendation: "</strong>
                                        {data.recommendation}
                                    </div>
                                </div>
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
