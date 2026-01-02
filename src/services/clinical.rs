/// qSOFA (Quick SOFA) for Sepsis Risk
/// Criteria used:
/// 1. Respiratory Rate >= 22 /min
/// 2. Altered Mentation (GCS < 15)
/// 3. Systolic Blood Pressure <= 100 mmHg
///
/// Returns true if risk is high (>= 2 criteria met)
pub fn calculate_qsofa(
    respiratory_rate: Option<f64>,
    systolic_bp: Option<f64>,
    gcs: Option<u8>,
) -> (u8, bool) {
    let mut score = 0;

    if let Some(rr) = respiratory_rate {
        if rr >= 22.0 {
            score += 1;
        }
    }

    if let Some(sbp) = systolic_bp {
        if sbp <= 100.0 {
            score += 1;
        }
    }

    if let Some(g) = gcs {
        if g < 15 {
            score += 1;
        }
    }

    (score, score >= 2)
}

/// Analyze mortality risk and provide clinical context
/// Returns a string with risk tier and actionable advice.
pub fn analyze_mortality(mortality: f64) -> String {
    if mortality < 5.0 {
        "Low Risk (<5%). Routine monitoring. Maintain standard protocols.".to_string()
    } else if mortality < 15.0 {
        "Moderate Risk (5-15%). Increased surveillance required. Optimized hemodynamic support suggested.".to_string()
    } else if mortality < 30.0 {
        "High Risk (15-30%). Aggressive Monitoring. Consider early invasive monitoring if unstable."
            .to_string()
    } else if mortality < 50.0 {
        "Severe Risk (30-50%). Critical Care. Multi-organ failure likely. Maximize supportive therapy.".to_string()
    } else {
        "Extreme Risk (>50%). Prognosis Guarded. Consider palliative care consultation if appropriate. Daily re-evaluation essential.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qsofa_high_risk() {
        let (score, is_high) = calculate_qsofa(Some(23.0), Some(90.0), Some(15));
        assert_eq!(score, 2);
        assert!(is_high);
    }

    #[test]
    fn test_mortality_analysis() {
        assert!(analyze_mortality(4.0).contains("Low Risk"));
        assert!(analyze_mortality(60.0).contains("Extreme Risk"));
    }
}
