//! The generated dataset parses under the strict format rules.

use mc_mappings::Dataset;

#[test]
fn generated_dataset_parses() {
    let data = Dataset::from_json(include_str!("../data/minecraft-v1.json")).unwrap();
    assert_eq!(data.configurations.len(), 10);
    assert!(data.profiles.iter().all(|p| p.classes.len() == data.catalog.classes.len()));
}
