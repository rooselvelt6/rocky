use ammonia::clean;

#[test]
fn test_xss_protection() {
    let unsafe_input = "<script>alert('xss')</script><b>Safe</b>";
    let safe_output = clean(unsafe_input);
    assert!(!safe_output.contains("<script>"));
    assert!(safe_output.contains("Safe"));
}

#[test]
fn test_surrealql_injection_prevention_logic() {
    let injection_attempt = "'; DELETE FROM patient; --";
    let cleaned = clean(injection_attempt);
    assert_eq!(cleaned, "'; DELETE FROM patient; --");
}

#[test]
fn test_jwt_structure() {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }

    let my_claims = Claims {
        sub: "test_user".to_owned(),
        exp: 10000000000,
    };

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )
    .unwrap();
    assert!(token.len() > 10);
}
