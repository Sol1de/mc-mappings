//! The bundled Minecraft v1 tables: every target configuration, every required symbol.
//! Expected runtime names were cross-checked against the upstream files (Mojang
//! `client_mappings`, Fabric Intermediary) and the runtime-namespace proofs of the generator.

use mc_mappings::{ClassId, Dataset, FieldId, Launcher, Loader, MethodId, ResolveError};

fn dataset() -> Dataset {
    Dataset::minecraft_v1().unwrap()
}

fn class(d: &Dataset, v: &str, l: Launcher, o: Loader, id: &str) -> String {
    let s = d.select(v, l, o).unwrap();
    s.class(&id.parse::<ClassId>().unwrap())
        .unwrap()
        .runtime_name
        .to_owned()
}

fn method(d: &Dataset, v: &str, l: Launcher, o: Loader, id: &str) -> (String, bool) {
    let s = d.select(v, l, o).unwrap();
    let m = s.method(&id.parse::<MethodId>().unwrap()).unwrap();
    (
        format!("{}.{}{}", m.owner, m.runtime_name, m.descriptor),
        m.is_static,
    )
}

fn field(d: &Dataset, v: &str, l: Launcher, o: Loader, id: &str) -> String {
    let s = d.select(v, l, o).unwrap();
    let f = s.field(&id.parse::<FieldId>().unwrap()).unwrap();
    format!("{}.{}:{}", f.owner, f.runtime_name, f.descriptor)
}

const TARGETS: [(&str, Launcher, Loader); 10] = [
    ("1.21", Launcher::Standard, Loader::None),
    ("1.21", Launcher::Standard, Loader::Fabric),
    ("1.21", Launcher::Standard, Loader::Forge),
    ("1.21", Launcher::Lunar, Loader::None),
    ("1.21", Launcher::Lunar, Loader::Fabric),
    ("1.21.11", Launcher::Standard, Loader::None),
    ("1.21.11", Launcher::Standard, Loader::Fabric),
    ("1.21.11", Launcher::Standard, Loader::Forge),
    ("1.21.11", Launcher::Lunar, Loader::None),
    ("1.21.11", Launcher::Lunar, Loader::Fabric),
];

#[test]
fn audit_passes_on_every_target_configuration() {
    let d = dataset();
    let report = d.audit();
    assert!(report.is_ok(), "{:#?}", report.problems);
    assert_eq!(report.configurations.len(), 10);
    for (v, l, o) in TARGETS {
        let c = report
            .configurations
            .iter()
            .find(|c| {
                c.minecraft_version == v && c.launcher == l.as_str() && c.loader == o.as_str()
            })
            .unwrap_or_else(|| panic!("{v} {l:?} {o:?} missing"));
        assert!(c.complete);
        // Some packet symbols exist in one of the two versions only (spec/catalog.md).
        let expected = if v == "1.21" {
            [77, 89, 175]
        } else {
            [79, 89, 177]
        };
        assert_eq!([c.classes, c.fields, c.methods], expected);
    }
    assert_eq!(report.required, [79, 89, 185]);
}

#[test]
fn version_dependent_symbols() {
    let d = dataset();
    // Same symbol, version-specific type.
    let fall = "minecraft.entity.fall_distance";
    assert!(field(&d, "1.21", Launcher::Lunar, Loader::None, fall).ends_with(".fallDistance:F"));
    assert!(field(&d, "1.21.11", Launcher::Lunar, Loader::None, fall).ends_with(".fallDistance:D"));
    let fov = "minecraft.game_renderer.get_fov";
    let (older, _) = method(&d, "1.21", Launcher::Lunar, Loader::None, fov);
    let (newer, _) = method(&d, "1.21.11", Launcher::Lunar, Loader::None, fov);
    assert!(
        older.ends_with("FZ)D") && newer.ends_with("FZ)F"),
        "{older} {newer}"
    );
    // Packet content that exists in one version only.
    let old: MethodId = "minecraft.set_entity_motion_packet.get_xa".parse().unwrap();
    let new: MethodId = "minecraft.set_entity_motion_packet.get_movement"
        .parse()
        .unwrap();
    for (v, l, o) in TARGETS {
        let s = d.select(v, l, o).unwrap();
        let (present, absent) = if v == "1.21" {
            (&old, &new)
        } else {
            (&new, &old)
        };
        assert!(s.method(present).is_ok(), "{v} {l:?} {o:?}");
        assert!(matches!(
            s.method(absent),
            Err(ResolveError::AbsentInVersion { .. })
        ));
    }
    let older = d.select("1.21", Launcher::Standard, Loader::None).unwrap();
    assert!(matches!(
        older.class(&"minecraft.position_move_rotation".parse().unwrap()),
        Err(ResolveError::AbsentInVersion { .. })
    ));
}

#[test]
fn catalog_page_lists_every_symbol() {
    let doc = include_str!("../spec/catalog.md");
    let d = dataset();
    let row = |class: &str| {
        doc.lines()
            .find(|l| l.starts_with(&format!("| `{class}`")))
            .unwrap_or_else(|| panic!("spec/catalog.md has no row for {class}"))
    };
    for class in &d.catalog.classes {
        row(class.as_str());
    }
    let fields = d.catalog.fields.iter().map(|i| i.as_str());
    for id in fields.chain(d.catalog.methods.iter().map(|i| i.as_str())) {
        let (class, name) = id.rsplit_once('.').unwrap();
        assert!(
            row(class).contains(&format!("`{name}`")),
            "{id} is missing from spec/catalog.md"
        );
    }
}

#[test]
fn classic_launcher_profiles() {
    let d = dataset();
    let get = "minecraft.client.get_instance";
    // Vanilla runs on obfuscated names (Mojang client_mappings: Minecraft -> fgo / gfj).
    assert_eq!(
        method(&d, "1.21", Launcher::Standard, Loader::None, get),
        ("fgo.Q()Lfgo;".into(), true)
    );
    assert_eq!(
        method(&d, "1.21.11", Launcher::Standard, Loader::None, get),
        ("gfj.V()Lgfj;".into(), true)
    );
    assert!(
        field(
            &d,
            "1.21.11",
            Launcher::Standard,
            Loader::None,
            "minecraft.client.player"
        )
        .starts_with("gfj.s:L")
    );
    // Fabric runs on Intermediary names.
    for v in ["1.21", "1.21.11"] {
        assert_eq!(
            method(&d, v, Launcher::Standard, Loader::Fabric, get),
            (
                "net/minecraft/class_310.method_1551()Lnet/minecraft/class_310;".into(),
                true
            )
        );
        assert_eq!(
            field(
                &d,
                v,
                Launcher::Standard,
                Loader::Fabric,
                "minecraft.client.player"
            ),
            "net/minecraft/class_310.field_1724:Lnet/minecraft/class_746;"
        );
        // Forge (>= 1.20.6) runs on Mojang names.
        assert_eq!(
            method(&d, v, Launcher::Standard, Loader::Forge, get),
            (
                "net/minecraft/client/Minecraft.getInstance()Lnet/minecraft/client/Minecraft;"
                    .into(),
                true
            )
        );
    }
}

#[test]
fn lunar_profiles_are_detected_not_assumed() {
    let d = dataset();
    for v in ["1.21", "1.21.11"] {
        // Lunar without loader: Mojang names, unlike standard Vanilla.
        assert_eq!(
            class(&d, v, Launcher::Lunar, Loader::None, "minecraft.client"),
            "net/minecraft/client/Minecraft"
        );
        assert_ne!(
            class(&d, v, Launcher::Lunar, Loader::None, "minecraft.client"),
            class(&d, v, Launcher::Standard, Loader::None, "minecraft.client")
        );
        // Lunar + Fabric: Intermediary names, same table as standard Fabric.
        assert_eq!(
            method(
                &d,
                v,
                Launcher::Lunar,
                Loader::Fabric,
                "minecraft.living_entity.get_health"
            ),
            method(
                &d,
                v,
                Launcher::Standard,
                Loader::Fabric,
                "minecraft.living_entity.get_health"
            )
        );
        let lunar = d.select(v, Launcher::Lunar, Loader::Fabric).unwrap();
        let standard = d.select(v, Launcher::Standard, Loader::Fabric).unwrap();
        assert_eq!(
            lunar.profile.profile_id, standard.profile.profile_id,
            "shared after verification"
        );
        assert_ne!(
            lunar.configuration, standard.configuration,
            "identities stay separate"
        );
    }
    assert!(matches!(
        d.select("1.21.11", Launcher::Lunar, Loader::Forge),
        Err(ResolveError::UnsupportedConfiguration { .. })
    ));
}

#[test]
fn renamed_symbols_keep_their_identity() {
    let d = dataset();
    let id = "minecraft.resource_location";
    assert_eq!(
        class(&d, "1.21", Launcher::Lunar, Loader::None, id),
        "net/minecraft/resources/ResourceLocation"
    );
    assert_eq!(
        class(&d, "1.21.11", Launcher::Lunar, Loader::None, id),
        "net/minecraft/resources/Identifier"
    );
    assert_eq!(
        class(&d, "1.21.11", Launcher::Standard, Loader::Fabric, id),
        "net/minecraft/class_2960"
    );
    assert_eq!(
        method(
            &d,
            "1.21.11",
            Launcher::Standard,
            Loader::Forge,
            "minecraft.resource_key.location"
        ),
        (
            "net/minecraft/resources/ResourceKey.identifier()Lnet/minecraft/resources/Identifier;"
                .into(),
            false
        )
    );
}

#[test]
fn members_resolve_to_their_declaring_class() {
    let d = dataset();
    let (execute, _) = method(
        &d,
        "1.21.11",
        Launcher::Lunar,
        Loader::None,
        "minecraft.blockable_event_loop.execute",
    );
    assert_eq!(
        execute,
        "net/minecraft/util/thread/BlockableEventLoop.execute(Ljava/lang/Runnable;)V"
    );
    let (click, is_static) = method(
        &d,
        "1.21",
        Launcher::Standard,
        Loader::Forge,
        "minecraft.multi_player_game_mode.handle_inventory_mouse_click",
    );
    assert_eq!(
        click,
        "net/minecraft/client/multiplayer/MultiPlayerGameMode.handleInventoryMouseClick(IIILnet/minecraft/world/inventory/ClickType;Lnet/minecraft/world/entity/player/Player;)V"
    );
    assert!(!is_static);
}

#[test]
fn only_the_target_versions_are_supported() {
    let d = dataset();
    assert_eq!(
        d.select("1.21.0", Launcher::Standard, Loader::None)
            .unwrap()
            .profile
            .profile_id,
        d.select("1.21", Launcher::Standard, Loader::None)
            .unwrap()
            .profile
            .profile_id
    );
    for v in ["1.21.1", "1.21.10", "26.1", "1.21.11-rc1"] {
        assert!(
            d.select(v, Launcher::Standard, Loader::None).is_err(),
            "{v}"
        );
    }
}

#[test]
fn symbols_for_the_mc_bridge_write_operations() {
    let d = dataset();
    for (v, l, o) in TARGETS {
        // set_velocity uses the (double, double, double) overload: no Vec3 construction.
        let (m, is_static) = method(&d, v, l, o, "minecraft.entity.set_delta_movement");
        assert!(m.ends_with("(DDD)V") && !is_static, "{v} {l:?} {o:?}: {m}");
        // click_slot passes ClickType enum constants, read as static fields.
        let s = d.select(v, l, o).unwrap();
        let click_type = class(&d, v, l, o, "minecraft.click_type");
        for id in ["pickup", "quick_move", "swap", "throw"] {
            let f = s
                .field(
                    &format!("minecraft.click_type.{id}")
                        .parse::<FieldId>()
                        .unwrap(),
                )
                .unwrap();
            assert!(f.is_static);
            assert_eq!(
                (f.owner, f.descriptor.as_str()),
                (click_type.as_str(), format!("L{click_type};").as_str())
            );
        }
    }
    assert_eq!(
        field(
            &d,
            "1.21.11",
            Launcher::Lunar,
            Loader::None,
            "minecraft.click_type.quick_move"
        ),
        "net/minecraft/world/inventory/ClickType.QUICK_MOVE:Lnet/minecraft/world/inventory/ClickType;"
    );
}
