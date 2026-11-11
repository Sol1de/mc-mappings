use mc_mappings::{
    ClassId, Dataset, FieldId, Launcher, Loader, MethodId, ProfileId, Type,
    normalize_minecraft_version,
};
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("fixtures/synthetic.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn parse(value: &Value) -> Result<Dataset, serde_json::Error> {
    Dataset::from_json(&value.to_string())
}

#[test]
fn semantic_identifiers_are_exact_and_typed() {
    for id in ["a", "example.client", "a0_b.c_2"] {
        assert_eq!(id.parse::<ClassId>().unwrap().as_str(), id);
        assert!(id.parse::<FieldId>().is_ok());
        assert!(id.parse::<MethodId>().is_ok());
        assert!(id.parse::<ProfileId>().is_ok());
    }
    for id in [
        "", ".a", "a.", "a..b", "A", "a.B", "1a", "a.0b", "a-b", " a", "a ", "é", "a\nb",
    ] {
        assert!(id.parse::<ClassId>().is_err(), "{id:?}");
        assert!(id.parse::<FieldId>().is_err(), "{id:?}");
        assert!(id.parse::<MethodId>().is_err(), "{id:?}");
        assert!(id.parse::<ProfileId>().is_err(), "{id:?}");
    }
    assert!(serde_json::from_str::<ClassId>("42").is_err());
}

#[test]
fn only_the_exact_input_alias_is_normalized() {
    assert_eq!(normalize_minecraft_version("1.21.0"), "1.21");
    for version in [
        "1.21",
        "1.21.11",
        "1.21.1",
        "1.21-pre1",
        "1.21-rc1",
        "24w14a",
        "1.21.x",
        "01.21",
        " 1.21",
        "1.21.0 ",
        "",
    ] {
        assert_eq!(normalize_minecraft_version(version), version);
    }
}

#[test]
fn synthetic_spec_example_and_distinct_profiles() {
    let dataset = Dataset::from_json(FIXTURE).unwrap();
    assert_eq!(dataset.schema_version, 1);
    assert_eq!(dataset.data_version.get(), 1);
    assert_eq!(dataset.configurations[0].launcher, Launcher::Standard);
    assert_eq!(dataset.configurations[0].loader, Loader::None);
    assert!(matches!(
        dataset.profiles[0].methods[1].returns,
        Type::Array(_)
    ));

    let mut data = fixture();
    let mut lunar = data["profiles"][0].clone();
    lunar["profile_id"] = json!("example.lunar");
    lunar["classes"][0]["runtime_name"] = json!("other/Client");
    data["profiles"].as_array_mut().unwrap().push(lunar);
    data["configurations"].as_array_mut().unwrap().push(json!({
        "minecraft_version": "1.21.11", "launcher": "lunar", "loader": "fabric", "profile_id": "example.lunar"
    }));
    let dataset = parse(&data).unwrap();
    assert_ne!(
        dataset.profiles[0].profile_id,
        dataset.profiles[1].profile_id
    );
    assert_ne!(dataset.profiles[0].classes, dataset.profiles[1].classes);
}

#[test]
fn json_requires_exact_fields_and_types() {
    let cases = [
        ("/schema_version", json!(3)),
        ("/schema_version", json!(1.0)),
        ("/schema_version", json!("1")),
        ("/data_version", json!(0)),
        ("/data_version", json!(-1)),
        ("/data_version", json!(1.5)),
        ("/data_version", json!(null)),
        ("/catalog/classes", json!({})),
        ("/catalog/classes/0", json!("Example.client")),
        ("/configurations/0/minecraft_version", json!(1.21)),
        ("/configurations/0/launcher", json!("Standard")),
        ("/configurations/0/loader", json!("vanilla")),
        ("/profiles/0/profile_id", json!("bad-profile")),
        ("/profiles/0/fields/0/static", json!("false")),
        ("/profiles/0/methods/0/parameters", json!(null)),
        ("/profiles/0/fields/0/type", json!("I")),
    ];
    for (path, value) in cases {
        let mut data = fixture();
        *data.pointer_mut(path).unwrap() = value;
        assert!(parse(&data).is_err(), "{path}: {data}");
    }
    for path in [
        "",
        "/catalog",
        "/configurations/0",
        "/profiles/0",
        "/profiles/0/classes/0",
        "/profiles/0/fields/0",
        "/profiles/0/methods/0",
        "/profiles/0/fields/0/type",
    ] {
        let original = fixture();
        let object = original.pointer(path).unwrap().as_object().unwrap();
        for key in object.keys() {
            let mut missing = original.clone();
            missing
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(parse(&missing).is_err(), "missing {path}/{key}");
        }
        let mut extra = original.clone();
        extra
            .pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(true));
        assert!(parse(&extra).is_err(), "unknown field in {path}");
        // Construct raw JSON: Value would discard duplicate keys before parsing.
        let object_text = object
            .iter()
            .map(|(key, value)| format!("{key:?}:{value}"))
            .collect::<Vec<_>>()
            .join(",");
        for (key, value) in object {
            let duplicate = format!("{{{object_text},{key:?}:{value}}}");
            let raw = original.to_string().replacen(
                &original.pointer(path).unwrap().to_string(),
                &duplicate,
                1,
            );
            assert!(Dataset::from_json(&raw).is_err(), "duplicate {path}/{key}");
        }
    }
    for raw in [
        format!("{FIXTURE} null"),
        FIXTURE.trim_end().trim_end_matches('}').into(),
        "[]".into(),
    ] {
        assert!(Dataset::from_json(&raw).is_err());
    }
}

#[test]
fn enum_values_must_be_json_strings() {
    for (path, name) in [
        ("/configurations/0/launcher", "standard"),
        ("/configurations/0/loader", "none"),
        ("/profiles/0/fields/0/type/name", "int"),
    ] {
        for invalid in [
            json!({name: null}),
            json!([name]),
            json!(null),
            json!(1),
            json!(true),
        ] {
            let mut data = fixture();
            *data.pointer_mut(path).unwrap() = invalid;
            let raw = data.to_string();
            assert!(Dataset::from_json(&raw).is_err(), "{path}: {raw}");
            assert!(
                serde_json::from_str::<Dataset>(&raw).is_err(),
                "{path}: {raw}"
            );
        }
    }
}

#[test]
fn positional_arrays_cannot_replace_objects() {
    let cases = [
        (
            "",
            json!([1, 1, {"classes":[], "fields":[], "methods":[]}, [], []]),
        ),
        (
            "/catalog",
            json!([
                ["example.client"],
                ["example.client.counter"],
                ["example.client.find", "example.client.find_by_index"]
            ]),
        ),
        (
            "/configurations/0",
            json!(["1.21", "standard", "none", "example.standard"]),
        ),
        ("/profiles/0", json!(["example.standard", [], [], []])),
        (
            "/profiles/0/classes/0",
            json!(["example.client", "example/runtime/Client"]),
        ),
        (
            "/profiles/0/fields/0",
            json!(["example.client.counter", "example.client", "a", false, {"kind":"primitive", "name":"int"}]),
        ),
        (
            "/profiles/0/methods/0",
            json!(["example.client.find", "example.client", "b", false, [], {"kind":"void"}]),
        ),
    ];
    for (path, value) in cases {
        let mut data = fixture();
        *data.pointer_mut(path).unwrap() = value;
        assert!(parse(&data).is_err(), "{path}");
    }
}

#[test]
fn stored_versions_and_configurations_are_canonical_and_in_scope() {
    for version in ["1.21.0", "1.21.1", "1.21-pre1", "01.21", " 1.21"] {
        let mut data = fixture();
        data["configurations"][0]["minecraft_version"] = json!(version);
        assert!(parse(&data).is_err(), "{version}");
    }
    for launcher in ["standard", "lunar"] {
        for loader in ["none", "fabric", "forge"] {
            let mut data = fixture();
            data["configurations"][0]["launcher"] = json!(launcher);
            data["configurations"][0]["loader"] = json!(loader);
            assert_eq!(
                parse(&data).is_ok(),
                !(launcher == "lunar" && loader == "forge")
            );
        }
    }
}

#[test]
fn runtime_names_are_validated() {
    for path in [
        "/profiles/0/classes/0/runtime_name",
        "/profiles/0/methods/0/parameters/0/runtime_name",
    ] {
        for name in ["", "a.b", "La;", "[a", "/a", "a/", "a//b", "a b", "a\tb"] {
            let mut data = fixture();
            *data.pointer_mut(path).unwrap() = json!(name);
            assert!(parse(&data).is_err(), "{path} {name:?}");
        }
    }
    for path in [
        "/profiles/0/fields/0/runtime_name",
        "/profiles/0/methods/0/runtime_name",
    ] {
        for name in ["", "a.b", "a/b", "a;", "[a", "a b", "<other>", "a>"] {
            let mut data = fixture();
            *data.pointer_mut(path).unwrap() = json!(name);
            assert!(parse(&data).is_err(), "{path} {name:?}");
        }
    }
}

#[test]
fn type_objects_are_exact_and_void_is_return_only() {
    for name in [
        "boolean", "byte", "char", "short", "int", "long", "float", "double",
    ] {
        let mut data = fixture();
        data["profiles"][0]["fields"][0]["type"] = json!({"kind":"primitive", "name":name});
        assert!(parse(&data).is_ok(), "{name}");
    }
    for invalid in [
        json!({"kind":"primitive", "name":"void"}),
        json!({"kind":"primitive", "name":"Int"}),
        json!({"kind":"class", "id":"Bad"}),
        json!({"kind":"external", "runtime_name":"java.lang.String"}),
        json!({"kind":"array", "element":null}),
        json!({"kind":"unknown"}),
        json!({"kind":"void", "name":"int"}),
        json!({"kind":"class", "id":"example.client", "name":"int"}),
    ] {
        let mut data = fixture();
        data["profiles"][0]["methods"][0]["returns"] = invalid;
        assert!(parse(&data).is_err());
    }
    for path in [
        "/profiles/0/fields/0/type",
        "/profiles/0/methods/0/parameters/0",
        "/profiles/0/methods/1/returns/element",
    ] {
        let mut data = fixture();
        *data.pointer_mut(path).unwrap() = json!({"kind":"void"});
        assert!(parse(&data).is_err(), "{path}");
    }
    let mut data = fixture();
    data["profiles"][0]["methods"][0]["returns"] = json!({"kind":"void"});
    assert!(parse(&data).is_ok());
}

#[test]
fn special_method_names_have_local_constraints() {
    for name in ["<init>", "<clinit>"] {
        for is_static in [false, true] {
            for has_parameters in [false, true] {
                for returns_void in [false, true] {
                    let mut data = fixture();
                    let method = &mut data["profiles"][0]["methods"][0];
                    method["runtime_name"] = json!(name);
                    method["static"] = json!(is_static);
                    method["parameters"] = if has_parameters {
                        json!([{"kind":"primitive", "name":"int"}])
                    } else {
                        json!([])
                    };
                    method["returns"] = if returns_void {
                        json!({"kind":"void"})
                    } else {
                        json!({"kind":"primitive", "name":"int"})
                    };
                    let valid = returns_void
                        && if name == "<init>" {
                            !is_static
                        } else {
                            is_static && !has_parameters
                        };
                    assert_eq!(
                        parse(&data).is_ok(),
                        valid,
                        "{name} {is_static} {has_parameters} {returns_void}"
                    );
                }
            }
        }
        let mut data = fixture();
        data["profiles"][0]["fields"][0]["runtime_name"] = json!(name);
        assert!(parse(&data).is_err());
    }
}

#[test]
fn array_dimensions_are_bounded_during_parsing() {
    for dimensions in [1, 254, 255, 256, 1000] {
        let array = format!(
            "{}{{\"kind\":\"primitive\",\"name\":\"int\"}}{}",
            "{\"element\":".repeat(dimensions),
            ",\"kind\":\"array\"}".repeat(dimensions)
        );
        let raw = FIXTURE.replace("{\"kind\": \"primitive\", \"name\": \"int\"}", &array);
        assert_ne!(raw, FIXTURE);
        assert_eq!(
            Dataset::from_json(&raw).is_ok(),
            dimensions <= 255,
            "{dimensions}"
        );
    }
}

#[test]
fn cross_table_auditing_is_separate_from_parsing() {
    let mut data = fixture();
    data["profiles"][0]["classes"] = json!([]);
    data["catalog"]["methods"] = json!(["example.client", "example.client"]);
    let duplicate = data["configurations"][0].clone();
    data["configurations"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(parse(&data).is_ok());
    assert!(!parse(&data).unwrap().audit().is_ok());
    let empty = json!({"schema_version":1, "data_version":1, "catalog":{"classes":[], "fields":[], "methods":[]}, "configurations":[], "profiles":[]});
    assert!(parse(&empty).is_ok());
}

#[test]
fn version_scopes_need_schema_2_and_canonical_versions() {
    let mut data = fixture();
    data["catalog"]["scoped"] =
        json!([{"id": "example.client.counter", "minecraft_versions": ["1.21"]}]);
    assert!(parse(&data).is_err(), "schema 1 has no version scopes");
    data["schema_version"] = json!(2);
    assert_eq!(parse(&data).unwrap().catalog.scoped.len(), 1);
    for versions in [json!([]), json!(["1.21.1"]), json!("1.21")] {
        data["catalog"]["scoped"][0]["minecraft_versions"] = versions;
        assert!(parse(&data).is_err(), "{data}");
    }
    data["catalog"]["scoped"] = json!([{"id": "example.client.counter"}]);
    assert!(parse(&data).is_err());
}
