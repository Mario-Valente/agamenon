use agamenon::models::CompatibilityLevel;
use agamenon::services::CompatibilityChecker;

#[test]
fn test_backward_compatible_adding_field_with_default() {
    let old_schema = r#"{
        "type": "record",
        "name": "User",
        "fields": [
            {"name": "id", "type": "int"},
            {"name": "name", "type": "string"}
        ]
    }"#;

    let new_schema = r#"{
        "type": "record",
        "name": "User",
        "fields": [
            {"name": "id", "type": "int"},
            {"name": "name", "type": "string"},
            {"name": "email", "type": ["null", "string"], "default": null}
        ]
    }"#;

    let result = CompatibilityChecker::check(new_schema, old_schema, CompatibilityLevel::Backward);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_not_backward_compatible_adding_field_without_default() {
    let old_schema = r#"{
        "type": "record",
        "name": "User",
        "fields": [
            {"name": "id", "type": "int"}
        ]
    }"#;

    let new_schema = r#"{
        "type": "record",
        "name": "User",
        "fields": [
            {"name": "id", "type": "int"},
            {"name": "name", "type": "string"}
        ]
    }"#;

    let result = CompatibilityChecker::check(new_schema, old_schema, CompatibilityLevel::Backward);
    // Should return error when adding field without default
    assert!(result.is_err());
}

#[test]
fn test_forward_compatible_removing_field_with_default() {
    let old_schema = r#"{
        "type": "record",
        "name": "User",
        "fields": [
            {"name": "id", "type": "int"},
            {"name": "temp_field", "type": ["null", "string"], "default": null}
        ]
    }"#;

    let new_schema = r#"{
        "type": "record",
        "name": "User",
        "fields": [
            {"name": "id", "type": "int"}
        ]
    }"#;

    let result = CompatibilityChecker::check(new_schema, old_schema, CompatibilityLevel::Forward);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_none_compatibility() {
    let old_schema = r#"{"type": "string"}"#;
    let new_schema = r#"{"type": "int"}"#;

    let result = CompatibilityChecker::check(new_schema, old_schema, CompatibilityLevel::None);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_numeric_promotion_backward() {
    let old_schema = r#"{
        "type": "record",
        "name": "Test",
        "fields": [{"name": "value", "type": "int"}]
    }"#;

    let new_schema = r#"{
        "type": "record",
        "name": "Test",
        "fields": [{"name": "value", "type": "long"}]
    }"#;

    let result = CompatibilityChecker::check(new_schema, old_schema, CompatibilityLevel::Backward);
    assert!(result.is_ok());
    assert!(result.unwrap());
}
