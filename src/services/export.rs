use crate::models::patient::Patient;
use std::error::Error;

/// Converts a vector of patients to a CSV string.
pub fn patients_to_csv(patients: Vec<Patient>) -> Result<String, Box<dyn Error>> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    // Write header
    wtr.write_record(&[
        "ID",
        "First Name",
        "Last Name",
        "DOB",
        "Gender",
        "Hospital Admission",
        "UCI Admission",
        "Principal Diagnosis",
        "Mortality Prediction", // Placeholders for now, or fetch
    ])?;

    for patient in patients {
        wtr.write_record(&[
            &patient
                .id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_default(),
            &patient.first_name,
            &patient.last_name,
            &patient.date_of_birth,
            &format!("{:?}", patient.gender),
            &patient.hospital_admission_date,
            &patient.uci_admission_date,
            &patient.principal_diagnosis,
            "", // Future: calculate aggregations
        ])?;
    }

    let data = String::from_utf8(wtr.into_inner()?)?;
    Ok(data)
}
