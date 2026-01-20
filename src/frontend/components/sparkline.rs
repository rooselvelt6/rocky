use leptos::*;

#[component]
pub fn Sparkline(
    #[prop(into)] data: Vec<f32>,
    #[prop(default = "red")] color: &'static str,
    #[prop(default = 100)] width: usize,
    #[prop(default = 30)] height: usize,
) -> impl IntoView {
    let points = data
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let x = (i as f32 / (data.len().max(2) - 1) as f32) * width as f32;
            // Normalize y to height (assuming val is 0-max range or relative)
            // Let's assume max value is 15 (for Glasgow) or 50 (Apache) or dynamic?
            // Dynamic max for better visualization
            let max_val = data.iter().cloned().fold(f32::NAN, f32::max).max(1.0);
            let min_val = data.iter().cloned().fold(f32::NAN, f32::min).min(0.0);
            let range = (max_val - min_val).max(1.0);

            let y = height as f32 - ((val - min_val) / range) * (height as f32 - 4.0) - 2.0;
            format!("{},{}", x, y)
        })
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <svg width=format!("{}px", width) height=format!("{}px", height) class="overflow-visible">
            <polyline
                points=points
                fill="none"
                stroke=color
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="animate-draw"
            />
            // Optional: Circle at the end
            {
               if let Some(last_val) = data.last() {
                    let last_x = width;
                    let max_val = data.iter().cloned().fold(f32::NAN, f32::max).max(1.0);
                    let min_val = data.iter().cloned().fold(f32::NAN, f32::min).min(0.0);
                    let range = (max_val - min_val).max(1.0);
                    let last_y = height as f32 - ((last_val - min_val) / range) * (height as f32 - 4.0) - 2.0;
                    view! {
                        <circle cx=last_x cy=last_y r="3" fill=color />
                    }.into_view()
               } else {
                    view! { <g/> }.into_view()
               }
            }
        </svg>
    }
}
