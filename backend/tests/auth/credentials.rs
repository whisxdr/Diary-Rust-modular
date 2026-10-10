//! Authentication and session helpers, exercised without a database.

use dear_my_diary::{
    features::auth::{dto::RegisterRequest, password},
    infrastructure::session,
};

#[test]
fn password_hashes_verify_and_reject() {
    let hash = password::hash("correct horse battery").expect("hashing works");
    assert!(password::verify("correct horse battery", &hash).is_ok());
    assert!(password::verify("wrong password", &hash).is_err());
}

#[test]
fn password_hashes_are_salted() {
    let first = password::hash("same password").expect("hashing works");
    let second = password::hash("same password").expect("hashing works");
    assert_ne!(first, second, "each hash must use a fresh salt");
}

#[test]
fn session_tokens_are_unique_and_hash_stably() {
    let first = session::generate_token();
    let second = session::generate_token();

    assert_ne!(first, second);
    assert_eq!(session::hash_token(&first), session::hash_token(&first));
    assert_ne!(session::hash_token(&first), session::hash_token(&second));
}

#[test]
fn cookie_lookup_ignores_unrelated_names() {
    let header = Some("other=1; dmd_session=abc; extra=2");
    assert_eq!(
        session::read_cookie(header, "dmd_session").as_deref(),
        Some("abc")
    );
    assert_eq!(session::read_cookie(header, "missing"), None);
    assert_eq!(session::read_cookie(None, "dmd_session"), None);
}

#[test]
fn cookies_are_marked_httponly_and_same_site() {
    let cookie = session::build_cookie("dmd_session", "value", 60);
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    assert!(cookie.contains("Max-Age=60"));

    // The CSRF cookie has to stay readable by the frontend.
    assert!(!session::build_csrf_cookie("dmd_csrf", "value", 60).contains("HttpOnly"));
}

#[test]
fn registration_rules_reject_weak_input() {
    let short_password = RegisterRequest {
        email: "rani@example.com".into(),
        display_name: "Rani".into(),
        password: "short".into(),
    };
    assert!(short_password.validate().is_err());

    let bad_email = RegisterRequest {
        email: "not-an-email".into(),
        display_name: "Rani".into(),
        password: "a long enough password".into(),
    };
    assert!(bad_email.validate().is_err());
    assert_eq!(bad_email.normalized_email(), "not-an-email");

    let good = RegisterRequest {
        email: "  Rani@Example.COM ".into(),
        display_name: "Rani".into(),
        password: "a long enough password".into(),
    };
    assert!(good.validate().is_ok());
    assert_eq!(good.normalized_email(), "rani@example.com");
}
