use mc_mappings::{Dataset, Type};

#[test]
fn rust_mutations_can_be_revalidated() {
    let mut dataset = Dataset::from_json(include_str!("fixtures/synthetic.json")).unwrap();
    dataset.profiles[0].fields[0].field_type = Type::Void;
    assert!(dataset.validate().is_err());
}

#[test]
fn generic_deserialization_checks_local_constraints() {
    let json = include_str!("fixtures/synthetic.json");
    assert!(serde_json::from_str::<Dataset>(json).is_ok());
    let invalid = json.replace("example/runtime/Client", "example.runtime.Client");
    assert!(serde_json::from_str::<Dataset>(&invalid).is_err());
}
