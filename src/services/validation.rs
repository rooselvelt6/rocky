// src/services/validation.rs
use chrono::{DateTime, Duration, Utc};

/// Validates if enough time (24 hours) has passed since the last assessment
/// Returns Ok(()) if valid, Err(String) with error message if not
pub fn validate_24_hour_interval(last_assessment_time: Option<&str>) -> Result<(), String> {
    if let Some(last_time_str) = last_assessment_time {
        // Parse the timestamp
        let last_time = DateTime::parse_from_rfc3339(last_time_str)
            .or_else(|_| {
                // Try parsing as SurrealDB datetime if RFC3339 fails
                DateTime::parse_from_rfc3339(&format!("{}Z", last_time_str))
            })
            .map_err(|e| format!("Failed to parse last assessment time: {}", e))?;

        let now = Utc::now();
        let time_since_last = now.signed_duration_since(last_time.with_timezone(&Utc));

        // Check if 24 hours have passed
        if time_since_last < Duration::hours(24) {
            let hours_remaining = 24 - time_since_last.num_hours();
            return Err(format!(
                "Must wait {} hours before performing another assessment of this type",
                hours_remaining
            ));
        }
    }

    Ok(())
}

/// Represents the validation result with detailed information
#[derive(Debug, serde::Serialize)]
pub struct ValidationResult {
    pub can_assess: bool,
    pub hours_since_last: Option<i64>,
    pub hours_remaining: Option<i64>,
    pub message: Option<String>,
}

/// More detailed validation that returns information about the time constraints
pub fn check_assessment_eligibility(last_assessment_time: Option<&str>) -> ValidationResult {
    if let Some(last_time_str) = last_assessment_time {
        if let Ok(last_time) = DateTime::parse_from_rfc3339(last_time_str)
            .or_else(|_| DateTime::parse_from_rfc3339(&format!("{}Z", last_time_str)))
        {
            let now = Utc::now();
            let time_since_last = now.signed_duration_since(last_time.with_timezone(&Utc));
            let hours_since = time_since_last.num_hours();

            if hours_since < 24 {
                return ValidationResult {
                    can_assess: false,
                    hours_since_last: Some(hours_since),
                    hours_remaining: Some(24 - hours_since),
                    message: Some(format!(
                        "Must wait {} more hours before next assessment",
                        24 - hours_since
                    )),
                };
            } else {
                return ValidationResult {
                    can_assess: true,
                    hours_since_last: Some(hours_since),
                    hours_remaining: Some(0),
                    message: None,
                };
            }
        }
    }

    // No previous assessment or parse error - allow assessment
    ValidationResult {
        can_assess: true,
        hours_since_last: None,
        hours_remaining: None,
        message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_no_previous_assessment() {
        let result = validate_24_hour_interval(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recent_assessment_fails() {
        let one_hour_ago = (Utc::now() - Duration::hours(1)).to_rfc3339();
        let result = validate_24_hour_interval(Some(&one_hour_ago));
        assert!(result.is_err());
    }

    #[test]
    fn test_old_assessment_passes() {
        let twenty_five_hours_ago = (Utc::now() - Duration::hours(25)).to_rfc3339();
        let result = validate_24_hour_interval(Some(&twenty_five_hours_ago));
        assert!(result.is_ok());
    }

    #[test]
    fn test_eligibility_check() {
        let result = check_assessment_eligibility(None);
        assert!(result.can_assess);
        assert_eq!(result.hours_since_last, None);
    }
}
