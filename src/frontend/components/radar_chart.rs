use leptos::*;

#[component]
pub fn RadarChart(
    #[prop(into)] data: Vec<f32>,
    #[prop(into)] labels: Vec<String>,
    #[prop(default = 200.0)] size: f32,
    #[prop(into, default = "indigo".to_string())] color: String,
) -> impl IntoView {
    let center = size / 2.0;
    let radius = (size / 2.0) * 0.8;
    let num_points = data.len();

    // Generate concentric circles (grid)
    let grid_circles = (1..=4).map(|i| {
        let r = radius * (i as f32 / 4.0);
        view! { <circle cx=center cy=center r=r fill="none" stroke="#e2e8f0" stroke-width="1" /> }
    }).collect_view();

    // Generate axis lines
    let axis_lines = (0..num_points)
        .map(|i| {
            let angle = (i as f32 * 2.0 * std::f32::consts::PI / num_points as f32)
                - (std::f32::consts::PI / 2.0);
            let x2 = center + radius * angle.cos();
            let y2 = center + radius * angle.sin();
            view! { <line x1=center y1=center x2=x2 y2=y2 stroke="#e2e8f0" stroke-width="1" /> }
        })
        .collect_view();

    // Generate label positions
    let label_elements = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let angle = (i as f32 * 2.0 * std::f32::consts::PI / num_points as f32)
                - (std::f32::consts::PI / 2.0);
            let x = center + (radius + 20.0) * angle.cos();
            let y = center + (radius + 15.0) * angle.sin();

            let text_anchor = if angle.cos() > 0.1 {
                "start"
            } else if angle.cos() < -0.1 {
                "end"
            } else {
                "middle"
            };

            view! {
                <text
                    x=x
                    y=y
                    font-size="10"
                    fill="#64748b"
                    text-anchor=text_anchor
                    dominant-baseline="middle"
                    class="font-semibold"
                >
                    {label.clone()}
                </text>
            }
        })
        .collect_view();

    // Generate the data polygon
    let points = data
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let angle = (i as f32 * 2.0 * std::f32::consts::PI / num_points as f32)
                - (std::f32::consts::PI / 2.0);
            // Normalize value (assuming 0-4 for SOFA)
            let normalized_val = (val / 4.0).min(1.0).max(0.0);
            let d = radius * normalized_val;
            let x = center + d * angle.cos();
            let y = center + d * angle.sin();
            format!("{},{}", x, y)
        })
        .collect::<Vec<_>>()
        .join(" ");

    let fill_color = match color.as_str() {
        "teal" => "rgba(20, 184, 166, 0.3)",
        "red" => "rgba(239, 68, 68, 0.3)",
        "purple" => "rgba(168, 85, 247, 0.3)",
        _ => "rgba(79, 70, 229, 0.3)",
    };

    let stroke_color = match color.as_str() {
        "teal" => "#14b8a6",
        "red" => "#ef4444",
        "purple" => "#a855f7",
        _ => "#4f46e5",
    };

    view! {
        <div class="flex justify-center items-center">
            <svg width=size height=size viewBox=format!("0 0 {} {}", size, size)>
                // Grid
                {grid_circles}
                {axis_lines}

                // Data Area
                <polygon
                    points=points
                    fill=fill_color
                    stroke=stroke_color
                    stroke-width="2"
                    stroke-linejoin="round"
                />

                // Labels
                {label_elements}
            </svg>
        </div>
    }
}
