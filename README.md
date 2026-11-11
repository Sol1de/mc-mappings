# mc-mappings

Minecraft runtime mapping tables for Rust: select the exact configuration, then resolve semantic
symbols to runtime class names, member names and complete JVM descriptors. Pure data — no JVM,
JNI, JVMTI or dependency on another catalog crate. Windows x64 is the targeted platform.

```rust
use mc_mappings::{Dataset, Launcher, Loader, MethodId};

let tables = Dataset::minecraft_v1()?;                          // bundled tables
let fabric = tables.select("1.21.11", Launcher::Lunar, Loader::Fabric)?;
let id: MethodId = "minecraft.client.get_instance".parse()?;
let m = fabric.method(&id)?;
assert_eq!(m.owner, "net/minecraft/class_310");
assert_eq!((m.runtime_name, m.descriptor.as_str(), m.is_static), ("method_1551", "()Lnet/minecraft/class_310;", true));
```

## Coverage

`data/minecraft-v1.json` (`schema_version` 2, `data_version` 2) covers the
[catalog](spec/catalog.md) on every target configuration: 77 classes, 89 fields and 175 methods in
1.21, 79 classes, 89 fields and 177 methods in 1.21.11 (some packet symbols exist in one version
only).

| Minecraft | Launcher | Loader | Runtime names (verified) |
| --- | --- | --- | --- |
| 1.21, 1.21.11 | standard | none (Vanilla) | obfuscated (`official`) |
| 1.21, 1.21.11 | standard | fabric | Intermediary |
| 1.21, 1.21.11 | standard | forge | Mojang |
| 1.21, 1.21.11 | lunar | none | Mojang (not the Vanilla names) |
| 1.21, 1.21.11 | lunar | fabric | Intermediary |

Configurations whose tables were verified identical share a profile (6 profiles for 10
configurations); their identities stay separate. Any other version (`1.21.1`, snapshots, release
candidates), and Lunar/Forge, is an unsupported configuration. `1.21.0` is the only alias.
Provenance, reference builds and licences: [data/README.md](data/README.md),
[spec/references.md](spec/references.md).

## API and errors

- `Dataset::from_json` parses strictly (unknown/duplicate/missing fields, wrong JSON types,
  invalid identifiers, unsupported schema, local name and signature rules) up to 255 array
  dimensions; `Dataset::validate` rechecks local rules after edits in Rust.
- `Dataset::audit` checks the cross-table rules (unique identifiers and configurations, profiles
  matching the catalog exactly, owners and type references, runtime collisions, external types
  hiding mapped classes) and resolves every required symbol of every configuration. Its
  `AuditReport` (serializable) lists problems per configuration and symbol.
- `Dataset::select(version, launcher, loader)` returns a `Selection`; `class`, `field`, `method`
  and `descriptor` resolve through it. Errors (`ResolveError`):
  `UnsupportedConfiguration` (no exact registry entry, never a fallback), `UnknownSymbol` (not in
  the catalog), `AbsentInVersion` (in the catalog, but the game does not have it in the selected
  Minecraft version), `InvalidData` (duplicate entry, missing profile, symbol, owner or type).

## Checks

```sh
cargo test                         # format, resolver, audit, bundled tables per configuration
cargo run --example audit          # JSON audit report of the bundled tables; exit 1 on problems
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy `-D warnings`, debug and release tests, the audit
and `cargo package`, plus the release automation tests. A push to `main` with a new
`package.version` publishes it to crates.io and tags it `v<version>`:
[spec/release.md](spec/release.md).

## Adding a version or a configuration

1. Decide the addition explicitly and update [spec/versions.md](spec/versions.md),
   [spec/configurations.md](spec/configurations.md) and the evidence.
2. Generate the tables with `mc_mappings_generator` (for Lunar, launch the configuration once,
   then `mc_mappings_generator scan`) and `consumers/mc_mappings/build-dataset.sh`, after adding
   the configuration to that script; bump `data_version`.
3. Extend the version check in `Dataset::validate`, the tests in `tests/minecraft_v1.rs` and the
   coverage table above. Keep every previously supported configuration and its tests.
4. `cargo test` and the audit must pass before publishing.

## Licence

The code and the table format are under the [MIT licence](LICENSE).

The tables in `data/` contain runtime names taken from third-party sources and are distributed
as-is: the MIT licence does not cover those names, which remain subject to their owners' terms.
They are derived from the Mojang client and mappings (© Mojang/Microsoft), Fabric Intermediary
(CC0-1.0) and the Lunar Client class cache; Forge installers (LGPL-2.1-only) were only read to
establish the Forge namespace. Per configuration, the sources and their terms are
recorded in [`data/minecraft-v1.provenance.json`](data/minecraft-v1.provenance.json). This project
is not affiliated with or endorsed by Mojang, Microsoft or Lunar Client.
