//! Selection and resolution without a JVM (spec/model.md, validation cases).

use mc_mappings::{
    ClassId, Dataset, FieldId, Launcher, Loader, MethodId, ResolveError, ResolvedMember,
};
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("fixtures/synthetic.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn parse(value: &Value) -> Dataset {
    Dataset::from_json(&value.to_string()).unwrap()
}

fn method(d: &Dataset, id: &str) -> Result<String, ResolveError> {
    let s = d.select("1.21", Launcher::Standard, Loader::None)?;
    let m = s.method(&id.parse::<MethodId>().unwrap())?;
    Ok(format!("{}.{}{}", m.owner, m.runtime_name, m.descriptor))
}

#[test]
fn exact_selection_and_the_only_alias() {
    let d = Dataset::from_json(FIXTURE).unwrap();
    let s = d.select("1.21", Launcher::Standard, Loader::None).unwrap();
    assert_eq!(s.profile.profile_id.as_str(), "example.standard");
    let alias = d
        .select("1.21.0", Launcher::Standard, Loader::None)
        .unwrap();
    assert_eq!(alias.profile.profile_id, s.profile.profile_id);
    for version in [
        "1.21.1",
        "1.21-pre1",
        "1.21-rc1",
        "24w14a",
        "1.21.x",
        "01.21",
        " 1.21",
        "1.21.11",
    ] {
        assert!(
            matches!(
                d.select(version, Launcher::Standard, Loader::None),
                Err(ResolveError::UnsupportedConfiguration { .. })
            ),
            "{version:?}"
        );
    }
    // Recognised version, missing launcher/loader combination: no fallback either.
    let err = d.select("1.21", Launcher::Lunar, Loader::None).unwrap_err();
    assert_eq!(
        err.to_string(),
        "unsupported configuration: Minecraft \"1.21\", launcher lunar, loader none"
    );
    assert!(
        d.select("1.21", Launcher::Standard, Loader::Fabric)
            .is_err()
    );
}

#[test]
fn duplicate_entries_and_missing_profiles_are_invalid_data() {
    let mut data = fixture();
    let entry = data["configurations"][0].clone();
    data["configurations"].as_array_mut().unwrap().push(entry);
    let d = parse(&data);
    assert!(matches!(
        d.select("1.21", Launcher::Standard, Loader::None),
        Err(ResolveError::InvalidData(_))
    ));
    let mut data = fixture();
    data["configurations"][0]["profile_id"] = json!("example.missing");
    let d = parse(&data);
    let err = d
        .select("1.21", Launcher::Standard, Loader::None)
        .unwrap_err();
    assert_eq!(
        err,
        ResolveError::InvalidData("missing profile example.missing".into())
    );
}

#[test]
fn overloads_arrays_and_cross_references() {
    let d = Dataset::from_json(FIXTURE).unwrap();
    // Two overloads with one runtime name: each has its own identifier and descriptor.
    assert_eq!(
        method(&d, "example.client.find").unwrap(),
        "example/runtime/Client.b(Ljava/lang/String;)Lexample/runtime/Client;"
    );
    assert_eq!(
        method(&d, "example.client.find_by_index").unwrap(),
        "example/runtime/Client.b(I)[Lexample/runtime/Client;"
    );
    let s = d.select("1.21", Launcher::Standard, Loader::None).unwrap();
    let field = s
        .field(&"example.client.counter".parse::<FieldId>().unwrap())
        .unwrap();
    assert_eq!(
        field,
        ResolvedMember {
            owner: "example/runtime/Client",
            runtime_name: "a",
            descriptor: "I".into(),
            is_static: false
        }
    );
    let class = s
        .class(&"example.client".parse::<ClassId>().unwrap())
        .unwrap();
    assert_eq!(class.runtime_name, "example/runtime/Client");

    // Nested arrays of mapped and external classes, translated recursively.
    let mut data = fixture();
    data["profiles"][0]["methods"][1]["parameters"] = json!([
        {"kind": "array", "element": {"kind": "array", "element": {"kind": "class", "id": "example.client"}}},
        {"kind": "array", "element": {"kind": "external", "runtime_name": "java/lang/String"}},
        {"kind": "primitive", "name": "long"}
    ]);
    data["profiles"][0]["methods"][1]["returns"] = json!({"kind": "void"});
    data["profiles"][0]["methods"][1]["static"] = json!(true);
    let d = parse(&data);
    assert_eq!(
        method(&d, "example.client.find_by_index").unwrap(),
        "example/runtime/Client.b([[Lexample/runtime/Client;[Ljava/lang/String;J)V"
    );
}

#[test]
fn missing_and_unknown_symbols_are_errors() {
    let d = Dataset::from_json(FIXTURE).unwrap();
    assert_eq!(
        method(&d, "example.client.nope"),
        Err(ResolveError::UnknownSymbol("example.client.nope".into()))
    );
    let mut data = fixture();
    data["profiles"][0]["methods"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    let d = parse(&data);
    assert_eq!(
        method(&d, "example.client.find"),
        Err(ResolveError::InvalidData(
            "profile example.standard lacks method example.client.find".into()
        ))
    );
    // A mapped type whose class record is missing: no silent external fallback.
    let mut data = fixture();
    data["catalog"]["classes"] = json!(["example.client", "example.other"]);
    data["profiles"][0]["methods"][0]["returns"] = json!({"kind": "class", "id": "example.other"});
    let d = parse(&data);
    assert!(matches!(
        method(&d, "example.client.find"),
        Err(ResolveError::InvalidData(_))
    ));
}
