//! Cross-table audit: inconsistent or incomplete tables fail with precise diagnostics.

use mc_mappings::{Dataset, Problem};
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("fixtures/synthetic.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn problems(data: &Value) -> Vec<Problem> {
    Dataset::from_json(&data.to_string())
        .unwrap()
        .audit()
        .problems
}

fn has(problems: &[Problem], symbol: Option<&str>, text: &str) -> bool {
    problems
        .iter()
        .any(|p| p.symbol.as_deref() == symbol && p.message.contains(text))
}

#[test]
fn consistent_tables_pass() {
    let report = Dataset::from_json(FIXTURE).unwrap().audit();
    assert!(report.is_ok(), "{report:#?}");
    assert_eq!(report.required, [1, 1, 2]);
    let c = &report.configurations[0];
    assert_eq!(
        (c.classes, c.fields, c.methods, c.complete),
        (1, 1, 2, true)
    );
}

#[test]
fn incomplete_table_fails_per_symbol() {
    let mut data = fixture();
    data["profiles"][0]["fields"] = json!([]);
    let report = Dataset::from_json(&data.to_string()).unwrap().audit();
    assert!(!report.is_ok());
    assert!(has(
        &report.problems,
        Some("example.client.counter"),
        "missing field record"
    ));
    let c = &report.configurations[0];
    assert!(!c.complete);
    assert_eq!(c.fields, 0);
    let p = report
        .problems
        .iter()
        .find(|p| p.configuration.is_some())
        .unwrap();
    assert_eq!(p.configuration.as_deref(), Some("1.21 standard none"));
    assert_eq!(p.symbol.as_deref(), Some("example.client.counter"));
}

#[test]
fn inconsistent_tables_fail() {
    // Duplicate identifiers across kinds, duplicate configuration, missing profile.
    let mut data = fixture();
    data["catalog"]["fields"] = json!(["example.client"]);
    let p = problems(&data);
    assert!(has(
        &p,
        Some("example.client"),
        "duplicate catalog identifier"
    ));
    let mut data = fixture();
    let entry = data["configurations"][0].clone();
    data["configurations"].as_array_mut().unwrap().push(entry);
    assert!(
        problems(&data)
            .iter()
            .any(|p| p.message == "duplicate configuration")
    );
    let mut data = fixture();
    data["configurations"][0]["profile_id"] = json!("example.none");
    assert!(
        problems(&data)
            .iter()
            .any(|p| p.message == "missing profile")
    );
    assert!(
        problems(&data)
            .iter()
            .any(|p| p.message.contains("not used by any configuration"))
    );

    // Records outside the catalog, owners and types without class records.
    let mut data = fixture();
    data["profiles"][0]["classes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "example.extra", "runtime_name": "x/Extra"}));
    assert!(has(
        &problems(&data),
        Some("example.extra"),
        "not in the catalog"
    ));
    let mut data = fixture();
    data["profiles"][0]["fields"][0]["owner"] = json!("example.ghost");
    assert!(has(
        &problems(&data),
        Some("example.client.counter"),
        "owner example.ghost has no class record"
    ));
    let mut data = fixture();
    data["profiles"][0]["fields"][0]["type"] = json!({"kind": "class", "id": "example.ghost"});
    assert!(has(
        &problems(&data),
        Some("example.client.counter"),
        "has no class record"
    ));

    // An external type naming a mapped class, and runtime collisions.
    let mut data = fixture();
    data["profiles"][0]["fields"][0]["type"] =
        json!({"kind": "external", "runtime_name": "example/runtime/Client"});
    assert!(has(
        &problems(&data),
        Some("example.client.counter"),
        "is a mapped class"
    ));
    let mut data = fixture();
    data["profiles"][0]["methods"][1]["parameters"] =
        json!([{"kind": "external", "runtime_name": "java/lang/String"}]);
    data["profiles"][0]["methods"][1]["returns"] = json!({"kind": "class", "id": "example.client"});
    data["profiles"][0]["methods"][1]["static"] = json!(true);
    assert!(has(
        &problems(&data),
        Some("example.client.find_by_index"),
        "is mapped twice"
    ));
    let mut data = fixture();
    data["catalog"]["classes"] = json!(["example.client", "example.other"]);
    data["profiles"][0]["classes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": "example.other", "runtime_name": "example/runtime/Client"}));
    assert!(has(&problems(&data), Some("example.other"), "mapped twice"));
}

#[test]
fn report_serializes_for_ci() {
    let mut data = fixture();
    data["profiles"][0]["methods"] = json!([]);
    let report = Dataset::from_json(&data.to_string()).unwrap().audit();
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["configurations"][0]["complete"], false);
    assert_eq!(json["configurations"][0]["launcher"], "standard");
    assert!(json["problems"].as_array().unwrap().len() >= 2);
}

/// The fixture in schema 2: `find_by_index` exists in 1.21 only, 1.21.11 has its own profile.
fn scoped() -> Value {
    let mut data = fixture();
    data["schema_version"] = json!(2);
    data["catalog"]["scoped"] =
        json!([{"id": "example.client.find_by_index", "minecraft_versions": ["1.21"]}]);
    let mut newer = data["profiles"][0].clone();
    newer["profile_id"] = json!("example.newer");
    newer["methods"].as_array_mut().unwrap().pop();
    data["profiles"].as_array_mut().unwrap().push(newer);
    data["configurations"].as_array_mut().unwrap().push(json!({
        "minecraft_version": "1.21.11", "launcher": "standard", "loader": "none",
        "profile_id": "example.newer"
    }));
    data
}

#[test]
fn version_scopes_are_audited() {
    let report = Dataset::from_json(&scoped().to_string()).unwrap().audit();
    assert!(report.is_ok(), "{report:#?}");
    assert_eq!(
        (
            report.configurations[0].methods,
            report.configurations[1].methods
        ),
        (2, 1)
    );
    let symbol = Some("example.client.find_by_index");
    // A record for a symbol that does not exist in the profile's version.
    let mut bad = scoped();
    let extra = bad["profiles"][0]["methods"][1].clone();
    bad["profiles"][1]["methods"]
        .as_array_mut()
        .unwrap()
        .push(extra);
    assert!(has(&problems(&bad), symbol, "not in the catalog"));
    // No record where the symbol exists.
    let mut bad = scoped();
    bad["profiles"][0]["methods"].as_array_mut().unwrap().pop();
    assert!(has(&problems(&bad), symbol, "missing method record"));
    // One profile for two versions that do not have the same symbols.
    let mut bad = scoped();
    bad["configurations"][1]["profile_id"] = json!("example.standard");
    assert!(has(&problems(&bad), symbol, "does not always exist"));
    // Repeated scope, and scope for an identifier outside the catalog.
    let mut bad = scoped();
    let scope = bad["catalog"]["scoped"][0].clone();
    bad["catalog"]["scoped"].as_array_mut().unwrap().extend([
        scope,
        json!({"id": "example.unknown", "minecraft_versions": ["1.21"]}),
    ]);
    let found = problems(&bad);
    assert!(has(&found, symbol, "several version scopes"));
    assert!(has(&found, Some("example.unknown"), "not in the catalog"));
}
