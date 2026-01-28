use leptos::*;

/// Severity level enum based on clinical scores
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeverityLevel {
    Low,
    Moderate,
    High,
    Critical,
}

impl SeverityLevel {
    /// Determine severity from SOFA score
    pub fn from_sofa(score: u8) -> Self {
        match score {
            0..=5 => Self::Low,
            6..=10 => Self::Moderate,
            11..=15 => Self::High,
            _ => Self::Critical,
        }
    }

    /// Determine severity from APACHE II score
    pub fn from_apache(score: u8) -> Self {
        match score {
            0..=14 => Self::Low,
            15..=24 => Self::Moderate,
            25..=34 => Self::High,
            _ => Self::Critical,
        }
    }

    /// Determine severity from SAPS II score
    pub fn from_saps(score: u8) -> Self {
        match score {
            0..=29 => Self::Low,
            30..=49 => Self::Moderate,
            50..=69 => Self::High,
            _ => Self::Critical,
        }
    }

    /// Determine severity from Glasgow Coma Scale (inverted - lower is worse)
    pub fn from_glasgow(score: u8) -> Self {
        match score {
            13..=15 => Self::Low,
            9..=12 => Self::Moderate,
            6..=8 => Self::High,
            _ => Self::Critical,
        }
    }

    /// Determine severity from NEWS2 score
    pub fn from_news2(score: u8) -> Self {
        match score {
            0..=4 => Self::Low,
            5..=6 => Self::Moderate,
            7..=9 => Self::High,
            _ => Self::Critical,
        }
    }

    /// Get background color CSS class
    pub fn bg_class(&self) -> &'static str {
        match self {
            Self::Low => "bg-green-100",
            Self::Moderate => "bg-yellow-100",
            Self::High => "bg-orange-100",
            Self::Critical => "bg-red-100",
        }
    }

    /// Get text color CSS class
    pub fn text_class(&self) -> &'static str {
        match self {
            Self::Low => "text-green-800",
            Self::Moderate => "text-yellow-800",
            Self::High => "text-orange-800",
            Self::Critical => "text-red-800",
        }
    }

    /// Get border color CSS class
    pub fn border_class(&self) -> &'static str {
        match self {
            Self::Low => "border-green-300",
            Self::Moderate => "border-yellow-300",
            Self::High => "border-orange-300",
            Self::Critical => "border-red-300",
        }
    }

    /// Get icon for severity
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Low => "fa-check-circle",
            Self::Moderate => "fa-exclamation-circle",
            Self::High => "fa-exclamation-triangle",
            Self::Critical => "fa-skull-crossbones",
        }
    }

    /// Get label text
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low => "Bajo Riesgo",
            Self::Moderate => "Moderado",
            Self::High => "Alto Riesgo",
            Self::Critical => "Crítico",
        }
    }
}

// Specific badge components for each scale type

#[component]
pub fn SofaBadge(score: u8) -> impl IntoView {
    let severity = SeverityLevel::from_sofa(score);
    let label = severity.label();

    view! {
        <span class={format!(
            "inline-flex items-center gap-1.5 font-bold rounded-full border px-3 py-1 text-sm {} {} {}",
            severity.bg_class(),
            severity.text_class(),
            severity.border_class()
        )}>
            <i class={format!("fas {} text-sm", severity.icon())}></i>
            <span>{label}</span>
        </span>
    }
}

#[component]
pub fn ApacheBadge(score: u8) -> impl IntoView {
    let severity = SeverityLevel::from_apache(score);
    let label = severity.label();

    view! {
        <span class={format!(
            "inline-flex items-center gap-1.5 font-bold rounded-full border px-3 py-1 text-sm {} {} {}",
            severity.bg_class(),
            severity.text_class(),
            severity.border_class()
        )}>
            <i class={format!("fas {} text-sm", severity.icon())}></i>
            <span>{label}</span>
        </span>
    }
}

#[component]
pub fn SapsBadge(score: u8) -> impl IntoView {
    let severity = SeverityLevel::from_saps(score);
    let label = severity.label();

    view! {
        <span class={format!(
            "inline-flex items-center gap-1.5 font-bold rounded-full border px-3 py-1 text-sm {} {} {}",
            severity.bg_class(),
            severity.text_class(),
            severity.border_class()
        )}>
            <i class={format!("fas {} text-sm", severity.icon())}></i>
            <span>{label}</span>
        </span>
    }
}

#[component]
pub fn GlasgowBadge(score: u8) -> impl IntoView {
    let severity = SeverityLevel::from_glasgow(score);
    let label = severity.label();

    view! {
        <span class={format!(
            "inline-flex items-center gap-1.5 font-bold rounded-full border px-3 py-1 text-sm {} {} {}",
            severity.bg_class(),
            severity.text_class(),
            severity.border_class()
        )}>
            <i class={format!("fas {} text-sm", severity.icon())}></i>
            <span>{label}</span>
        </span>
    }
}

#[component]
pub fn News2Badge(score: u8) -> impl IntoView {
    let severity = SeverityLevel::from_news2(score);
    let label = severity.label();

    view! {
        <span class={format!(
            "inline-flex items-center gap-1.5 font-bold rounded-full border px-3 py-1 text-sm {} {} {}",
            severity.bg_class(),
            severity.text_class(),
            severity.border_class()
        )}>
            <i class={format!("fas {} text-sm", severity.icon())}></i>
            <span>{label}</span>
        </span>
    }
}
